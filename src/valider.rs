//! A JSON validator

use libporte::automaton::Automaton;
use libporte::chars::Chars;
use libporte::{err::TokenizeError};

use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    process::exit,
};

/// Exit code returned when the input is a valid JSON document.
const EXIT_VALID: i32 = 0;

/// Exit code returned when the input isn't a valid JSON document.
const EXIT_INVALID: i32 = 1;

/// Exit code returned when an internal error in the parser occured
const EXIT_FAILURE: i32 = 2;

fn main() {
    let s: Vec<String> = env::args().collect();
    let s = &s[1];
    let s = match File::open(s) {
        Ok(f) => BufReader::new(f).bytes().map(|x| x.unwrap()),
        Err(_) => todo!(),
    };
    
    let mut s = Chars::new(Box::new(s)).peekable();
    let mut a = Automaton::new(&mut s);
    exit(match a.find(|x| x.is_err()) {
        Some(Err(TokenizeError::InternalError(e))) => {
            dbg!(e);
            EXIT_FAILURE
        },
        Some(Err(e)) => {dbg!(e);EXIT_INVALID},
        Some(Ok(_)) => {
            dbg!("invalid path...");
            EXIT_FAILURE
        }
        None => EXIT_VALID,
    });
}
