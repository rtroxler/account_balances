pub fn accounts_receivable_code(revenue_code: &str) -> String {
    match revenue_code {
        "4000" => String::from("1101"),
        "4050" => String::from("1103"),
        _ => String::from("FUCK")
    }
}

pub fn deferred_code(revenue_code: &str) -> String {
    match revenue_code {
        "4000" => String::from("2020"),
        _ => String::from("FUCK")
    }
}
