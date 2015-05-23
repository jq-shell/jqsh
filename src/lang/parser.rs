use std::mem;

use unicode::UString;

use lang::{Context, Filter};
use lang::context::PrecedenceGroup;

#[derive(Debug)]
pub enum ParseError {
    InvalidToken(char),
    NotAllowed(Filter),
    NotFullyParsed(Vec<Tf>)
}

#[derive(Debug)]
pub enum Token {
    /// An unrecognized character
    Invalid(char),
    /// The sequential execution operator `;;`, and all following code
    AndThen(UString),
    /// A sequence of one or more whitespace characters
    Whitespace
}

struct Tokens {
    code: UString,
    // context: Context
}

impl Tokens {
    fn new<C: Into<char>, T: IntoIterator<Item=C>>(code: T, _: Context) -> Tokens {
        Tokens {
            code: code.into_iter().collect(), //TODO don't immediately collect all the code
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
                ';' if self.code.len() >= 2 && self.code[1] == ';' => {
                    let code = self.code[2..].to_owned();
                    self.code = UString::default();
                    AndThen(code)
                }
                _ => { Invalid(self.code.remove(0)) }
            })
        } else {
            None
        }
    }
}

/// A token or filter, used by the in-place parsing algorithm.
#[derive(Debug)]
pub enum Tf {
    Token(Token),
    Filter(Filter)
}

/// Convert a sequence of tokens into an executable filter.
pub fn parse<C: Into<char>, T: IntoIterator<Item=C>>(code: T, context: Context) -> Result<Filter, ParseError> {
    let mut tf = Tokens::new(code, context.clone()).map(Tf::Token).collect::<Vec<_>>(); // the list of tokens and filters on which the in-place parsing algorithm operates
    // error if any invalid token is found
    if let Some(pos) = tf.iter().position(|i| if let Tf::Token(Token::Invalid(_)) = *i { true } else { false }) {
        if let Tf::Token(Token::Invalid(c)) = tf[pos] {
            return Err(ParseError::InvalidToken(c));
        } else {
            unreachable!();
        }
    }
    // define the macro used for testing if filters are allowed
    macro_rules! try_filter {
        ($f:expr) => {
            if (context.filter_allowed)($f) {
                $f
            } else {
                return Err(ParseError::NotAllowed($f));
            }
        }
    }
    // remove leading and trailing whitespace as it is semantically irrelevant
    while let Some(&Tf::Token(Token::Whitespace)) = tf.first() { tf.remove(0); }
    while let Some(&Tf::Token(Token::Whitespace)) = tf.last() { tf.pop(); }
    // return an empty filter if the token list is empty
    if tf.len() == 0 { return Ok(try_filter!(Filter::Empty)); }
    // parse operators in decreasing precedence
    for precedence_group in context.operators {
        match precedence_group {
            PrecedenceGroup::AndThen => {
                let mut found = None; // flag any AndThen tokens and remember their contents
                for idx in (0..tf.len()).rev() { // iterate right-to-left for in-place manipulation
                    if let Some(ref remaining_code) = mem::replace(&mut found, None) {
                        if let Tf::Token(Token::Whitespace) = tf[idx] {
                            // ignore whitespace between `;;` and its left operand
                            tf.remove(idx);
                            continue;
                        }
                        tf[idx] = Tf::Filter(try_filter!(Filter::AndThen {
                            lhs: Box::new(if let Tf::Filter(ref lhs) = tf[idx] { lhs.clone() } else { try_filter!(Filter::Empty) }),
                            remaining_code: Clone::clone(&remaining_code)
                        }));
                        tf.remove(idx + 1);
                    } else if let Tf::Token(Token::AndThen(ref remaining_code)) = tf[idx] {
                        found = Some(remaining_code.clone()); // found an AndThen (`;;`), will be merged into a syntax tree with the element to its left
                    }
                }
                if let Some(ref remaining_code) = found {
                    // the code begins with an `;;`
                    tf[0] = Tf::Filter(try_filter!(Filter::AndThen { lhs: Box::new(try_filter!(Filter::Empty)), remaining_code: remaining_code.clone() }));
                }
            }
        }
    }
    if tf.len() == 1 {
        match tf.pop() {
            Some(Tf::Filter(result)) => Ok(result),
            Some(Tf::Token(token)) => Err(ParseError::NotFullyParsed(vec![Tf::Token(token)])),
            None => unreachable!()
        }
    } else {
        Err(ParseError::NotFullyParsed(tf))
    }
}
