//! Common tasks in parsing

use crate::err::TokenizeError;

/// A `Reader` to read one or more times with the same `reader`.
///
/// Returns a `Vec<T>` of each of the results of the sub-reader.
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