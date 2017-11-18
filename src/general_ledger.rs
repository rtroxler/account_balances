extern crate chrono;
use chrono::prelude::*;
use std::collections::HashMap;

use usd::USD;

//#[derive(Debug)]
struct GeneralLedger { // By Day
    entries: HashMap<(Date<Utc>, String), USD>
}

impl GeneralLedger {
    fn print(&self) {
        println!("|     Date      | Acct | Amount |");
        println!("---------------------------------");
        for (&(date, ref code), amount) in self.entries.iter() {
            println!("| {} | {} | {:?} |", date, code, amount);
        }
    }
}


pub fn generate() {
    let mut gl = GeneralLedger { entries: HashMap::new() };
    gl.entries.insert((Utc::today(), String::from("4000")), USD::from_float(-30.0));
    gl.entries.insert((Utc::today(), String::from("1000")), USD::from_float(30.0));

    gl.print();
    //println!("{:?}", gl);
    println!("Hi from GL");
}
