extern crate chrono;
use chrono::prelude::*;
//use std::time::Duration;

use std::fmt;

struct USD {
    pennies: u64
}

impl USD {
    // TODO: Handle invalid floats
    pub fn from_float(d: f64) -> USD {
        let pennies = (d * 100.0) as u64;

        USD {
            pennies: pennies
        }
    }
}

impl fmt::Debug for USD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dollars = self.pennies / 100;
        let cents = self.pennies % 100;
        write!(f, "${}.{}", dollars, cents)
    }
}



#[derive(Debug)]
struct AccountBalance {
    amount: USD,
    account_code: String,
    effective_on: DateTime<Utc>,
    service_start_date: DateTime<Utc>,
    service_end_date: DateTime<Utc>,
    modifier: String,
}

impl AccountBalance {
    fn days_in_service_period(&self) -> u64 {
        let duration = self.service_end_date.signed_duration_since(self.service_start_date);
        (duration.to_std().unwrap().as_secs() / 86_400) + 1
    }

    fn amount_per_day(&self) { //-> [(DateTime<Utc>, USD); 31] {
        //pennies_per_day = pennies / days_in_period
        //leftover_pennies = pennies % days_in_period
        //(1..days_in_period).collect do
        //  amount = pennies_per_day
        //  if leftover_pennies > 0
        //    amount += 1
        //    leftover_pennies -= 1
        //  end
        //  negative ? -(amount / 100.0).to_d : (amount / 100.0).to_d
        //end
        let spd = self.amount.pennies / self.days_in_service_period();
        let leftover = self.amount.pennies % self.days_in_service_period();

        //(0.self.days_in_service_period).map(
        for day in 0..self.days_in_service_period() {
            println!("{}", self.service_start_date + chrono::Duration::days(day as i64));
        };
    }
}

fn main() {
    let rent_charge = AccountBalance {
        amount: USD::from_float(30.0),
        account_code: String::from("1101"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_end_date: Utc.ymd(2017, 11, 30).and_hms(0,0,0),
        modifier: String::from("rent")
    };

    println!("Hello, world!");
    println!("{:?}", rent_charge);
    println!("amount: {:?}", rent_charge.amount);

    //println!("{:?}", rent_charge.days_in_service_period());

    // Next:
    // Daily accrual it!
    rent_charge.amount_per_day();
}
