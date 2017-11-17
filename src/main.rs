extern crate chrono;
use chrono::prelude::*;

mod usd;
use usd::USD;


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

    fn amount_per_day(&self) -> Vec<(DateTime<Utc>, USD)> {
        // TODO: Worry about negative numbers?
        let spd = self.amount.pennies / self.days_in_service_period();
        let mut leftover = self.amount.pennies % self.days_in_service_period();

        (0..self.days_in_service_period()).map(|day| {
            let mut day_amount = spd;
            if leftover > 0 {
                day_amount += 1;
                leftover -= 1;
            }
            (self.service_start_date + chrono::Duration::days(day as i64), USD::from_pennies(day_amount) )
        }).collect()
    }
}

fn main() {
    let rent_charge = AccountBalance {
        amount: USD::from_float(33.01),
        account_code: String::from("1101"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_end_date: Utc.ymd(2017, 11, 30).and_hms(0,0,0),
        modifier: String::from("rent")
    };

    println!("{:?}", rent_charge);
    println!("amount: {:?}", rent_charge.amount);

    println!("{:?} days in service period\n", rent_charge.days_in_service_period());

    // Next:
    // Daily accrual it!
    println!("{:?}", rent_charge.amount_per_day());
}
