extern crate chrono;
use chrono::prelude::*;

mod usd;
use usd::USD;

mod general_ledger;
use general_ledger::GeneralLedger;

#[derive(Debug)]
struct Payment {
    amount: USD,
    account_code: String,
    effective_on: DateTime<Utc>,
    payee_amount: USD,
    payee_account_code: String,
    payee_service_start_date: Option<DateTime<Utc>>,
    payee_service_end_date: Option<DateTime<Utc>>,
    payee_effective_on: DateTime<Utc>,
    payee_resolved_on: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct Assessment {
    amount: USD,
    account_code: String,
    effective_on: DateTime<Utc>,
    service_start_date: Option<DateTime<Utc>>,
    service_end_date: Option<DateTime<Utc>>,
}

impl Assessment {
    fn days_in_service_period(&self) -> i64 {
        // TODO: unwrap
        let duration = self.service_end_date.unwrap().signed_duration_since(self.service_start_date.unwrap());
        (duration.to_std().unwrap().as_secs() / 86_400) as i64 + 1
    }

    // Do we take the closed on and if it's within this period roll it up?
    // Or not even write it? Maybe this? Other account balances (write off, prorate, etc) would
    // take care of the rest?
    fn amount_per_day(&self) -> Vec<(DateTime<Utc>, USD)> {
        // TODO: Worry about negative numbers at some point?
        let spd = self.amount.pennies / self.days_in_service_period();
        let mut leftover = self.amount.pennies % self.days_in_service_period();

        (0..self.days_in_service_period()).map(|day| {
            let mut day_amount = spd;
            if leftover > 0 {
                day_amount += 1;
                leftover -= 1;
            }
            (self.service_start_date.unwrap() + chrono::Duration::days(day as i64),
             USD::from_pennies(day_amount) )
        }).collect()
    }
}

// Gross...
impl Payment {
    fn days_in_payee_service_period(&self) -> i64 {
        let duration = self.payee_service_end_date.unwrap().signed_duration_since(self.payee_service_start_date.unwrap());
        (duration.to_std().unwrap().as_secs() / 86_400) as i64 + 1
    }

    fn payee_amount_per_day(&self) -> Vec<(DateTime<Utc>, USD)> {
        // TODO: Worry about negative numbers at some point?
        let spd = self.payee_amount.pennies / self.days_in_payee_service_period();
        let mut leftover = self.payee_amount.pennies % self.days_in_payee_service_period();

        (0..self.days_in_payee_service_period()).map(|day| {
            let mut day_amount = spd;
            if leftover > 0 {
                day_amount += 1;
                leftover -= 1;
            }
            (self.payee_service_start_date.unwrap() + chrono::Duration::days(day as i64),
             USD::from_pennies(day_amount) )
        }).collect()
    }
}

trait Transaction {
    fn process(&self, gl: &mut GeneralLedger);
}

impl Transaction for Payment {
    fn process(&self, gl: &mut GeneralLedger) {
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

        //println!("AR DAYS: {:?}", ar_days);

        let creditable_ar = ar_days.iter().fold(USD::zero(), |sum, date_amount| sum + date_amount.1);
        //println!("AR to credit: {:?}", creditable_ar);

        let (ar_to_credit, deferred_amount) = if self.amount >= creditable_ar {
            (creditable_ar, self.amount - creditable_ar)
        } else {
            (self.amount, USD::zero())
        };
        // payment to ar
        gl.record_double_entry(self.effective_on.date(), ar_to_credit, self.account_code.clone(), String::from("1001"));
        // payment to deferred if applicable
        if deferred_amount > USD::zero() {
            gl.record_double_entry(self.effective_on.date(), deferred_amount, self.account_code.clone(), String::from("2020"));
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
                                        String::from("2020"), // Deferred code
                                        String::from("1001")); // A/R account code
                deferred_amount_mut -= amount;
            } else {
                gl.record_double_entry(date.date(),
                                        deferred_amount_mut,
                                        String::from("2020"), // Deferred code
                                        String::from("1001")); // A/R account code
                deferred_amount_mut = USD::zero();
            }
        }
    }
}
impl Transaction for Assessment {
    fn process(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        for (date, amount) in self.amount_per_day() {
            gl.record_double_entry(date.date(),
                                   amount,
                                   String::from("1101"), // A/R account code
                                   self.account_code.clone());
        }
    }
}



fn process(gl: &mut GeneralLedger, account_balances: Vec<Box<Transaction>>) {
    for ab in account_balances.iter() {
        ab.process(gl);
    }
}

fn main() {
    let rent_charge = Assessment {
        amount: USD::from_float(30.0),
        account_code: String::from("4000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
    };

    let payment = Payment {
        amount: USD::from_float(20.5),
        account_code: String::from("1000"),
        effective_on: Utc.ymd(2017, 12, 2).and_hms(0,0,0),
        payee_amount: USD::from_float(30.0),
        payee_account_code: String::from("4000"),
        payee_service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        payee_service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
        payee_effective_on: Utc.ymd(2017,11,1).and_hms(0,0,0), // Is this needed?
        payee_resolved_on: None
    };

    //let credit = Payment {
        //amount: USD::from_float(20.5),
        //account_code: String::from("4501"),
        //effective_on: Utc.ymd(2017, 12, 2).and_hms(0,0,0),
        //payee_amount: USD::from_float(30.0),
        //payee_account_code: String::from("4000"),
        //payee_service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        //payee_service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
        //payee_effective_on: Utc.ymd(2017,11,1).and_hms(0,0,0), // Is this needed?
        //payee_resolved_on: Some(Utc.ymd(2017,11,4).and_hms(0,0,0))
    //};
    // TODO: Hella broke
    //let payment2 = Payment {
        //amount: USD::from_float(9.5),
        //account_code: String::from("1000"),
        //effective_on: Utc.ymd(2017, 11, 2).and_hms(0,0,0),
        //payee_amount: USD::from_float(30.0),
        //payee_account_code: String::from("4000"),
        //payee_service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        //payee_service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
        //payee_effective_on: Utc.ymd(2017,11,1).and_hms(0,0,0), // Is this needed?
    //};

    let mut gl = GeneralLedger::new();

    let mut account_balances: Vec<Box<Transaction>> = Vec::new();
    account_balances.push(Box::new(rent_charge));
    account_balances.push(Box::new(payment));
    //account_balances.push(Box::new(payment2));
    //account_balances.push(Box::new(credit));

    // collect GL entries
    process(&mut gl, account_balances);
    gl.print();

    //general_ledger::generate();
}

