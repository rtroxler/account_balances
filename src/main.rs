extern crate chrono;
use chrono::prelude::*;

mod usd;
use usd::USD;

mod general_ledger;
use general_ledger::GeneralLedger;

#[derive(Debug)]
struct Payment {
    assessment_id: i64,
    amount: USD,
    account_code: String,
    effective_on: DateTime<Utc>,
}

#[derive(Debug)]
struct Assessment {
    id: i64,
    amount: USD,
    account_code: String,
    effective_on: DateTime<Utc>,
    service_start_date: Option<DateTime<Utc>>,
    service_end_date: Option<DateTime<Utc>>,
}

impl Assessment {
    fn days_in_service_period(&self) -> i64 {
        let duration = self.service_end_date.unwrap().signed_duration_since(self.service_start_date.unwrap());
        (duration.to_std().unwrap().as_secs() / 86_400) as i64 + 1
    }

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

trait Transaction {
    fn process(&self, gl: &mut GeneralLedger);
}
impl Transaction for Payment {
    fn process(&self, _gl: &mut GeneralLedger) {
        // We're a payment, pay for things
        println!("\tHere's a payment!");
        println!("Processing {:?}\n", self);
    }
}
impl Transaction for Assessment {
    fn process(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        for (date, amount) in self.amount_per_day() {
            gl.record_double_entry(date.date(),
                                   amount,
                                   String::from("1001"), // A/R account code
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
        id: 1,
        amount: USD::from_float(300.0),
        account_code: String::from("4000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
    };

    let payment = Payment {
        assessment_id: 1,
        amount: USD::from_float(30.0),
        account_code: String::from("1000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
    };

    //// Next:
    // Daily accrual it!
    //println!("{:?}", rent_charge.amount_per_day());
    let mut gl = GeneralLedger::new();

    let mut account_balances: Vec<Box<Transaction>> = Vec::new();
    account_balances.push(Box::new(rent_charge));
    account_balances.push(Box::new(payment));

    // collect GL entries
    process(&mut gl, account_balances);
    gl.print();

    //general_ledger::generate();
}

