use crate::{err::TokenizeError, token::Token};

pub trait Lexer: Iterator<Item = Token> {
    fn report(&self) -> &Option<Result<(),TokenizeError>>;
}