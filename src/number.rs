//! A representation for the JSON numbers (that could be arbitrarly large).

use std::convert::TryFrom;

/// Representation of a decimal number.
/// It must have a `sign` and a non-empty `int` part (with no meaningless leading zeros).
/// It can have a `frac`tional part and an exponent.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Number {
    pub(crate) sign: Sign,
    pub(crate) int: Vec<Digit>,
    pub(crate) frac: Option<Vec<Digit>>,
    pub(crate) exp: Option<Exp>,
}

impl Number {
    /// Creates a new Number, from the data.
    ///
    /// # Example
    /// For encoding: 0.314e1
    /// ```ignore
    /// use libporte::number::{*,Digit::*};
    /// let n = Number::new(Sign::Positive,vec![D0],Some(vec![D3,D1,D4]),Some((Sign::Positive,vec![D1])));
    /// ```
    #[must_use]
    pub(crate) fn new(
        sign: Sign,
        int: Vec<Digit>,
        frac: Option<Vec<Digit>>,
        exp: Option<(Sign, Vec<Digit>)>,
    ) -> Self {
        Self {
            sign,
            int,
            frac,
            exp: exp.map(|(s, v)| Exp { s, v }),
        }
    }

    /// Transforms a Number to the canonic scientific notation
    pub fn scientific_notation(&mut self) {
        clear_leading_zeros(&mut self.int);
        let new_exp = &self.exp;
        if self.int[0] == Digit::D0 {

        }
    }
}

fn clear_leading_zeros(v: &mut Vec<Digit>) -> usize {
    match v.iter().position(|d| d != &Digit::D0) {
        Some(0usize) => {0usize},
        Some(first_non_zero) => {
            v.drain(0usize..first_non_zero);
            first_non_zero
        },
        None => {
            let r = v.len() - 1;
            v.clear();
            v.push(Digit::D0);
            r
        }
    }
}

impl TryFrom<Number> for u64 {
    type Error = ();

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        todo!()
    }
}

enum ConversionError {
    NotInteger,
    TooLarge,
    TooPrecise,
}

/// Enum representation of a sign (either positive or negative)
#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(missing_docs)]
pub(crate) enum Sign {
    Positive,
    Negative,
}

/// Representation of decimal digit.
#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(missing_docs)]
pub(crate) enum Digit {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
}

/// Representation of an exponent
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct Exp {
    pub(crate) s: Sign,
    pub(crate) v: Vec<Digit>,
}
