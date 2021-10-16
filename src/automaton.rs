use std::iter::Peekable;

use crate::{
    chars::Chars,
    err::{internal_error, TokenizeError},
    lexer::Lexer,
    token::Token,
    value::Value,
};

/// A stack-based automaton to read a stream of Tokens.
pub struct Automaton<'a> {
    lexer: Lexer<'a>,
    state: State,
    stack: Vec<Stack>,
    keys: usize,
}

impl<'a> Automaton<'a> {
    /// Create a new JSON Automaton
    pub fn new(input: &'a mut Peekable<Chars>) -> Self {
        Self {
            lexer: Lexer::new(input),
            state: State::Begin,
            stack: Vec::new(),
            keys: 0,
        }
    }

    fn array_end(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(Stack::Array) = self.stack.pop() {
            match self.stack.last() {
                Some(Stack::Array) => {
                    self.state = State::LastWasValueIn(Array);
                }
                Some(Stack::Object) => {
                    self.state = State::LastWasValueIn(Object);
                    self.keys -= 1;
                }
                None => {
                    self.state = State::End;
                }
            };
            Some(Ok(Action::Close))
        } else {
            self.state = State::Ended;
            Some(Err(internal_error!())) //COV_IGNORE
        }
    }

    fn object_end(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(Stack::Object) = self.stack.pop() {
            match self.stack.last() {
                Some(Stack::Array) => {
                    self.state = State::LastWasValueIn(Array);
                }
                Some(Stack::Object) => {
                    self.state = State::LastWasValueIn(Object);
                    self.keys -= 1;
                }
                None => {
                    self.state = State::End;
                }
            };
            Some(Ok(Action::Close))
        } else {
            self.state = State::Ended;
            Some(Err(internal_error!())) //COV_IGNORE
        }
    }
}

macro_rules! debug_state {
    ($token:expr,$state:expr) => {
        if (cfg!(debug_assertions)) {
            println!("{:<30}{:?}", format!("{:?}", &$token), &$state);
        }
    };
}

impl Iterator for Automaton<'_> {
    type Item = Result<Action, TokenizeError>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.lexer.next() {
            debug_state!(t, self.state);
            match self.state {
                State::Begin => match t {
                    Token::ArrayBegin => {
                        self.state = State::InArrayEmpty;
                        self.stack.push(Stack::Array);
                        Some(Ok(Action::NewArray))
                    }
                    Token::ObjectBegin => {
                        self.state = State::InObjectEmpty;
                        self.stack.push(Stack::Object);
                        Some(Ok(Action::NewObject))
                    }
                    Token::False => {
                        self.state = State::End;
                        Some(Ok(Action::Push(Value::False)))
                    }
                    Token::True => {
                        self.state = State::End;
                        Some(Ok(Action::Push(Value::True)))
                    }
                    Token::Null => {
                        self.state = State::End;
                        Some(Ok(Action::Push(Value::Null)))
                    }
                    Token::String(s) => {
                        self.state = State::End;
                        Some(Ok(Action::Push(Value::String(s))))
                    }
                    Token::Number(n) => {
                        self.state = State::End;
                        Some(Ok(Action::Push(Value::Number(n))))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    Token::ArrayEnd
                    | Token::ObjectEnd
                    | Token::NameSeparator
                    | Token::ValueSeparator => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InArrayEmpty => match t {
                    Token::ArrayBegin => {
                        self.state = State::InArrayEmpty;
                        self.stack.push(Stack::Array);
                        Some(Ok(Action::NewArray))
                    }
                    Token::ObjectBegin => {
                        self.state = State::InObjectEmpty;
                        self.stack.push(Stack::Object);
                        Some(Ok(Action::NewObject))
                    }
                    Token::False => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::False)))
                    }
                    Token::True => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::True)))
                    }
                    Token::Null => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::Null)))
                    }
                    Token::String(s) => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::String(s))))
                    }
                    Token::Number(n) => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::Number(n))))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    Token::ArrayEnd => self.array_end(),
                    Token::ValueSeparator => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                    Token::NameSeparator | Token::ObjectEnd => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::LastWasValueIn(Array) => match t {
                    Token::ValueSeparator => {
                        self.state = State::InArrayLastWasDelim;
                        Some(Ok(Action::Nothing))
                    }
                    Token::ArrayEnd => self.array_end(),
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    _ => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InArrayLastWasDelim => match t {
                    Token::ArrayBegin => {
                        self.state = State::InArrayEmpty;
                        self.stack.push(Stack::Array);
                        Some(Ok(Action::NewArray))
                    }
                    Token::ObjectBegin => {
                        self.state = State::InObjectEmpty;
                        self.stack.push(Stack::Object);
                        Some(Ok(Action::NewObject))
                    }
                    Token::False => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::False)))
                    }
                    Token::True => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::True)))
                    }
                    Token::Null => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::Null)))
                    }
                    Token::String(s) => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::String(s))))
                    }
                    Token::Number(n) => {
                        self.state = State::LastWasValueIn(Array);
                        Some(Ok(Action::Push(Value::Number(n))))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    Token::ValueSeparator | Token::ArrayEnd => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                    Token::NameSeparator | Token::ObjectEnd => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InObjectEmpty => match t {
                    Token::String(s) => {
                        self.keys += 1;
                        self.state = State::InObjectLastWasKey;
                        Some(Ok(Action::NewKey(s)))
                    }
                    Token::ObjectEnd => self.object_end(),
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    _ => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InObjectLastWasKey => match t {
                    Token::NameSeparator => {
                        self.state = State::InObjectLastWasNameDelim;
                        Some(Ok(Action::Nothing))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    _ => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InObjectLastWasNameDelim => match t {
                    Token::ArrayBegin => {
                        self.state = State::InArrayEmpty;
                        self.stack.push(Stack::Array);
                        Some(Ok(Action::NewArray))
                    }
                    Token::ObjectBegin => {
                        self.state = State::InObjectEmpty;
                        self.stack.push(Stack::Object);
                        Some(Ok(Action::NewObject))
                    }
                    Token::False => {
                        self.state = State::LastWasValueIn(Object);
                        self.keys -= 1;
                        Some(Ok(Action::Push(Value::False)))
                    }
                    Token::True => {
                        self.state = State::LastWasValueIn(Object);
                        self.keys -= 1;
                        Some(Ok(Action::Push(Value::True)))
                    }
                    Token::Null => {
                        self.state = State::LastWasValueIn(Object);
                        self.keys -= 1;
                        Some(Ok(Action::Push(Value::Null)))
                    }
                    Token::String(s) => {
                        self.state = State::LastWasValueIn(Object);
                        self.keys -= 1;
                        Some(Ok(Action::Push(Value::String(s))))
                    }
                    Token::Number(n) => {
                        self.state = State::LastWasValueIn(Object);
                        self.keys -= 1;
                        Some(Ok(Action::Push(Value::Number(n))))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),

                    Token::ValueSeparator | Token::ArrayEnd => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                    Token::NameSeparator | Token::ObjectEnd => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::LastWasValueIn(Object) => match t {
                    Token::ValueSeparator => {
                        self.state = State::InObjectLastWasDelim;
                        Some(Ok(Action::Nothing))
                    }
                    Token::ObjectEnd => self.object_end(),
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    _ => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::InObjectLastWasDelim => match t {
                    Token::String(s) => {
                        self.keys += 1;
                        self.state = State::InObjectLastWasKey;
                        Some(Ok(Action::NewKey(s)))
                    }
                    Token::WhiteSpace => Some(Ok(Action::Nothing)),
                    _ => {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::UnexpectedToken(t)))
                    }
                },
                State::End => {
                    self.state = State::Ended;
                    match t {
                        Token::WhiteSpace => match &self.lexer.status {
                            Some(Ok(())) => Some(Ok(Action::TheEnd)),
                            Some(Err(e)) => Some(Err(e.clone())),
                            None => {
                                if self.lexer.next().is_none() {
                                    Some(Ok(Action::TheEnd))
                                } else {
                                    Some(Err(TokenizeError::InputTooLong))
                                }
                            }
                        },
                        _ => {
                            self.state = State::Ended;
                            Some(Err(TokenizeError::InputTooLong))
                        }
                    }
                }
                State::Ended => None,
            }
        } else {
            match self.state {
                State::End => {
                    if matches!(self.lexer.status, Some(Ok(())))
                        && self.keys == 0
                        && self.stack.is_empty()
                    {
                        self.state = State::Ended;
                        Some(Ok(Action::TheEnd))
                    } else {
                        self.state = State::Ended;
                        Some(Err(TokenizeError::InputTooLong))
                    }
                }
                State::Ended => None,
                _ => {
                    self.state = State::Ended;
                    Some(Err(TokenizeError::InputTooLong))
                }
            }
        }
    }
}

#[derive(Debug)]
/// Which action should a parser do at each step of the automaton
pub enum Action {
    /// Nothing to be done
    Nothing,
    /// A new array is to be created
    NewArray,
    /// A new object is to be created
    NewObject,
    /// A new key was read
    NewKey(String),
    /// A value is to be pushed to the last array/object (if none it's because the Value is the value of the whole JSON document)
    Push(Value),
    /// The last array/object ended and is to be pushed
    Close,
    /// The parsing ended
    TheEnd,
}

#[derive(Debug)]
enum Stack {
    Array,
    Object,
}
use Stack::*;

#[derive(Debug)]
enum State {
    Begin,
    InArrayEmpty,
    InArrayLastWasDelim,
    InObjectEmpty,
    InObjectLastWasKey,
    LastWasValueIn(Stack),
    InObjectLastWasDelim,
    InObjectLastWasNameDelim,
    End,
    Ended,
}
