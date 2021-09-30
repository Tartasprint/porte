use crate::{
    chars::Chars,
    err::{internal_error, TokenizeError},
    lexer::Lexer,
    token::Token,
    value::Value,
};
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a mut std::iter::Peekable<Chars>) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse(&mut self) -> Result<Value, TokenizeError> {
        let mut state = ParserState::Begin;
        let mut stack: Vec<AST> = Vec::new();
        let mut key_stack: Vec<String> = Vec::new();
        let input = &mut self.lexer;
        while let Some(t) = input.next() {
            match state {
                ParserState::Begin => match t {
                    Token::ArrayBegin => {
                        state = ParserState::InArrayEmpty;
                        stack.push(AST::new_array());
                    }
                    Token::ObjectBegin => {
                        state = ParserState::InObjectEmpty;
                        stack.push(AST::new_object());
                    }
                    Token::False => state = ParserState::End(Value::False),
                    Token::True => state = ParserState::End(Value::True),
                    Token::Null => state = ParserState::End(Value::Null),
                    Token::String(s) => state = ParserState::End(Value::String(s)),
                    Token::Number(n) => state = ParserState::End(Value::Number(n)),
                    Token::WhiteSpace => continue,
                    Token::ArrayEnd
                    | Token::ObjectEnd
                    | Token::NameSeparator
                    | Token::ValueSeparator => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::InArrayEmpty => match t {
                    Token::ArrayBegin => {
                        state = ParserState::InArrayEmpty;
                        stack.push(AST::new_array());
                    }
                    Token::ObjectBegin => {
                        state = ParserState::InObjectEmpty;
                        stack.push(AST::new_object());
                    }
                    Token::False => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::False, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::True => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::True, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::Null => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::Null, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::String(s) => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::String(s), None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::Number(n) => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::Number(n), None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::WhiteSpace => continue,
                    Token::ArrayEnd => array_end(&mut state, &mut stack, &mut key_stack)?,
                    Token::ValueSeparator => return Err(TokenizeError::UnexpectedToken(t)),
                    Token::NameSeparator | Token::ObjectEnd => {
                        return Err(TokenizeError::UnexpectedToken(t))
                    }
                },
                ParserState::InArrayLastWasValue => match t {
                    Token::ValueSeparator => state = ParserState::InArrayLastWasDelim,
                    Token::ArrayEnd => array_end(&mut state, &mut stack, &mut key_stack)?,
                    Token::WhiteSpace => continue,
                    _ => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::InArrayLastWasDelim => match t {
                    Token::ArrayBegin => {
                        state = ParserState::InArrayEmpty;
                        stack.push(AST::new_array());
                    }
                    Token::ObjectBegin => {
                        state = ParserState::InObjectEmpty;
                        stack.push(AST::new_object());
                    }
                    Token::False => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::False, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::True => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::True, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::Null => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::Null, None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::String(s) => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::String(s), None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::Number(n) => {
                        stack
                            .last_mut()
                            .ok_or_else(|| internal_error!())?
                            .push(Value::Number(n), None)?;
                        state = ParserState::InArrayLastWasValue
                    }
                    Token::WhiteSpace => continue,
                    Token::ValueSeparator | Token::ArrayEnd => {
                        return Err(TokenizeError::UnexpectedToken(t))
                    }
                    Token::NameSeparator | Token::ObjectEnd => {
                        return Err(TokenizeError::UnexpectedToken(t))
                    }
                },
                ParserState::InObjectEmpty => match t {
                    Token::String(s) => {
                        key_stack.push(s);
                        state = ParserState::InObjectLastWasKey;
                    }
                    Token::ObjectEnd => object_end(&mut state, &mut stack, &mut key_stack)?,
                    Token::WhiteSpace => continue,
                    _ => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::InObjectLastWasKey => match t {
                    Token::NameSeparator => state = ParserState::InObjectLastWasNameDelim,
                    Token::WhiteSpace => continue,
                    _ => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::InObjectLastWasNameDelim => match t {
                    Token::ArrayBegin => {
                        state = ParserState::InArrayEmpty;
                        stack.push(AST::new_array());
                    }
                    Token::ObjectBegin => {
                        state = ParserState::InObjectEmpty;
                        stack.push(AST::new_object());
                    }
                    Token::False => {
                        stack.last_mut().ok_or_else(|| internal_error!())?.push(
                            Value::False,
                            Some(key_stack.pop().ok_or_else(|| internal_error!())?),
                        )?;
                        state = ParserState::InObjectLastWasValue
                    }
                    Token::True => {
                        stack.last_mut().ok_or_else(|| internal_error!())?.push(
                            Value::True,
                            Some(key_stack.pop().ok_or_else(|| internal_error!())?),
                        )?;
                        state = ParserState::InObjectLastWasValue
                    }
                    Token::Null => {
                        stack.last_mut().ok_or_else(|| internal_error!())?.push(
                            Value::Null,
                            Some(key_stack.pop().ok_or_else(|| internal_error!())?),
                        )?;
                        state = ParserState::InObjectLastWasValue
                    }
                    Token::String(s) => {
                        stack.last_mut().ok_or_else(|| internal_error!())?.push(
                            Value::String(s),
                            Some(key_stack.pop().ok_or_else(|| internal_error!())?),
                        )?;
                        state = ParserState::InObjectLastWasValue
                    }
                    Token::Number(n) => {
                        stack.last_mut().ok_or_else(|| internal_error!())?.push(
                            Value::Number(n),
                            Some(key_stack.pop().ok_or_else(|| internal_error!())?),
                        )?;
                        state = ParserState::InObjectLastWasValue
                    }
                    Token::WhiteSpace => continue,
                    Token::ValueSeparator | Token::ArrayEnd => {
                        return Err(TokenizeError::UnexpectedToken(t))
                    }
                    Token::NameSeparator | Token::ObjectEnd => {
                        return Err(TokenizeError::UnexpectedToken(t))
                    }
                },
                ParserState::InObjectLastWasValue => match t {
                    Token::ValueSeparator => state = ParserState::InObjectLastWasDelim,
                    Token::ObjectEnd => object_end(&mut state, &mut stack, &mut key_stack)?,
                    Token::WhiteSpace => continue,
                    _ => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::InObjectLastWasDelim => match t {
                    Token::String(s) => {
                        key_stack.push(s);
                        state = ParserState::InObjectLastWasKey;
                    }
                    Token::WhiteSpace => continue,
                    _ => return Err(TokenizeError::UnexpectedToken(t)),
                },
                ParserState::End(_) => match t {
                    Token::WhiteSpace => match &(input.status) {
                        Some(Ok(())) => break,
                        Some(Err(e)) => return Err(e.clone()),
                        None => return Err(TokenizeError::InputTooLong),
                    },
                    _ => return Err(TokenizeError::InputTooLong),
                },
            }
        }

        if let ParserState::End(v) = state {
            Ok(v)
        } else {
            Err(TokenizeError::InputEndedEarly)
        }
    }
}

enum AST {
    IncompleteArray(Vec<Value>),
    IncompleteObject(Vec<(String, Value)>),
}

impl AST {
    fn new_array() -> Self {
        Self::IncompleteArray(Vec::new())
    }

    fn new_object() -> Self {
        Self::IncompleteObject(Vec::new())
    }

    /// Push something into the inner incomplete value
    fn push(&mut self, value: Value, key: Option<String>) -> Result<(), TokenizeError> {
        match self {
            Self::IncompleteArray(a) => {
                a.push(value);
                Ok(())
            }
            Self::IncompleteObject(o) => {
                o.push((key.ok_or_else(|| internal_error!())?, value));
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    Begin,
    InArrayEmpty,
    InArrayLastWasDelim,
    InArrayLastWasValue,
    InObjectEmpty,
    InObjectLastWasKey,
    InObjectLastWasValue,
    InObjectLastWasDelim,
    InObjectLastWasNameDelim,
    End(Value),
}

/// Procedure applied when the current parsed array has ended.
///
/// # Panics
///
/// As stated above, the current value being parsed must be an array, that is
/// the last item in the `stack` has to be an `AST::IncompleteArray(_)`, otherwise
/// it will `panic!()`.
/// If the value of the array has to be  put in an object a key will be popped from
/// the `key_stack`, if that isn't possible it will panic.
fn array_end(
    state: &mut ParserState,
    stack: &mut Vec<AST>,
    key_stack: &mut Vec<String>,
) -> Result<(), TokenizeError> {
    if let Some(AST::IncompleteArray(s)) = stack.pop() {
        match stack.last_mut() {
            Some(AST::IncompleteArray(a)) => {
                a.push(Value::Array(s));
                *state = ParserState::InArrayLastWasValue;
            }
            Some(AST::IncompleteObject(o)) => {
                o.push((
                    key_stack.pop().ok_or_else(|| internal_error!())?,
                    Value::Array(s),
                ));
                *state = ParserState::InObjectLastWasValue;
            }
            None => {
                *state = ParserState::End(Value::Array(s));
            }
        };
        Ok(())
    } else {
        Err(internal_error!()) //COV_IGNORE
    }
}

/// Procedure applied when the current parsed object has ended.
///
/// # Panics
///
/// As stated above, the current value being parsed must be an object, that is
/// the last item in the `stack` has to be an `AST::IncompleteObject(_)`, otherwise
/// it will `panic!()`.
/// If the value of the array has to be  put in an object a key will be popped from
/// the `key_stack`, if that isn't possible it will panic.
fn object_end(
    state: &mut ParserState,
    stack: &mut Vec<AST>,
    key_stack: &mut Vec<String>,
) -> Result<(), TokenizeError> {
    if let Some(AST::IncompleteObject(s)) = stack.pop() {
        match stack.last_mut() {
            Some(AST::IncompleteArray(a)) => {
                a.push(Value::Object(s));
                *state = ParserState::InArrayLastWasValue;
            }
            Some(AST::IncompleteObject(o)) => {
                o.push((
                    key_stack.pop().ok_or_else(|| internal_error!())?,
                    Value::Object(s),
                ));
                *state = ParserState::InObjectLastWasValue;
            }
            None => {
                *state = ParserState::End(Value::Object(s));
            }
        };
        Ok(())
    } else {
        Err(internal_error!())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete() {
        let s = r#"
        {
            "accounting" : [
                               { "firstName" : "John",
                                 "lastName"  : "Doe",
                                 "age"       : 23,
                                 "male"      : true},

                               { "firstName" : "Mary",
                                 "lastName"  : "Smith",
                                  "age"      : 32 }
                           ],
            "sales"      : [
                               { "firstName" : "Sally",
                                 "lastName"  : "Green",
                                  "age"      : 27 ,
                                  "male"      : false},

                               { "firstName" : "Jim",
                                 "lastName"  : "Galley",
                                 "age"       : 41 }
                           ]
          } "#;
        let mut s = Chars::from(s).peekable();
        let mut p = Parser::new(&mut s);
        p.parse().unwrap();
    }
}
