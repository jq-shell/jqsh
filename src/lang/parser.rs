use unicode::UString;

use lang::{Context, Filter};

#[derive(Debug)]
pub enum ParseError {
    InvalidToken(UString)
}

pub enum Token {
    Invalid(UString),
    Whitespace
}

struct Tokens {
    code: UString,
    // context: Context
}

impl Tokens {
    fn new<T: Into<UString>>(code: T, _: Context) -> Tokens {
        Tokens {
            code: code.into(),
            // context: context
        }
    }
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.code.len() > 0 {
            use self::Token::*;

            Some(match self.code[0] {
                ' ' => { self.code.remove(0); Whitespace }
                _ => {
                    let code = self.code.clone();
                    self.code = UString::default();
                    Invalid(code)
                }
            })
        } else {
            None
        }
    }
}

pub fn parse<T: Into<UString> + ?Sized>(code: T, context: Context) -> Result<(Filter, Option<UString>), ParseError> {
    for token in Tokens::new(code, context) {
        use self::Token::*;

        match token {
            Invalid(text) => { return Err(ParseError::InvalidToken(text)); }
            Whitespace => { /* ignore whitespace for now */ }
        }
    }
    Ok((Filter::Empty, None))
}
