use crate::token::Token;

#[derive(Debug, PartialEq, Eq, Clone)]
/// An common error type
pub enum TokenizeError {
    /// The input ended too early
    InputEndedEarly,
    /// Expected an digit but didn't get one
    ExpectedADigit,
    /// Expected an hexdigit but didn't get one
    ExpectedAHexdigit,
    /// A litteral wasn't found !!!(to be hidden)
    LitteralDidntMatch,
    /// A control character wasn't escaped
    ControlCharacterUnescaped,
    /// An unknown escape sequence was read
    UnkownEscapeSequence,
    /// A code point is not a Unicode valid one
    InvalidUnicodeCodePoint,
    /// Somthing that wasn't a valid token was found.
    UnkownToken,
    /// An incomplete surrogate pair occured
    BigMessWithSurrogatePairs,
    /// The result of reading a surrogate pair is invalid UTF8
    InvalidSurrogatePairs,
    /// An unexpected token occured
    UnexpectedToken(Token),
    /// The given input is too long and should already have ended
    InputTooLong,
    /// Occurs when there is a bug
    InternalError(ErrorLoc),
    /// A byte sequence was invalid UTF8
    InvalidUTF8,
}
/// Represents the location in the source code where an error occured
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorLoc {
    file: &'static str,
    line: u32,
    column: u32,
}

impl ErrorLoc {
    /// Creates a new `ErrorLoc`.
    #[must_use]
    pub(crate) fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

/// Shortcut to return an internal error with the current location in the code.
macro_rules! internal_error {
    () => {
        crate::err::TokenizeError::InternalError(crate::err::ErrorLoc::new(
            file!(),
            line!(),
            column!(),
        ))
    };
}

#[cfg(debug_assertions)]
macro_rules! debug {
    ($expression:expr) => {
        #[allow(clippy::no_effect)]
        $expression
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($expression:expr) => {
        $expression
    };
}

pub(crate) use debug;
pub(crate) use internal_error;
