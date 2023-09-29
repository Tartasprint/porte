use crate::number::Number;
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Number(Number),
    String(String),
    True,
    False,
    Null,
}


