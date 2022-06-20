use std::str::FromStr;

use cron::Schedule;

pub mod token_utils;

pub fn validate_cron_expression (cron: String) -> bool {
    match Schedule::from_str(&cron) {
        Ok(_) => true,
        Err(_) => false,
    }
  
}

pub fn is_password_valid(s: &str) -> bool {
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    for c in s.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_digit(10);
    }

    !has_whitespace && has_upper && has_lower && has_digit && s.len() >= 8 && s.len() <= 128
}