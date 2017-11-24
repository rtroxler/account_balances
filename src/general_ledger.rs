extern crate chrono;
use chrono::prelude::*;
use std::collections::HashMap;
use std::collections::BTreeMap;

use usd::USD;

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
            } else {
                println!("| {} | {} |       | {:?} |", date, code, amount.inverse());
            }
        }
    }

    pub fn record_double_entry(&mut self, date: Date<Utc>, amount: USD,
                           debit_account_code: String, credit_account_code: String) {
        {
            let debit = self.entries.entry((date, debit_account_code)).or_insert(USD::from_float(0.0));
            *debit += amount;
        }
        {
            let credit = self.entries.entry((date, credit_account_code)).or_insert(USD::from_float(0.0));
            *credit -= amount;
        }
    }
}

//pub fn generate() {
    //let mut gl = GeneralLedger { entries: HashMap::new() };
    //gl.record_double_entry(Utc::today(), USD::from_float(30.0), String::from("1000"), String::from("4000"));
    //gl.record_double_entry(Utc::today(), USD::from_float(30.0), String::from("1000"), String::from("4000"));

    //gl.print();
    //println!("Hi from GL");
//}