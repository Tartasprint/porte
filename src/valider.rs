//! A JSON validator

use core::panic;
use std::{env, fs::read_to_string, io::ErrorKind, process::exit};

/// Exit code returned when the input is a valid JSON document.
const EXIT_VALID: i32 = 0;

/// Exit code returned when the input isn't a valid JSON document.
const EXIT_INVALID: i32 = 1;

/// Exit code returned when an internal error in the parser occured
const EXIT_FAILURE: i32 = 2;

use libporte::{err::TokenizeError, parse};
fn main() {
    let s: Vec<String> = env::args().collect();
    let s = &s[1];
    let s = match read_to_string(s) {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == ErrorKind::InvalidData {
                exit(1);
            } else {
                panic!();
            }
        }
    };
    let s = s.chars().peekable();
    exit(match parse(s) {
        Ok(_) => EXIT_VALID,
        Err(TokenizeError::InternalError(e)) => {
            dbg!(e);
            EXIT_FAILURE
        }
        Err(_) => EXIT_INVALID,
    })
}
