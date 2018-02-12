extern crate chrono;
use chrono::prelude::*;
use std::collections::HashMap;
use std::collections::BTreeMap;

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
}
