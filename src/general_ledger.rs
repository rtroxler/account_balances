extern crate chrono;
use chrono::prelude::*;
use std::collections::HashMap;
use std::collections::BTreeMap;

use account_map;

use transaction::Assessment;
use transaction::Payment;
use transaction::Transaction;

use usd::USD;

#[derive(Debug)]
pub struct GeneralLedger { // By Day
    entries: HashMap<(Date<Utc>, String), USD>
}

impl GeneralLedger {
    pub fn new() -> GeneralLedger {
        GeneralLedger {
            entries: HashMap::new()
        }
    }

    pub fn print(&self) {
        // TODO: This is turrible
        println!("|     Date      | Acct | Debit | Credit |");
        println!("-----------------------------------------");
        let ordered: BTreeMap<_, _>  = self.entries.iter().collect();
        for (&(date, ref code), amount) in ordered {
            if amount.pennies > 0 {
                println!("| {} | {} | {:?} |       |", date, code, amount);
            } else if amount.pennies < 0 {
                println!("| {} | {} |       | {:?} |", date, code, amount.inverse());
            } else {
                println!("| {} | {} |       |      |", date, code);
            }
        }
    }

    pub fn record_double_entry(&mut self, date: Date<Utc>, amount: USD,
                           debit_account_code: &String, credit_account_code: &String) {
        {
            let debit = self.entries.entry((date, debit_account_code.clone())).or_insert(USD::zero());
            *debit += amount;
        }
        {
            let credit = self.entries.entry((date, credit_account_code.clone())).or_insert(USD::zero());
            *credit -= amount;
        }
    }

    pub fn fetch_amount(&self, date: Date<Utc>, account_code: String) -> Option<&USD> {
        self.entries.get(&(date, account_code))
    }

    // How to process payments too, enum? with match arms?
    pub fn process_assessment(&mut self, assessment: &Assessment) { // -> Result
        match assessment.account_code() {
            "4000" => self.assess_daily(assessment),
            "4050" => self.assess_monthly(assessment),
            "4100" => println!("Cash do nothing"),
            _ => println!("Fuck")
        }
    }
    fn assess_daily(&mut self, assessment: &Assessment) {
        for (date, amount) in assessment.payee_amount_per_day() {
            self.record_double_entry(date.date(),
                                   amount,
                                   &account_map::accounts_receivable_code(assessment.account_code()),
                                   &assessment.account_code);
        }
    }
    fn assess_monthly(&mut self, assessment: &Assessment) {
        self.record_double_entry(assessment.effective_on.date(),
                               assessment.amount,
                               &account_map::accounts_receivable_code(assessment.account_code()),
                               &assessment.account_code);
    }

    pub fn process_payment(&mut self, payment: &Payment) {
        match payment.account_code() {
            "4000" => self.pay_daily(payment),
            //"4050" => self.pay_monthly(payment), // TODO
            //"4100" => self.pay_cash(payment), // TODO
            _ => println!("Fuck")
        }
    }
    fn pay_daily(&mut self, payment: &Payment) {
        //// For Credits // TODO
        //if self.account_code == String::from("4501") {
            //match self.payee_resolved_on {
                //Some(_date) => println!("Payee is resolved! Do credit things"), // The entries for this might be weird when paired with a payment. But YOLO
                //None => return,
            //}
        //}
        // Calculate how much should go to A/R and how much to defer
        let days_into_service_period = payment.effective_on.signed_duration_since(payment.payee_service_start_date.unwrap());
        let mut days_exclusive: usize = (days_into_service_period.to_std().unwrap().as_secs() / 86_400) as usize;

        let amounts = payment.payee_amount_per_day();
        if days_exclusive > amounts.len() {
            days_exclusive = amounts.len();
        }
        let (ar_days, leftover_days) = amounts.split_at(days_exclusive);

        let creditable_ar = ar_days.iter().fold(USD::zero(), |sum, date_amount| sum + date_amount.1);

        let (ar_to_credit, deferred_amount) = if payment.amount >= creditable_ar {
            (creditable_ar, payment.amount - creditable_ar)
        } else {
            (payment.amount, USD::zero())
        };


        // What if the GL module took care of this -- so that it knows about it's current state
        // enough to treat partial payments etc correctly
        //
        // It should take care of this in fact.
        // And should know how to validate itself (neg A/R etc)
        //
        //  But how, what message does it receive to do this
        //
        //  AB should know the amounts per day
        //  and amounts and that's it? which it pretty much does


        // payment to ar
        self.record_double_entry(payment.effective_on.date(), ar_to_credit, &payment.account_code, &account_map::accounts_receivable_code(&payment.payee_account_code));

        // payment to deferred if applicable
        if deferred_amount > USD::zero() {
            self.record_double_entry(payment.effective_on.date(), deferred_amount, &payment.account_code, &account_map::deferred_code(&payment.payee_account_code));
        }

        // EAT THE AR
        let mut deferred_amount_mut = deferred_amount;
        for &(date, amount) in leftover_days {
            if deferred_amount_mut == USD::zero() {
                //println!("Breaking out of loop");
                break;
            }
            if amount <= deferred_amount_mut {
                self.record_double_entry(date.date(),
                                        amount,
                                        &account_map::deferred_code(&payment.payee_account_code),
                                        &account_map::accounts_receivable_code(&payment.payee_account_code));
                deferred_amount_mut -= amount;
            } else {
                self.record_double_entry(date.date(),
                                        deferred_amount_mut,
                                        &account_map::deferred_code(&payment.payee_account_code),
                                        &account_map::accounts_receivable_code(&payment.payee_account_code));
                deferred_amount_mut = USD::zero();
            }
        }
    }
}

//impl Process for GeneralLedger  // ? DA, payment/assessment, idk
