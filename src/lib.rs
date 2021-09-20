//#![deny(missing_docs)]
//! A crate for parsing JSON
#![deny(clippy::panic,clippy::missing_panics_doc)]
#![feature(type_alias_impl_trait)]

use std::{iter::Peekable, str::Chars};
mod lexer;
mod number;
mod idioms;
mod ast;
mod value;
mod token;
pub mod err;

/// Parses a JSON text and returns the corresponding constructed `Value`
pub fn parse(mut s: Peekable<Chars>) -> Result<value::Value,err::TokenizeError> {
    let mut r = Vec::<token::Token>::new();
    loop {
        match lexer::read_token(&mut s) {
            Ok(Some(t)) =>  {
                r.push(t);
            },
            Ok(None) => break,
            Err(e) => return Err(e),
        }
    }
    ast::parse(r)
}