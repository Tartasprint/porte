//! Functions to tokenize the text.

use crate::{err::TokenizeError, idioms_slice::{self, read_one_or_more}, lexer::Lexer, number::{Digit, Number, Sign}, token::Token};


pub struct LexerSlice<'a> {
    input: &'a[char],
    pos: usize,
    status: Option<Result<(), TokenizeError>>,
}

impl<'a> LexerSlice<'a> {
    pub fn new(input: &'a[char]) -> Self {
        Self {
            input,
            pos: 0,
            status: None,
        }
    }
}

impl Lexer for LexerSlice<'_> {
    fn report(&self) -> &Option<Result<(),TokenizeError>> {
        &self.status
    }
}


impl Iterator for LexerSlice<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.status.is_none() {
            match read_token(self.input,self.pos) {
                Ok((Some(s),new_pos)) => {self.pos = new_pos;Some(s)},
                Ok((None,new_pos)) => {
                    self.status = Some(Ok(()));
                    self.pos = new_pos;
                    None
                }
                Err(e) => {
                    self.status = Some(Err(e));
                    None
                }
            }
        } else {
            None
        }
    }
}

type ReaderResult<T> = Result<(T,usize),TokenizeError>;

const HI_SURROGATE_MIN: u32 = 0xD800;
const HI_SURROGATE_MAX: u32 = 0xDBFF;
const LO_SURROGATE_MIN: u32 = 0xDC00;
const LO_SURROGATE_MAX: u32 = 0xDFFF;

use std::ops::ShlAssign;
/// Reads a JSON token.
pub(crate) fn read_token(input: &[char], pos: usize) -> ReaderResult<Option<Token>> {
    if pos < input.len() {
        match input[pos] {
            '0'..='9' | '-' => match read_number(input, pos) {
                Ok((n, new_pos)) => Ok((Some(Token::Number(n)), new_pos)),
                Err(e) => Err(e),
            },
            '"' => match read_string(input, pos + 1) {
                Ok((s, new_pos)) => Ok((Some(Token::String(s)), new_pos)),
                Err(e) => Err(e),
            },
            '[' => Ok((Some(Token::ArrayBegin), pos + 1)),
            ']' => Ok((Some(Token::ArrayEnd), pos + 1)),
            '{' => Ok((Some(Token::ObjectBegin), pos + 1)),
            '}' => Ok((Some(Token::ObjectEnd), pos + 1)),
            't' => {
                let (_, new_pos) = read_rue(input, pos + 1)?;
                Ok((Some(Token::True), new_pos))
            }
            'f' => {
                let (_, new_pos) = read_alse(input, pos + 1)?;
                Ok((Some(Token::False), new_pos))
            }
            'n' => {
                let (_, new_pos) = read_ull(input, pos + 1)?;
                Ok((Some(Token::Null), new_pos))
            }
            ',' => Ok((Some(Token::ValueSeparator), pos + 1)),
            ':' => Ok((Some(Token::NameSeparator), pos + 1)),
            ' ' | '\t' | '\x0A' | '\x0D' => {
                let (_, new_pos) = read_white_space(input, pos)?;
                Ok((Some(Token::WhiteSpace), new_pos))
            }
            _ => Err(TokenizeError::UnkownToken),
        }
    } else {
        Ok((None, pos))
    }
}

/// Reads a RFC 8259 JSON number;
fn read_number(input: &[char], pos: usize) -> ReaderResult<Number> {
    let (sign, pos) = read_neg_sign(input, pos)?;
    let (int, pos) = read_int(input, pos)?;
    let (frac, pos) = read_frac(input, pos)?;
    let (exp, new_pos) = read_exp(input, pos)?;
    Ok((Number::new(sign, int, frac, exp), new_pos))
}

/// Reads a decimal digit.
fn read_digit(input: &[char], pos: usize) -> ReaderResult<Digit> {
    if pos < input.len() {
        match input[pos] {
            '0' => Ok((Digit::D0, pos + 1)),
            '1' => Ok((Digit::D1, pos + 1)),
            '2' => Ok((Digit::D2, pos + 1)),
            '3' => Ok((Digit::D3, pos + 1)),
            '4' => Ok((Digit::D4, pos + 1)),
            '5' => Ok((Digit::D5, pos + 1)),
            '6' => Ok((Digit::D6, pos + 1)),
            '7' => Ok((Digit::D7, pos + 1)),
            '8' => Ok((Digit::D8, pos + 1)),
            '9' => Ok((Digit::D9, pos + 1)),
            _ => Err(TokenizeError::ExpectedADigit),
        }
    } else {
        Err(TokenizeError::InputEndedEarly)
    }
}

/// Reads RFC 8259 JSON white space.
fn read_white_space(input: &[char], mut pos: usize) -> ReaderResult<()> {
    while pos < input.len() {
        match input[pos] {
            ' ' | '\t' | '\x0A' | '\x0D' => pos += 1,
            _ => break,
        }
    }
    Ok(((), pos))
}

/// Reads the int part of a RFC 8259 JSON number.
fn read_int(input: &[char], mut pos: usize) -> ReaderResult<Vec<Digit>>{
    match read_digit(input, pos) {
        Ok((Digit::D0, new_pos)) => Ok((vec![Digit::D0], new_pos)),
        Ok((d, new_pos)) => {
            pos = new_pos;
            let mut r = vec![d];
            while let Ok((d, new_pos)) = read_digit(input, pos) {
                pos = new_pos;
                r.push(d);
            }
            Ok((r, pos))
        }
        Err(e) => Err(e),
    }
}

/// Reads the fractional part of a RFC 8259 JSON number.
fn read_frac(input: &[char], mut pos: usize) -> ReaderResult<Option<Vec<Digit>>> {
    if pos < input.len() && input[pos] == '.' {
        pos += 1;
        let (frac, new_pos) = read_one_or_more(input, pos, read_digit)?;
        Ok((Some(frac), new_pos))
    } else {
        Ok((None, pos))
    }
}

/// Reads the exponential part of a RFC 8259 JSON number.
fn read_exp(
    input: &[char],
    mut pos: usize,
) -> ReaderResult<Option<(Sign, Vec<Digit>)>> {
    if pos < input.len() && (input[pos] == 'e' || input[pos] == 'E') {
        pos += 1;
        let (sign, new_pos) = read_pn_sign(input, pos)?;
        pos = new_pos;
        let (exp, new_pos) = read_one_or_more(input, pos, read_digit)?;
        Ok((Some((sign, exp)), new_pos))
    } else {
        Ok((None, pos))
    }
}

/// Reads an optional negative sign (`'-'`).
fn read_neg_sign(input: &[char], pos: usize) -> ReaderResult<Sign>{
    if pos < input.len() && input[pos] == '-' {
        Ok((Sign::Negative, pos + 1))
    } else {
        Ok((Sign::Positive, pos))
    }
}

/// Reads an optional positive sign (`'+'`) or a negative sign (`'-'`).
fn read_pn_sign(input: &[char], pos: usize) -> ReaderResult<Sign> {
    if pos < input.len() {
        if input[pos] == '-' {
            Ok((Sign::Negative, pos + 1))
        } else if input[pos] == '+' {
            Ok((Sign::Positive, pos + 1))
        } else {
            Ok((Sign::Positive, pos))
        }
    } else {
        Ok((Sign::Positive, pos))
    }
}

/// Reads RFC8259 JSON string.
fn read_string(input: &[char], mut pos: usize) -> ReaderResult<String> {
    let mut a = String::new();
    while pos < input.len() {
        match input[pos] {
            '\u{0000}'..='\u{001F}' => return Err(TokenizeError::ControlCharacterUnescaped),
            '\\' => match read_escape_sequence(input, pos + 1) {
                Ok((r, new_pos)) => {
                    a.push(r);
                    pos = new_pos;
                }
                Err(e) => return Err(e),
            },
            '"' => return Ok((a, pos + 1)),
            _ => {
                a.push(input[pos]);
                pos += 1;
            }
        }
    }
    Err(TokenizeError::InputEndedEarly)
}

/// Reads a "rue" (ending of "true").
fn read_rue(input: &[char], pos: usize) -> ReaderResult<()> {
    idioms_slice::read_string(input, pos, "rue")
}

/// Reads a "false" (ending of "false").
fn read_alse(input: &[char], pos: usize) -> ReaderResult<()> {
    idioms_slice::read_string(input, pos, "alse")
}

/// Reads a "ull" (ending of "null").
fn read_ull(input: &[char], pos: usize) -> ReaderResult<()> {
    idioms_slice::read_string(input, pos, "ull")
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
fn read_escape_sequence(input: &[char], mut pos: usize) -> ReaderResult<char> {
    if pos < input.len() {
        match input[pos] {
            '"' => Ok(('"', pos + 1)),
            '\\' => Ok(('\\', pos + 1)),
            '/' => Ok(('/', pos + 1)),
            'b' => Ok(('\u{0008}', pos + 1)),
            'f' => Ok(('\u{000C}', pos + 1)),
            'n' => Ok(('\n', pos + 1)),
            'r' => Ok(('\r', pos + 1)),
            't' => Ok(('\t', pos + 1)),
            'u' => {
                let mut a: u32 = 0;
                pos += 1;
                for _ in 0..4 {
                    match read_hexdigit(input, pos) {
                        Ok((r, new_pos)) => {
                            pos = new_pos;
                            a.shl_assign(4);
                            a += u32::from(r);
                        }
                        Err(e) => return Err(e),
                    };
                }
                if HI_SURROGATE_MIN <= a && a <= HI_SURROGATE_MAX {
                    let ((), new_pos) = idioms_slice::read_string(input, pos, "\\u")?;
                    pos = new_pos;
                    let mut b: u32 = 0;
                    //pos += 1;
                    for _ in 0..4 {
                        match read_hexdigit(input, pos) {
                            Ok((r, new_pos)) => {
                                crate::err::debug!("Surrogating");
                                pos = new_pos;
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
                            Some(c) => Ok((c, pos)),
                            None => { Err(TokenizeError::InvalidSurrogatePairs)},
                        }
                    } else {
                        Err(TokenizeError::BigMessWithSurrogatePairs)
                    }
                } else {
                    match std::char::from_u32(a) {
                        Some(c) => Ok((c, pos)),
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
fn read_hexdigit(input: &[char], pos: usize) -> ReaderResult<u8> {
    if pos < input.len() {
        match input[pos] {
            '0'..='9' => Ok((input[pos] as u8 - b'0', pos + 1)),
            'a'..='f' => Ok((input[pos] as u8 - b'a' + 10_u8, pos + 1)),
            'A'..='F' => Ok((input[pos] as u8 - b'A' + 10_u8, pos + 1)),
            _ => Err(TokenizeError::ExpectedAHexdigit),
        }
    } else {
        Err(TokenizeError::InputEndedEarly)
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
                    use crate::err::TokenizeError;
                    #[test]
                    fn empty_input(){
                        let s: Vec<char> = "".chars().collect();
                        let r = $func_name(&s, 0_usize);
                        assert_eq!(Err(TokenizeError::InputEndedEarly), r);
                        let s: Vec<char> = "".chars().collect();
                        let r = $func_name(&s, 1_usize);
                        assert_eq!(Err(TokenizeError::InputEndedEarly), r);
                        let s: Vec<char> = "01234".chars().collect();
                        let r = $func_name(&s, 5_usize);
                        assert_eq!(Err(TokenizeError::InputEndedEarly), r);
                    }
                }
            )*
        };
    }

    mod read_hexdigit {
        use crate::err::TokenizeError;

        use super::read_hexdigit;

        test_read_x! {read_hexdigit}

        #[test]
        fn digits() {
            let s: Vec<char> = "0".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((0_u8, 1)), r);
            let s: Vec<char> = "1".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((1_u8, 1)), r);
            let s: Vec<char> = "2".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((2_u8, 1)), r);
            let s: Vec<char> = "3".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((3_u8, 1)), r);
            let s: Vec<char> = "4".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((4_u8, 1)), r);
            let s: Vec<char> = "5".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((5_u8, 1)), r);
            let s: Vec<char> = "6".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((6_u8, 1)), r);
            let s: Vec<char> = "7".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((7_u8, 1)), r);
            let s: Vec<char> = "8".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((8_u8, 1)), r);
            let s: Vec<char> = "9".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((9_u8, 1)), r);
        }

        #[test]
        fn lower_case() {
            let s: Vec<char> = "a".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((10_u8, 1)), r);
            let s: Vec<char> = "b".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((11_u8, 1)), r);
            let s: Vec<char> = "c".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((12_u8, 1)), r);
            let s: Vec<char> = "d".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((13_u8, 1)), r);
            let s: Vec<char> = "e".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((14_u8, 1)), r);
            let s: Vec<char> = "f".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((15_u8, 1)), r);
        }

        #[test]
        fn upper_case() {
            let s: Vec<char> = "A".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((10_u8, 1)), r);
            let s: Vec<char> = "B".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((11_u8, 1)), r);
            let s: Vec<char> = "C".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((12_u8, 1)), r);
            let s: Vec<char> = "D".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((13_u8, 1)), r);
            let s: Vec<char> = "E".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((14_u8, 1)), r);
            let s: Vec<char> = "F".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((15_u8, 1)), r);
        }

        #[test]
        fn invalid() {
            let s: Vec<char> = "z".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "@".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "One".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "|ab".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "\0".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "\t".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
        }

        #[test]
        fn update_position() {
            let s: Vec<char> = "0123456789aBcDeF".chars().collect();
            let r = read_hexdigit(&s, 0_usize);
            assert_eq!(Ok((0_u8, 1_usize)), r);
            let r = read_hexdigit(&s, 1_usize);
            assert_eq!(Ok((1_u8, 2_usize)), r);
            let r = read_hexdigit(&s, 2_usize);
            assert_eq!(Ok((2_u8, 3_usize)), r);
            let r = read_hexdigit(&s, 3_usize);
            assert_eq!(Ok((3_u8, 4_usize)), r);
            let r = read_hexdigit(&s, 4_usize);
            assert_eq!(Ok((4_u8, 5_usize)), r);
            let r = read_hexdigit(&s, 5_usize);
            assert_eq!(Ok((5_u8, 6_usize)), r);
            let r = read_hexdigit(&s, 6_usize);
            assert_eq!(Ok((6_u8, 7_usize)), r);
            let r = read_hexdigit(&s, 7_usize);
            assert_eq!(Ok((7_u8, 8_usize)), r);
            let r = read_hexdigit(&s, 8_usize);
            assert_eq!(Ok((8_u8, 9_usize)), r);
            let r = read_hexdigit(&s, 9_usize);
            assert_eq!(Ok((9_u8, 10_usize)), r);
            let r = read_hexdigit(&s, 10_usize);
            assert_eq!(Ok((10_u8, 11_usize)), r);
            let r = read_hexdigit(&s, 11_usize);
            assert_eq!(Ok((11_u8, 12_usize)), r);
            let r = read_hexdigit(&s, 12_usize);
            assert_eq!(Ok((12_u8, 13_usize)), r);
            let r = read_hexdigit(&s, 13_usize);
            assert_eq!(Ok((13_u8, 14_usize)), r);
            let r = read_hexdigit(&s, 14_usize);
            assert_eq!(Ok((14_u8, 15_usize)), r);
            let r = read_hexdigit(&s, 15_usize);
            assert_eq!(Ok((15_u8, 16_usize)), r);
        }
    }

    mod read_escape_sequence {
        use super::read_escape_sequence;
        use crate::err::TokenizeError;
        test_read_x! {read_escape_sequence}

        #[test]
        fn valid_single_chars() {
            let s: Vec<char> = "\"".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('"', 1_usize)), r);
            let s: Vec<char> = "\\".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\\', 1_usize)), r);
            let s: Vec<char> = "/".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('/', 1_usize)), r);
            let s: Vec<char> = "b".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\u{0008}', 1_usize)), r);
            let s: Vec<char> = "f".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\u{000C}', 1_usize)), r);
            let s: Vec<char> = "n".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\n', 1_usize)), r);
            let s: Vec<char> = "r".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\r', 1_usize)), r);
            let s: Vec<char> = "t".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\t', 1_usize)), r);
        }

        #[test]
        fn invalid_single_chars() {
            let s: Vec<char> = " ".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s: Vec<char> = "1".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s: Vec<char> = "a".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let s: Vec<char> = "\n".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
        }

        #[test]
        fn valid_unicode_escape() {
            let s: Vec<char> = "u0020".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok((' ', s.len())), r);
            let s: Vec<char> = "u0061".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('a', s.len())), r);
            let s: Vec<char> = "uABCD".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\u{ABCD}', s.len())), r);
            let s: Vec<char> = "uD057".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\u{D057}', s.len())), r);
        }

        #[test]
        #[ignore = "TODO: Check that there are no 4-hexdigits invalid unicode code points"]
        fn invalid_unicode_code_point() {
            // TODO: Check that there are no 4-hexdigits invalid unicode code points
        }

        #[test]
        fn unicode_invalid_too_short() {
            let s: Vec<char> = "u0".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
            let s: Vec<char> = "u0 some garbage after the escape".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ExpectedAHexdigit), r);
            let s: Vec<char> = "u".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
            let s: Vec<char> = "uFF".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
        }

        #[test]
        fn unicode_is_exactly_4hexdigits() {
            let s: Vec<char> = "u00001".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\0', 5_usize)), r);
            let s: Vec<char> = "u000AFE".chars().collect();
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\u{000A}', 5_usize)), r);
        }

        #[test]
        fn update_position() {
            let s: Vec<char> = "u0000\"\\bfnrtu0000".chars().collect();
            let r = read_escape_sequence(&s, 18_usize);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
            let r = read_escape_sequence(&s, 0_usize);
            assert_eq!(Ok(('\0', 5_usize)), r);
            let r = read_escape_sequence(&s, 5_usize);
            assert_eq!(Ok(('"', 6_usize)), r);
            let r = read_escape_sequence(&s, 6_usize);
            assert_eq!(Ok(('\\', 7_usize)), r);
            let r = read_escape_sequence(&s, 7_usize);
            assert_eq!(Ok(('\u{0008}', 8_usize)), r);
            let r = read_escape_sequence(&s, 8_usize);
            assert_eq!(Ok(('\u{000C}', 9_usize)), r);
            let r = read_escape_sequence(&s, 9_usize);
            assert_eq!(Ok(('\n', 10_usize)), r);
            let r = read_escape_sequence(&s, 10_usize);
            assert_eq!(Ok(('\r', 11_usize)), r);
            let r = read_escape_sequence(&s, 11_usize);
            assert_eq!(Ok(('\t', 12_usize)), r);
            let r = read_escape_sequence(&s, 12_usize);
            assert_eq!(Ok(('\0', 17_usize)), r);
        }
    }

    mod read_string {
        use super::read_string;
        use crate::err::TokenizeError;

        test_read_x! {read_string}

        #[test]
        fn empty_string() {
            let s: Vec<char> = "\"".chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok(("".to_owned(), 1_usize)), r);
            let s: Vec<char> = "\"should have already stopped".chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok(("".to_owned(), 1_usize)), r);
        }

        #[test]
        fn simple_strings() {
            let a = "Hey I'm James, how are you ?".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok((a.clone(), a.chars().count() + 1_usize)), r);
            let a = "I'm quite bored writing tests. &\u{e9}'(-\u{e8}__\u{e7})=$\u{f9}*".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok((a.clone(), a.chars().count() + 1_usize)), r);
        }

        #[test]
        fn u0000_through_u001f_are_invalid() {
            let a = "Hey I'm James, how are you ?\0".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "Hey I'm James,\u{1} how are you ?".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "\u{17}Hey I'm James, how are you ?".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            let a = "\tHey I'm James, how are you ?".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::ControlCharacterUnescaped), r);
            // Other control characters (such as DEL) are valid according to RFC8259
            let a = "\u{7F}Hey I'm James, how are you ?".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok((a.clone(), a.chars().count() + 1_usize)), r);
        }

        #[test]
        fn some_characters_must_escaped() {
            let a = "Hey I'm \"James\", how are you ?".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Ok(("Hey I'm ".to_string(), 9_usize)), r);
            let a = "Hey I'm \\\"James\\\", how are you ?".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(
                Ok((
                    "Hey I'm \"James\", how are you ?".to_string(),
                    a.chars().count() + 1_usize
                )),
                r
            );
            let a = "Hey I'm \\/James/\\, how are you ?".to_string();
            let s: Vec<char> = (a + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::UnkownEscapeSequence), r);
            let a = "Hey I'm \\\\/James/\\\\, how are you ?".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(
                Ok((
                    "Hey I'm \\/James/\\, how are you ?".to_string(),
                    a.chars().count() + 1_usize
                )),
                r
            );
        }

        #[test]
        fn string_must_be_closed_by_quote() {
            let a = "Hey I'm James, how are you ?".to_string();
            let s: Vec<char> = a.chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(Err(TokenizeError::InputEndedEarly), r);
        }

        #[test]
        fn string_with_escape_sequence() {
            let a = "\\t\\tSome \\\"centered\\\" line\\t\\t\\r\\n".to_string();
            let s: Vec<char> = (a.clone() + "\"").chars().collect();
            let r = read_string(&s, 0_usize);
            assert_eq!(
                Ok((
                    "\t\tSome \"centered\" line\t\t\r\n".to_string(),
                    a.chars().count() + 1_usize
                )),
                r
            );
        }

        #[test]
        fn string_in_context() {
            let pre = "{\"".to_string();
            let post = "\": 3}";
            let a = "\\t\\tSome \\\"centered\\\" line\\t\\t\\r\\n".to_string();
            let s: Vec<char> = (pre.clone() + &a + post).chars().collect();
            let r = read_string(&s, pre.chars().count());
            assert_eq!(
                Ok((
                    "\t\tSome \"centered\" line\t\t\r\n".to_string(),
                    pre.chars().count() + a.chars().count() + 1_usize
                )),
                r
            );
        }
    }

    mod read_number {
        use std::vec;

        use super::read_number;
        use crate::err::TokenizeError;
        use crate::number::Digit::{D0, D1, D2, D3, D4, D5, D6, D7, D8, D9};
        use crate::number::{Exp, Number, Sign};
        #[test]
        fn some_positive_int() {
            let a: Vec<char> = "123".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D2, D3],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 3_usize)), read_number(&a, 0_usize));
            let a: Vec<char> = "1789654320".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D7, D8, D9, D6, D5, D4, D3, D2, D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 10_usize)), read_number(&a, 0_usize));
        }

        #[test]
        fn some_negative_int() {
            let a: Vec<char> = "-103".chars().collect();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1, D0, D3],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 4_usize)), read_number(&a, 0_usize));
            let a: Vec<char> = "-1789654320".chars().collect();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1, D7, D8, D9, D6, D5, D4, D3, D2, D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 11_usize)), read_number(&a, 0_usize));
        }

        #[test]
        fn some_invalid_positive_int() {
            let a: Vec<char> = "+123".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "+1789654320".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
        }

        #[test]
        fn leading_zero() {
            let a: Vec<char> = "0123".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 1_usize)), read_number(&a, 0_usize));
            let a: Vec<char> = "-01789654320".chars().collect();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D0],
                frac: None,
                exp: None,
            };
            assert_eq!(Ok((n, 2_usize)), read_number(&a, 0_usize));
        }

        #[test]
        fn with_fraction() {
            let a: Vec<char> = "3.141592653589793".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D3],
                frac: Some(vec![
                    D1, D4, D1, D5, D9, D2, D6, D5, D3, D5, D8, D9, D7, D9, D3,
                ]),
                exp: None,
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
            let a: Vec<char> = "-0.5".chars().collect();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D0],
                frac: Some(vec![D5]),
                exp: None,
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
        }

        #[test]
        fn with_lower_case_exp() {
            let a: Vec<char> = "6022E20".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D6, D0, D2, D2],
                frac: None,
                exp: Some(Exp {
                    s: Sign::Positive,
                    v: vec![D2, D0],
                }),
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
        }

        #[test]
        fn with_upper_case_exp() {
            let a: Vec<char> = "1602e-22".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D1, D6, D0, D2],
                frac: None,
                exp: Some(Exp {
                    s: Sign::Negative,
                    v: vec![D2, D2],
                }),
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
        }

        #[test]
        fn sign_frac_exp() {
            let a: Vec<char> = "6.022E+22".chars().collect();
            let n = Number {
                sign: Sign::Positive,
                int: vec![D6],
                frac: Some(vec![D0, D2, D2]),
                exp: Some(Exp {
                    s: Sign::Positive,
                    v: vec![D2, D2],
                }),
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
            let a: Vec<char> = "-1.602e-19".chars().collect();
            let n = Number {
                sign: Sign::Negative,
                int: vec![D1],
                frac: Some(vec![D6, D0, D2]),
                exp: Some(Exp {
                    s: Sign::Negative,
                    v: vec![D1, D9],
                }),
            };
            assert_eq!(Ok((n, a.len())), read_number(&a, 0_usize));
        }

        #[test]
        fn invalid_inputs() {
            let a: Vec<char> = "d23f".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "NaN".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "Infinity".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "three".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "e".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
            let a: Vec<char> = "MDCCLXXXIX".chars().collect();
            assert_eq!(Err(TokenizeError::ExpectedADigit), read_number(&a, 0_usize));
        }
    }

    #[should_panic]
    #[test]
    fn complete_test() {
        let s: Vec<char> = "{\"asd\": { \"sdf\" : [123, 3.14]}}".chars().collect();
        let mut pos = 0_usize;
        let mut r = Vec::<Token>::new();
        while let Ok((Some(t), new_pos)) = read_token(&s, pos) {
            pos = new_pos;
            r.push(t);
        }

        assert_eq!(Vec::<Token>::new(), r)
    }
}
