//! A JSON validator

use libporte::automaton::{Action, Automaton};
use libporte::chars::Chars;
use libporte::lexer_iter::LexerIter;
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
    
    let s = Chars::new(Box::new(s));
    let end = Automaton::new(Box::new(LexerIter::new(s))).last();

    match end {
        Some(Ok(Action::TheEnd)) => {exit(EXIT_VALID)},
        Some(Err(TokenizeError::InternalError(e))) => {eprintln!("{:?}",e); exit(EXIT_FAILURE)},
        Some(_) => {exit(EXIT_INVALID)},
        None => {exit(EXIT_INVALID)}
    };
}
