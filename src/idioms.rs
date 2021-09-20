//! Common tasks in parsing

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

pub(crate) fn read_one_or_more<T>(input: &[char], mut pos: usize, reader: Reader<T>) -> Result<(Vec<T>, usize), TokenizeError> {
    match reader(input, pos) {
        Ok((t, new_pos)) => {
            pos = new_pos;
            let mut r = vec![t];
            while let Ok((t,new_pos)) = reader(input, pos) {
                pos = new_pos;
                r.push(t);
            }
            Ok((r, pos))
        }
        Err(e) => Err(e),
    }
}


/// A `Reader` with one more argument. Used to parse a constant string.
///
/// # Example
/// To read the method "POST" in a HTTP request:
/// ```
/// use libporte::idioms::read_string;
/// use libporte::err::TokenizeError;
/// // This is a POST request
/// let valid: Vec<char> = "POST / HTTP/1.1".chars().collect();
/// assert_eq!(Ok(((),4usize)), read_string(&valid, 0usize, "POST"));
/// // This isn't
/// let other:  Vec<char> = "HEAD / HTTP/1.1".chars().collect();
/// assert_eq!(Err(TokenizeError::LitteralDidntMatch), read_string(&other, 0usize, "POST"));
/// ```
pub(crate) fn read_string(input: &[char], mut pos: usize, s: &str) -> Result<((),usize),TokenizeError> {
    for c in s.chars() {
        if pos < input.len() && input[pos] == c {
            pos += 1;
        } else {
            return Err(TokenizeError::LitteralDidntMatch);
        }
    };
    Ok(((),pos))
}


/// A `Reader` is a function wich receives an `input` and a the `pos` of the next
/// character to be read. It returns `Ok((t, new_pos))` where `t` is the information
/// processed and `new_pos` the new position (if all the parsing went well, Err(())
/// otherwise).
///
/// # Note
/// An implementation should be resistant to invalid `pos`.
type Reader<T> = fn(input: &[char], pos: usize) -> ReaderResult<T>;

pub(crate) type ReaderResult<T> = Result<(T,usize), TokenizeError>;