//! Module for the representation of the tokens.

use crate::number::Number;
/// Representation of the RFC8259 JSON tokens.
#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(missing_docs)]
pub enum Token {
    ArrayBegin,
    ArrayEnd,
    ObjectBegin,
    ObjectEnd,
    NameSeparator,
    ValueSeparator,
    Number(Number),
    String(String),
    True,
    False,
    Null,
    WhiteSpace,
}