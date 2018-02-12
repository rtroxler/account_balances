extern crate chrono;
use chrono::prelude::*;

use usd::USD;

use account_map;

use general_ledger::GeneralLedger;

// Will not work
// Can't access data without pattern matching it out, which then moves it.
// Not the solution I'm looking for
// enum Transaction  {
// Payment { },
//Assessment { }
//}

#[derive(Debug)]
pub struct Assessment {
    pub amount: USD,
    pub account_code: String,
    pub effective_on: DateTime<Utc>,
    pub service_start_date: Option<DateTime<Utc>>,
    pub service_end_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Payment {
    pub amount: USD,
    pub account_code: String,
    pub effective_on: DateTime<Utc>,
    pub payee_amount: USD,
    pub payee_account_code: String,
    pub payee_service_start_date: Option<DateTime<Utc>>,
    pub payee_service_end_date: Option<DateTime<Utc>>,
    pub payee_effective_on: DateTime<Utc>,
    pub payee_resolved_on: Option<DateTime<Utc>>,
    //previously_paid_amount
    //payee_discount_amount
}
//Void?
//Do we need a credit transaction? Can it just be made a part of payee_discount_amount?
//What if it's a full credit, then it would.

pub trait Transaction {
    fn payee_service_start_date(&self) -> Option<DateTime<Utc>>;
    fn payee_service_end_date(&self) -> Option<DateTime<Utc>>;
    fn payee_amount(&self) -> USD;
    fn account_code(&self) -> &str;

    fn days_in_payee_service_period(&self) -> i64 {
        let duration = self.payee_service_end_date().unwrap().signed_duration_since(self.payee_service_start_date().unwrap());
        (duration.to_std().unwrap().as_secs() / 86_400) as i64 + 1
    }

    //// Do we take the closed on and if it's within this period roll it up?
    //// Or not even write it? Maybe this? Other account balances (write off, prorate, etc) would
    //// take care of the rest?
    fn payee_amount_per_day(&self) -> Vec<(DateTime<Utc>, USD)> {
        // TODO: Worry about negative numbers at some point?
        let spd = self.payee_amount().pennies / self.days_in_payee_service_period();
        let mut leftover = self.payee_amount().pennies % self.days_in_payee_service_period();

        (0..self.days_in_payee_service_period()).map(|day| {
            let mut day_amount = spd;
            if leftover > 0 {
                day_amount += 1;
                leftover -= 1;
            }
            (self.payee_service_start_date().unwrap() + chrono::Duration::days(day as i64),
             USD::from_pennies(day_amount) )
        }).collect()
    }

    // Process
    fn process_daily_accrual(&self, gl: &mut GeneralLedger);
    fn process_accrual(&self, gl: &mut GeneralLedger);
    fn process_cash(&self, gl: &mut GeneralLedger);

    fn process(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        match self.account_code() {
            "4000" => self.process_daily_accrual(gl),
            "4050" => self.process_accrual(gl),
            "4100" => self.process_cash(gl),
            _ => println!("Fuck")
        }
    }
}

impl Transaction for Payment {
    fn account_code(&self) -> &str {
        self.payee_account_code.as_str()
    }
    fn payee_service_start_date(&self) -> Option<DateTime<Utc>> {
        self.payee_service_start_date
    }
    fn payee_service_end_date(&self) -> Option<DateTime<Utc>>  {
        self.payee_service_end_date
    }
    fn payee_amount(&self) -> USD {
        self.payee_amount
    }

    // TODO: these
    fn process_accrual(&self, _gl: &mut GeneralLedger) {}
    fn process_cash(&self, _gl: &mut GeneralLedger) {}
    fn process_daily_accrual(&self, gl: &mut GeneralLedger) {
        // We're a payment, pay for things

        // For Credits
        if self.account_code == String::from("4501") {
            match self.payee_resolved_on {
                Some(_date) => println!("Payee is resolved! Do credit things"), // The entries for this might be weird when paired with a payment. But YOLO
                None => return,
            }
        }

        // How much to A/R?
        let days_into_service_period = self.effective_on.signed_duration_since(self.payee_service_start_date.unwrap());
        let mut days_exclusive: usize = (days_into_service_period.to_std().unwrap().as_secs() / 86_400) as usize;

        let amounts = self.payee_amount_per_day();
        if days_exclusive > amounts.len() {
            days_exclusive = amounts.len();
        }
        let (ar_days, leftover_days) = amounts.split_at(days_exclusive);

        let creditable_ar = ar_days.iter().fold(USD::zero(), |sum, date_amount| sum + date_amount.1);
        //println!("AR to credit: {:?}", creditable_ar);

        let (ar_to_credit, deferred_amount) = if self.amount >= creditable_ar {
            (creditable_ar, self.amount - creditable_ar)
        } else {
            (self.amount, USD::zero())
        };
        // payment to ar
        gl.record_double_entry(self.effective_on.date(), ar_to_credit, &self.account_code, &account_map::accounts_receivable_code(&self.payee_account_code));
        // payment to deferred if applicable
        if deferred_amount > USD::zero() {
            gl.record_double_entry(self.effective_on.date(), deferred_amount, &self.account_code, &account_map::deferred_code(&self.payee_account_code));
        }

        let mut deferred_amount_mut = deferred_amount;
        for &(date, amount) in leftover_days {
            if deferred_amount_mut == USD::zero() {
                println!("Breaking out of loop");
                break;
            }
            if amount <= deferred_amount_mut {
                gl.record_double_entry(date.date(),
                                        amount,
                                        &account_map::deferred_code(&self.payee_account_code),
                                        &account_map::accounts_receivable_code(&self.payee_account_code));
                deferred_amount_mut -= amount;
            } else {
                gl.record_double_entry(date.date(),
                                        deferred_amount_mut,
                                        &account_map::deferred_code(&self.payee_account_code),
                                        &account_map::accounts_receivable_code(&self.payee_account_code));
                deferred_amount_mut = USD::zero();
            }
        }
    }
}

impl Transaction for Assessment {
    fn account_code(&self) -> &str {
        self.account_code.as_str()
    }
    fn payee_service_start_date(&self) -> Option<DateTime<Utc>> {
        self.service_start_date
    }
    fn payee_service_end_date(&self) -> Option<DateTime<Utc>>  {
        self.service_end_date
    }
    fn payee_amount(&self) -> USD {
        self.amount
    }

    fn process_daily_accrual(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        for (date, amount) in self.payee_amount_per_day() {
            gl.record_double_entry(date.date(),
                                   amount,
                                   &account_map::accounts_receivable_code(&self.account_code),
                                   &self.account_code);
        }

    }

    fn process_accrual(&self, gl: &mut GeneralLedger) {
        gl.record_double_entry(self.effective_on.date(),
                               self.amount,
                               &account_map::accounts_receivable_code(&self.account_code),
                               &self.account_code);
    }

    fn process_cash(&self, _gl: &mut GeneralLedger) {
        // Do nothing
    }
}
