//#![deny(missing_docs)]
//! A crate for parsing JSON
#![deny(clippy::panic, clippy::missing_panics_doc)]
#![feature(type_alias_impl_trait)]
mod idioms;
mod lexer;
mod number;
mod token;
mod value;
/// Functions to parse a JSON text
pub mod ast;
/// A representation for bufferized char reading
pub mod chars;
/// Common error types/handling
pub mod err;
/// A stack-based automaton to read a stream of Tokens.
pub mod automaton;
