//! Functions to tokenize the text.

use crate::{
    chars::Chars,
    err::{internal_error, TokenizeError},
    idioms::{self, read_one_or_more, ReaderResult},
    number::{Digit, Number, Sign},
    token::Token,
};
use std::ops::ShlAssign;
pub(crate) struct Lexer<'a> {
    input: &'a mut std::iter::Peekable<Chars>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a mut std::iter::Peekable<Chars>) -> Self {
        Self { input }
    }
}

impl Iterator for Lexer<'_> {
    type Item = ReaderResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        read_token(self.input).transpose()
    }
}

const HI_SURROGATE_MIN: u32 = 0xD800;
const HI_SURROGATE_MAX: u32 = 0xDBFF;
const LO_SURROGATE_MIN: u32 = 0xDC00;
const LO_SURROGATE_MAX: u32 = 0xDFFF;

/// Reads a JSON token.
pub(crate) fn read_token<'a>(
    input: &'a mut std::iter::Peekable<Chars>,
) -> ReaderResult<Option<Token>> {
    if let Some(c) = input.peek() {
        match (c.clone())? {
            '0'..='9' | '-' => match read_number(input) {
                Ok(n) => Ok(Some(Token::Number(n))),
                Err(e) => Err(e),
            },
            '"' => {
                input.next();
                match read_string(input) {
                    Ok(s) => Ok(Some(Token::String(s))),
                    Err(e) => Err(e),
                }
            }
            '[' => {
                input.next();
                Ok(Some(Token::ArrayBegin))
            }
            ']' => {
                input.next();
                Ok(Some(Token::ArrayEnd))
            }
            '{' => {
                input.next();
                Ok(Some(Token::ObjectBegin))
            }
            '}' => {
                input.next();
                Ok(Some(Token::ObjectEnd))
            }
            't' => {
                input.next();
                read_rue(input)?;
                Ok(Some(Token::True))
            }
            'f' => {
                input.next();
                read_alse(input)?;
                Ok(Some(Token::False))
            }
            'n' => {
                input.next();
                read_ull(input)?;
                Ok(Some(Token::Null))
            }
            ',' => {
                input.next();
                Ok(Some(Token::ValueSeparator))
            }
            ':' => {
                input.next();
                Ok(Some(Token::NameSeparator))
            }
            ' ' | '\t' | '\x0A' | '\x0D' => {
                input.next();
                read_white_space(input)?;
                Ok(Some(Token::WhiteSpace))
            }
            _ => Err(TokenizeError::UnkownToken),
        }
    } else {
        Ok(None)
    }
}

/// Reads a RFC 8259 JSON number;
fn read_number<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<Number> {
    let sign = read_neg_sign(input)?;
    let int = read_int(input)?;
    let frac = read_frac(input)?;
    let exp = read_exp(input)?;
    Ok(Number::new(sign, int, frac, exp))
}

/// Reads a decimal digit.
fn read_digit<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<Digit> {
    match input.peek() {
        Some(c) => match c.clone()? {
            '0' => {
                input.next();
                Ok(Digit::D0)
            }
            '1' => {
                input.next();
                Ok(Digit::D1)
            }
            '2' => {
                input.next();
                Ok(Digit::D2)
            }
            '3' => {
                input.next();
                Ok(Digit::D3)
            }
            '4' => {
                input.next();
                Ok(Digit::D4)
            }
            '5' => {
                input.next();
                Ok(Digit::D5)
            }
            '6' => {
                input.next();
                Ok(Digit::D6)
            }
            '7' => {
                input.next();
                Ok(Digit::D7)
            }
            '8' => {
                input.next();
                Ok(Digit::D8)
            }
            '9' => {
                input.next();
                Ok(Digit::D9)
            }
            _ => Err(TokenizeError::ExpectedADigit),
        },
        None => Err(TokenizeError::InputEndedEarly),
    }
}

/// Reads RFC 8259 JSON white space.
fn read_white_space<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<()> {
    while let Some(Ok(' ' | '\t' | '\x0A' | '\x0D')) = input.peek() {
        input.next();
    }
    Ok(())
}

/// Reads the int part of a RFC 8259 JSON number.
fn read_int<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<Vec<Digit>> {
    match read_digit(input) {
        Ok(Digit::D0) => Ok(vec![Digit::D0]),
        Ok(d) => {
            let mut r = vec![d];
            while let Ok(d) = read_digit(input) {
                r.push(d);
            }
            Ok(r)
        }
        Err(e) => Err(e),
    }
}

/// Reads the fractional part of a RFC 8259 JSON number.
fn read_frac<'a>(input: &'a mut std::iter::Peekable<Chars>) -> ReaderResult<Option<Vec<Digit>>> {
    if let Some(c) = input.peek() {
        if c.clone()? == '.' {
            input.next();
            let frac = read_one_or_more!(input, read_digit)?;
            Ok(Some(frac))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Reads the exponential part of a RFC 8259 JSON number.
fn read_exp<'a>(
    input: &'a mut std::iter::Peekable<Chars>,
) -> ReaderResult<Option<(Sign, Vec<Digit>)>> {
    match input.peek().as_ref() {
        Some(Ok('e' | 'E')) => {
            input.next();
            let sign = read_pn_sign(input)?;
            let exp = read_one_or_more!(input, read_digit)?;
            Ok(Some((sign, exp)))
        }
        Some(&c) => {
            dbg!(c);
            Ok(None)
        }
        None => Ok(None),
    }
}

/// Reads an optional negative sign (`'-'`).
fn read_neg_sign<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<Sign> {
    match input.peek() {
        Some(Ok('-')) => match input.next() {
            Some(_) => Ok(Sign::Negative),
            None => Err(internal_error!()),
        },
        Some(_) | None => Ok(Sign::Positive),
    }
}

/// Reads an optional positive sign (`'+'`) or a negative sign (`'-'`).
fn read_pn_sign<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<Sign> {
    match input.peek().as_ref() {
        Some(Ok('-')) => match input.next() {
            Some(_) => Ok(Sign::Negative),
            None => Err(internal_error!()),
        },
        Some(Ok('+')) => match input.next() {
            Some(_) => Ok(Sign::Positive),
            None => Err(internal_error!()),
        },
        Some(&c) => {
            dbg!('y', c);
            Ok(Sign::Positive)
        }
        None => Ok(Sign::Positive),
    }
}

/// Reads RFC8259 JSON string.
fn read_string<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<String> {
    let mut a = String::new();
    while let Some(c) = input.peek() {
        match c.as_ref() {
            Ok('\u{0000}'..='\u{001F}') => return Err(TokenizeError::ControlCharacterUnescaped),
            Ok('\\') => {
                input.next();
                match read_escape_sequence(input) {
                    Ok(r) => a.push(r),
                    Err(e) => return Err(e),
                }
            }
            Ok('"') => {
                input.next();
                return Ok(a);
            }
            Ok(&c) => {
                input.next();
                a.push(c)
            }
            Err(e) => return Err(e.clone()),
        }
    }
    Err(TokenizeError::InputEndedEarly)
}

/// Reads a "rue" (ending of "true").
fn read_rue<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<()> {
    idioms::read_string(input, "rue")
}

/// Reads a "false" (ending of "false").
fn read_alse<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<()> {
    idioms::read_string(input, "alse")
}

/// Reads a "ull" (ending of "null").
fn read_ull<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<()> {
    idioms::read_string(input, "ull")
}

/// Reads an escape sequence (where the escaping character have already been read) from `input` according
/// to `pos`. Escape sequences are defined in the RFC8259 as:
///
/// | Pattern      | Viz  |  description   |  value |
/// |--------------|------|----------------|--------|
/// | %x22         | "    | quotation mark | U+0022 |
/// | %x5C         | \    | reverse solidus| U+005C |
/// | %x2F         | /    | solidus        | U+002F |
/// | %x62         | b    | backspace      | U+0008 |
/// | %x66         | f    | form feed      | U+000C |
/// | %x6E         | n    | line feed      | U+000A |
/// | %x72         | r    | carriage return| U+000D |
/// | %x74         | t    | tab            | U+0009 |
/// | %x75 4HEXDIG | uXXXX|                | U+XXXX |
fn read_escape_sequence<'a>(input: &'a mut std::iter::Peekable<Chars>) -> ReaderResult<char> {
    if let Some(c) = input.next() {
        match c? {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => {
                let mut a: u32 = 0;
                for _ in 0..4 {
                    match read_hexdigit(input) {
                        Ok(r) => {
                            a.shl_assign(4);
                            a += u32::from(r);
                        }
                        Err(e) => return Err(e),
                    };
                }
                if HI_SURROGATE_MIN <= a && a <= HI_SURROGATE_MAX {
                    idioms::read_string(input, "\\u")?;
                    let mut b: u32 = 0;
                    for _ in 0..4 {
                        match read_hexdigit(input) {
                            Ok(r) => {
                                crate::err::debug!("Surrogating");
                                b.shl_assign(4);
                                b += u32::from(r);
                            }
                            Err(e) => return Err(e),
                        };
                    }
                    if LO_SURROGATE_MIN <= b && b <= LO_SURROGATE_MAX {
                        let mut code: u32 = 0x1_0000;
                        code += (a & 0x03FF) << 10;
                        code += b & 0x03FF;
                        match std::char::from_u32(code) {
                            Some(c) => Ok(c),
                            None => Err(TokenizeError::InvalidSurrogatePairs),
                        }
                    } else {
                        Err(TokenizeError::BigMessWithSurrogatePairs)
                    }
                } else {
                    match std::char::from_u32(a) {
                        Some(c) => Ok(c),
                        None => Err(TokenizeError::InvalidUnicodeCodePoint),
                    }
                }
            }
            _ => Err(TokenizeError::UnkownEscapeSequence),
        }
    } else {
        Err(TokenizeError::InputEndedEarly)
    }
}

/// Reads one characters from input and convert it to u8 considering it as an hexdigit.
///
/// As specified in RFC8259 a hexdigit is a character from `'0'` to `'9'` or from `'a'` to `'f'`
/// (or from `'A'` to `'F'` since it is case insensitive).
///
/// # Example
///
/// ```ignore
/// use libporte::lexer::read_hexdigit;
///
/// mut let s: Vec<char> = "2";
/// let mut s = Chars::from(s).peekable();
/// let r = read_hexdigitmut (&);
/// assert_eq!(Ok((2u8, 1)), r);
/// ```
fn read_hexdigit<'a>(input: &mut std::iter::Peekable<Chars>) -> ReaderResult<u8> {
    match input.next() {
        Some(Ok(c)) => match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'a'..='f' => Ok(c as u8 - b'a' + 10_u8),
            'A'..='F' => Ok(c as u8 - b'A' + 10_u8),
            _ => Err(TokenizeError::ExpectedAHexdigit),
        },
        Some(Err(e)) => Err(e),
        None => Err(TokenizeError::InputEndedEarly),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_read_x {
        ($($func_name:ident),*) => {
            $(
                mod reader_interface {
                    use super::$func_name;
                    use crate::{chars::Chars,err::TokenizeError};
                    #[test]
                    fn empty_input(){
                        let s = "";
                        let mut s = Chars::from(s).peekable();
                        let r = $func_name(&mut s);
                        assert_eq!(Err(TokenizeError::InputEndedEarly), r);
                    }
                }
            )*
        };
    }

    mod read_hexdigit {
        use crate::{chars::Chars, err::TokenizeError};

        use super::super::read_hexdigit;

        test_read_x! {read_hexdigit}

        #[test]
        fn digits() {
            let s = "0";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(0_u8), r);
            let s = "1";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(1_u8), r);
            let s = "2";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(2_u8), r);
            let s = "3";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(3_u8), r);
            let s = "4";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(4_u8), r);
            let s = "5";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(5_u8), r);
            let s = "6";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(6_u8), r);
            let s = "7";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(7_u8), r);
            let s = "8";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(8_u8), r);
            let s = "9";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(9_u8), r);
        }

        #[test]
        fn lower_case() {
            let s = "a";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(10_u8), r);
            let s = "b";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(11_u8), r);
            let s = "c";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(12_u8), r);
            let s = "d";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(13_u8), r);
            let s = "e";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(14_u8), r);
            let s = "f";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(15_u8), r);
        }

        #[test]
        fn upper_case() {
            let s = "A";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(10_u8), r);
            let s = "B";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(11_u8), r);
            let s = "C";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(12_u8), r);
            let s = "D";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(13_u8), r);
            let s = "E";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(14_u8), r);
            let s = "F";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Ok(15_u8), r);
        }

        #[test]
        fn invalid() {
            let s = "z";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "@";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "One";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "|ab";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "\0";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "\t";
            let mut s = Chars::from(s).peekable();
            let r = read_hexdigit(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
        }
    }
    mod read_escape_sequence {
        use super::super::read_escape_sequence;
        use crate::{chars::Chars, err::TokenizeError};
        test_read_x! {read_escape_sequence}

        #[test]
        fn valid_single_chars() {
            let s = "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('"'), r);
            let s = "\\";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\\'), r);
            let s = "/";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('/'), r);
            let s = "b";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{0008}'), r);
            let s = "f";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{000C}'), r);
            let s = "n";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\n'), r);
            let s = "r";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\r'), r);
            let s = "t";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\t'), r);
        }

        #[test]
        fn invalid_single_chars() {
            let s = " ";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s = "1";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s = "a";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s = "\n";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
        }

        #[test]
        fn valid_unicode_escape() {
            let s = "u0020";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok(' '), r);
            let s = "u0061";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('a'), r);
            let s = "uABCD";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{ABCD}'), r);
            let s = "uD057";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{D057}'), r);
        }

        #[test]
        #[ignore = "TODO: Check that there are no 4-hexdigits invalid unicode code points"]
        fn invalid_unicode_code_point() {
            // TODO: Check that there are no 4-hexdigits invalid unicode code points
        }

        #[test]
        fn unicode_invalid_too_short() {
            let s = "u0";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
            let s = "u0 some garbage after the escape";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s = "u";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
            let s = "uFF";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
        }

        #[test]
        fn unicode_is_exactly_4hexdigits() {
            let s = "u00001";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\0'), r);
            let s = "u000AFE";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{000A}'), r);
        }

        #[test]
        fn update_position() {
            let s = "u0000\"\\bfnrtu0000";
            let mut s = Chars::from(s).peekable();
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\0'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('"'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\\'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{0008}'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\u{000C}'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\n'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\r'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\t'), r);
            let r = read_escape_sequence(&mut s);
            assert_eq!(Ok('\0'), r);
        }
    }
    mod read_string {
        use super::super::read_string;
        use crate::{chars::Chars, err::TokenizeError};

        test_read_x! {read_string}

        #[test]
        fn empty_string() {
            let s = "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("".to_owned()), r);
            let s = "\"should have already stopped";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("".to_owned()), r);
        }

        #[test]
        fn simple_strings() {
            let a = "Hey I'm James, how are you ?".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok(a.clone()), r);
            let a = "I'm quite bored writing tests. &\u{e9}'(-\u{e8}__\u{e7})=$\u{f9}*".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok(a.clone()), r);
        }

        #[test]
        fn u0000_through_u001f_are_invalid() {
            let a = "Hey I'm James, how are you ?\0".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "Hey I'm James,\u{1} how are you ?".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "\u{17}Hey I'm James, how are you ?".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "\tHey I'm James, how are you ?".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            // Other control characters (such as DEL) are valid according to RFC8259
            let a = "\u{7F}Hey I'm James, how are you ?".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok(a.clone()), r);
        }

        #[test]
        fn some_characters_must_escaped() {
            let a = "Hey I'm \"James\", how are you ?".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("Hey I'm ".to_string()), r);
            let a = "Hey I'm \\\"James\\\", how are you ?".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("Hey I'm \"James\", how are you ?".to_string()), r);
            let a = "Hey I'm \\/James/\\, how are you ?".to_string();
            let s = a + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let a = "Hey I'm \\\\/James/\\\\, how are you ?".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("Hey I'm \\/James/\\, how are you ?".to_string()), r);
        }

        #[test]
        fn string_must_be_closed_by_quote() {
            let s = "Hey I'm James, how are you ?".to_string();
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
        }

        #[test]
        fn string_with_escape_sequence() {
            let a = "\\t\\tSome \\\"centered\\\" line\\t\\t\\r\\n".to_string();
            let s = a.clone() + "\"";
            let mut s = Chars::from(s).peekable();
            let r = read_string(&mut s);
            assert_eq!(Ok("\t\tSome \"centered\" line\t\t\r\n".to_string()), r);
        }

        #[test]
        fn string_in_context() {
            let pre = "{\"".to_string();
            let post = "\": 3}";
            let a = "\\t\\tSome \\\"centered\\\" line\\t\\t\\r\\n".to_string();
            let s = pre.clone() + &a + post;
            let mut s = Chars::from(s).peekable();
            // String starts at the second character but our function begins after
            // the quote, so take two items.
            s.next();
            s.next();
            let r = read_string(&mut s);
            assert_eq!(Ok("\t\tSome \"centered\" line\t\t\r\n".to_string()), r);
            let s: Vec<char> = s.map(|r| r.expect("UTF8")).collect();
            let end: Vec<char> = post.chars().skip(1usize).collect();
            assert_eq!(s, end)
        }
    }
    mod read_number {
        use std::vec;

        use super::super::read_number;
        use crate::number::Digit::{D0, D1, D2, D3, D4, D5, D6, D7, D8, D9};
        use crate::number::{Exp, Number, Sign};
        use crate::{chars::Chars, err::TokenizeError};
        #[test]
        fn some_positive_int() {
            let s = "123";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D2, D3],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
            let s = "1789654320";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D7, D8, D9, D6, D5, D4, D3, D2, D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn some_negative_int() {
            let s = "-103";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1, D0, D3],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
            let s = "-1789654320";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1, D7, D8, D9, D6, D5, D4, D3, D2, D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn some_invalid_positive_int() {
            let s = "+123";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "+1789654320";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
        }

        #[test]
        fn leading_zero() {
            let s = "0123";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
            let s = "-01789654320";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn with_fraction() {
            let s = "3.141592653589793";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D3],
                frac: Some(vec![
                    D1, D4, D1, D5, D9, D2, D6, D5, D3, D5, D8, D9, D7, D9, D3,
                ]),
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
            let s = "-0.5";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D0],
                frac: Some(vec![D5]),
                exp: None,
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn with_lower_case_exp() {
            let s = "6022e20";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D6, D0, D2, D2],
                frac: None,
                exp: Some(Exp {
                    s: Sign::Positive,
                    v: vec![D2, D0],
                }),
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn with_upper_case_exp() {
            let s = "1602E-22";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D6, D0, D2],
                frac: None,
                exp: Some(Exp {
                    s: Sign::Negative,
                    v: vec![D2, D2],
                }),
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn sign_frac_exp() {
            let s = "6.022E+22";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D6],
                frac: Some(vec![D0, D2, D2]),
                exp: Some(Exp {
                    s: Sign::Positive,
                    v: vec![D2, D2],
                }),
            };
            assert_eq!(Ok(n), read_number(&mut s));
            let s = "-1.602e-19";
            let mut s = Chars::from(s).peekable();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1],
                frac: Some(vec![D6, D0, D2]),
                exp: Some(Exp {
                    s: Sign::Negative,
                    v: vec![D1, D9],
                }),
            };
            assert_eq!(Ok(n), read_number(&mut s));
        }

        #[test]
        fn invalid_inputs() {
            let s = "d23f";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "NaN";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "Infinity";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "three";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "e";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
            let s = "MDCCLXXXIX";
            let mut s = Chars::from(s).peekable();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&mut s));
        }
    }

    mod read_white_space {
        use crate::chars::Chars;

        use super::super::read_white_space;
        #[test]
        fn read_just_what_needed() {
            let s = "    a";
            let mut s = Chars::from(s).peekable();
            let _ = read_white_space(&mut s);
            let s: Vec<char> = s.map(|r| r.expect("UTF8")).collect();
            assert_eq!(s, vec!['a']);
        }
    }

    #[should_panic]
    #[test]
    fn complete_test() {
        let s = "{\"asd\": { \"sdf\" : [123, 3.14]}}";
        let mut s = Chars::from(s).peekable();
        let mut r = Vec::<Token>::new();
        while let Ok(Some(t)) = read_token(&mut s) {
            r.push(t);
        }
        assert_eq!(Vec::<Token>::new(), r)
    }
}
