//! Common tasks in parsing

use crate::chars::Chars;

use crate::err::TokenizeError;

/// A `Reader` to read one or more times with the same `reader`.
///
/// Returns a `Vec<T>` of each of the results of the sub-reader.
///
/// # Example
/// To read a string of hexadecimal digits:
/// ```
/// use libporte::lexer::read_hexdigit;
/// use libporte::idioms::read_one_or_more;
///
/// let i : Vec<char> = "deadbeef".chars().collect();
/// assert_eq!(Ok((vec![13,14,10,13,11,14,14,15],8usize)),read_one_or_more(&i,0usize, read_hexdigit));
///```

// pub(crate) fn read_one_or_more<'a,'b: 'a, 'c, 'd: 'c, T>(input: &'a mut std::iter::Peekable<Chars<'b>>, reader: impl Fn(&'a mut std::iter::Peekable<Chars<'b>>) -> ReaderResult<T>) ->
// ReaderResult<Vec<T>> {
//     match reader(input) {
//         Ok(t) => {
//             let mut r = vec![t];
//             while let Ok(t) = reader(input) {
//                 r.push(t);
//             }
//             Ok(r)
//         }
//         Err(e) => Err(e),
//     }
// }

macro_rules! read_one_or_more {
    ($input: expr, $reader: ident) => {{
        match $reader($input) {
            Ok(t) => {
                let mut r = vec![t];
                while let Ok(t) = $reader($input) {
                    r.push(t);
                }
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }};
}

pub(crate) use read_one_or_more;
/// A `Reader` with one more argument. Used to parse a constant string.
///
/// # Example
/// To read the method "POST" in a HTTP request:
/// ```ignore
/// use libporte::idioms::read_string;
/// use libporte::err::TokenizeError;
/// // This is a POST request
/// let valid: Vec<char> = "POST / HTTP/1.1".chars().collect();
/// assert_eq!(Ok(((),4usize)), read_string(&valid, 0usize, "POST"));
/// // This isn't
/// let other:  Vec<char> = "HEAD / HTTP/1.1".chars().collect();
/// assert_eq!(Err(TokenizeError::LitteralDidntMatch), read_string(&other, 0usize, "POST"));
/// ```
pub(crate) fn read_string<'a, 'b: 'a>(
    input: &'a mut std::iter::Peekable<Chars>,
    s: &str,
) -> ReaderResult<()> {
    for sc in s.chars() {
        match input.next() {
            Some(ic) => {
                if ic != sc {
                    return Err(TokenizeError::LitteralDidntMatch);
                }
            }
            None => return Err(TokenizeError::LitteralDidntMatch),
        }
    }
    Ok(())
}

pub(crate) type ReaderResult<T> = Result<T, TokenizeError>;
