use crate::{automaton::{Action, Automaton}, err::{TokenizeError, internal_error}, lexer::Lexer, value::Value};

/// Parse the input (and check that it's valid as whole)
pub fn parse_and_valid<'a>(input: Box<dyn Lexer + 'a>) -> Result<Value, TokenizeError> {
    let automaton = Automaton::new(input);
    let mut stack: Vec<Stack> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    let mut value: Option<Value> = None;
    for action in automaton {
        match action? {
            Action::Nothing => continue,
            Action::NewArray => stack.push(Stack::Array(Vec::new())),
            Action::NewObject => stack.push(Stack::Object(Vec::new())),
            Action::NewKey(k) => keys.push(k),
            Action::Push(v) => match stack.last_mut() {
                Some(Stack::Array(a)) => a.push(v),
                Some(Stack::Object(o)) => o.push((keys.pop().ok_or_else(|| internal_error!())?, v)),
                None => {value = Some(v)},
            },
            Action::Close => {
                let v = match stack.pop().ok_or_else(|| internal_error!())? {
                    Stack::Array(a) => Value::Array(a),
                    Stack::Object(o) => Value::Object(o),
                };
                match stack.last_mut() {
                    Some(Stack::Array(a)) => a.push(v),
                    Some(Stack::Object(o)) => o.push((keys.pop().ok_or_else(|| internal_error!())?, v)),
                    None => {value = Some(v)},
                }
            },
            Action::TheEnd => {
                return value.ok_or_else(||internal_error!())
            },
        }
    };
    value.ok_or_else(|| internal_error!())
}

enum Stack {
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

#[cfg(test)]
mod tests {
    use crate::{chars::Chars, lexer_iter::LexerIter};

    use super::*;

    #[test]
    fn easy() {
        let s = r#"[1,2,3,{}]"#;
        let s = LexerIter::new(Chars::from(s));
        let p = parse_and_valid(Box::new(s));
        p.unwrap();
    }

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
        let s = LexerIter::new(Chars::from(s));
        let p = parse_and_valid(Box::new(s));
        p.unwrap();
    }
}
