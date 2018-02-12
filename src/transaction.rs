extern crate chrono;
use chrono::prelude::*;

use usd::USD;

// enum Transaction  {
// Payment {
    //amount: USD,
    //account_code: String,
    //effective_on: DateTime<Utc>,
    //payee_amount: USD,
    //payee_account_code: String,
    //payee_service_start_date: Option<DateTime<Utc>>,
    //payee_service_end_date: Option<DateTime<Utc>>,
    //payee_effective_on: DateTime<Utc>,
    //payee_resolved_on: Option<DateTime<Utc>>,
// },
//Assessment {
    //amount: USD,
    //account_code: String,
    //effective_on: DateTime<Utc>,
    //service_start_date: Option<DateTime<Utc>>,
    //service_end_date: Option<DateTime<Utc>>,
//  }
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
}

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
}
