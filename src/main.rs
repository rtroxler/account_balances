extern crate chrono;
use chrono::prelude::*;

mod account_map;
mod transaction;

use transaction::*;

mod usd;
use usd::USD;

mod general_ledger;
use general_ledger::GeneralLedger;

fn main() {
    // TODO: Some mechanism to receive ledger transactions and process them.
}

#[cfg(test)]
mod intergration_tests {
    use super::*;

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
    fn test_two_partial_payments_against_rent() {
        let mut gl = GeneralLedger::new();

        let rent_charge = Assessment {
            amount: USD::from_float(30.0),
            account_code: String::from("4000"),
            effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
            service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
            service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
        };

        let payment1 = Payment {
            amount: USD::from_float(15.0),
            account_code: String::from("1000"),
            effective_on: Utc.ymd(2017, 11, 1).and_hms(0,0,0),
            payee_amount: USD::from_float(30.0),
            payee_account_code: String::from("4000"),
            payee_service_start_date: Some(Utc.ymd(2017, 11, 1).and_hms(0,0,0)),
            payee_service_end_date: Some(Utc.ymd(2017, 11, 30).and_hms(0,0,0)),
            payee_effective_on: Utc.ymd(2017,11,1).and_hms(0,0,0), // Is this needed?
            payee_resolved_on: None
        };

        let payment2 = Payment {
            amount: USD::from_float(15.0),
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
        payment1.process(&mut gl);
        payment2.process(&mut gl);

        gl.print();

        assert_eq!(gl.fetch_amount(payment1.effective_on.date(), String::from("1000")), Some(&USD::from_float(30.0)));
        assert_eq!(gl.fetch_amount(payment1.effective_on.date(), String::from("2020")), Some(&USD::from_float(-29.0)));
        assert_eq!(gl.fetch_amount(payment1.effective_on.date(), String::from("4000")), Some(&USD::from_float(-1.0)));

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


// TODO
// payments
// 15 + 15 upfront
// 15 + 15 with second on day 20
// 15.5 + 14.5
// 15 + 15, void the first
// void in general
//
// credits
//
// move out / rental termination
}
