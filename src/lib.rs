//#![deny(missing_docs)]
//! A crate for parsing JSON
#![deny(clippy::panic, clippy::missing_panics_doc)]
#![feature(type_alias_impl_trait)]

use std::iter::Peekable;

use ast::Parser;
use chars::Chars;
mod ast;
pub mod chars;
pub mod err;
mod idioms;
mod lexer;
mod number;
mod token;
mod value;

/// Parses a JSON text and returns the corresponding constructed `Value`
pub fn parse(mut s: Peekable<Chars>) -> Result<value::Value, err::TokenizeError> {
    let mut parser = Parser::new(&mut s);
    parser.parse()
}
