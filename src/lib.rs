//#![deny(missing_docs)]
//! A crate for parsing JSON
#![deny(clippy::panic,clippy::missing_panics_doc)]
mod lexer;
mod number;
mod idioms;
mod ast;
mod value;
mod token;
pub mod err;

/// Parses a JSON text and returns the corresponding constructed `Value`
pub fn parse(s: Box<dyn Iterator<Item = char>>) -> Result<value::Value,err::TokenizeError> {
    let s: Vec<char> = s.collect();
    let mut r = Vec::<token::Token>::new();
    let mut pos = 0_usize;
    loop {
        match lexer::read_token(&s, pos) {
            Ok((Some(t), new_pos)) =>  {
                pos = new_pos;
                r.push(t);
            },
            Ok((None, _)) => break,
            Err(e) => return Err(e),
        }
    }
    ast::parse(r)
}