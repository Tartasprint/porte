use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizeError {
    InputEndedEarly,
    ExpectedADigit,
    ExpectedAHexdigit,
    LitteralDidntMatch,
    ControlCharacterUnescaped,
    UnkownEscapeSequence,
    InvalidUnicodeCodePoint,
    UnkownToken,
    BigMessWithSurrogatePairs,
    InvalidSurrogatePairs,
    UnexpectedToken(Token),
    InputTooLong,
    /// Occurs when there is a bug
    InternalError(ErrorLoc),
}
/// Represents the location in the source code where an error occured
#[derive(Debug, PartialEq, Eq)]
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
        dbg!($expression)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($expression:expr) => {
        $expression
    };
}

pub(crate) use internal_error;
pub(crate) use debug;
