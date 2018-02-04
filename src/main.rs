extern crate chrono;
use chrono::prelude::*;

mod account_map;

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
    fn process_daily_accrual(&self, gl: &mut GeneralLedger);
    fn process_accrual(&self, gl: &mut GeneralLedger);
    fn process_cash(&self, gl: &mut GeneralLedger);
}

impl Transaction for Payment {
    fn process_daily_accrual(&self, _gl: &mut GeneralLedger) {}
    fn process_accrual(&self, _gl: &mut GeneralLedger) {}
    fn process_cash(&self, _gl: &mut GeneralLedger) {}
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
    fn process(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        match self.account_code.as_str() {
            "4000" => self.process_daily_accrual(gl),
            "4050" => self.process_accrual(gl),
            "4100" => self.process_cash(gl),
            _ => println!("Fuck")
        }
    }

    fn process_daily_accrual(&self, gl: &mut GeneralLedger) {
        // We're assessment (charge), write entries based on our account code
        for (date, amount) in self.amount_per_day() {
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
        effective_on: Utc.ymd(2017, 11, 2).and_hms(0,0,0),
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
        //payee_resolved_on: None
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

#[test]
fn test_rent_account_balance_accrues_daily() {
    let rent_charge = Assessment {
        amount: USD::from_float(30.0),
        account_code: String::from("4000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
    };

    let mut gl = GeneralLedger::new();
    rent_charge.process(&mut gl);
    let start = rent_charge.service_start_date.unwrap().date();
    let end = rent_charge.service_end_date.unwrap().date();

    let mut date_stepper = start;
    while date_stepper <= end {
        assert_eq!(gl.fetch_amount(date_stepper, String::from("1101")), Some(&USD::from_float(1.0)));
        assert_eq!(gl.fetch_amount(date_stepper, String::from("4000")), Some(&USD::from_float(-1.0)));

        date_stepper = date_stepper.checked_add_signed(chrono::Duration::days(1))
            .expect("Overflow");
    }
}

#[test]
fn test_a_full_payment_against_rent() {
    let mut gl = GeneralLedger::new();

    let rent_charge = Assessment {
        amount: USD::from_float(30.0),
        account_code: String::from("4000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
    };

    let payment = Payment {
        amount: USD::from_float(30.0),
        account_code: String::from("1000"),
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        payee_amount: USD::from_float(30.0),
        payee_account_code: String::from("4000"),
        payee_service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
        payee_service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
        payee_effective_on: Utc.ymd(2017,11,1).and_hms(0,0,0), // Is this needed?
        payee_resolved_on: None
    };

    rent_charge.process(&mut gl);
    payment.process(&mut gl);

    assert_eq!(gl.fetch_amount(payment.effective_on.date(), String::from("1000")), Some(&USD::from_float(30.0)));
    assert_eq!(gl.fetch_amount(payment.effective_on.date(), String::from("2020")), Some(&USD::from_float(-29.0)));
    assert_eq!(gl.fetch_amount(payment.effective_on.date(), String::from("4000")), Some(&USD::from_float(-1.0)));

    let start = rent_charge.service_start_date.unwrap().date();
    let end = rent_charge.service_end_date.unwrap().date();
    let mut date_stepper = start.checked_add_signed(chrono::Duration::days(1)).expect("Overflow");
    while date_stepper <= end {
        assert_eq!(gl.fetch_amount(date_stepper, String::from("2020")), Some(&USD::from_float(1.0)));
        assert_eq!(gl.fetch_amount(date_stepper, String::from("4000")), Some(&USD::from_float(-1.0)));

        date_stepper = date_stepper.checked_add_signed(chrono::Duration::days(1))
            .expect("Overflow");
    }
}

#[test]
fn test_fee_account_balance_accrues_periodically() {
    let fee_charge = Assessment {
        amount: USD::from_float(30.0),
        account_code: String::from("4050"), // Fee
        effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
        service_start_date: None,
        service_end_date: None,
    };

    let mut gl = GeneralLedger::new();
    fee_charge.process(&mut gl);

    assert_eq!(gl.fetch_amount(fee_charge.effective_on.date(), String::from("1103")), Some(&USD::from_float(30.0)));
    assert_eq!(gl.fetch_amount(fee_charge.effective_on.date(), String::from("4050")), Some(&USD::from_float(-30.0)));

    // Doesn't have anything the next day
    // assert entries count == 2
}

