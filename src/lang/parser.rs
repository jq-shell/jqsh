use std::{fmt, mem};
use std::sync::{Arc, Mutex};

use itertools::{Itertools, MultiPeek};

use unicode::{self, UString};

use lang::{Context, Filter};
use lang::context::PrecedenceGroup;
use util::Labeled;

#[derive(Debug)]
pub enum ParseError {
    InvalidToken(char),
    MismatchedParens(Token, Tf),
    NotAllowed(Filter),
    NotFullyParsed(Vec<Tf>),
    UnbalancedParen(Token)
}

#[derive(Debug)]
pub enum Token {
    /// An unrecognized character
    Invalid(char),
    /// An opening parenthesis `(`
    OpenParen,
    /// A closing parenthesis `)`
    CloseParen,
    /// The sequential execution operator `;;`, and all following code
    AndThen(Code),
    /// A sequence of one or more whitespace characters
    Whitespace
}

//#[derive(Debug)] // https://github.com/bluss/rust-itertools/issues/32
enum CodeVariant {
    Empty,
    UString { s: UString, peek_index: usize },
    UStringIter(MultiPeek<unicode::IntoIter>),
    Mutation
}

impl fmt::Debug for CodeVariant { // https://github.com/bluss/rust-itertools/issues/32
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CodeVariant::Empty => try!(write!(w, "CodeVariant::Empty")),
            CodeVariant::UString { ref s, ref peek_index } => try!(write!(w, "CodeVariant::UString {{ s: {:?}, peek_index: {:?} }}", s, peek_index)),
            CodeVariant::UStringIter(_) => try!(write!(w, "CodeVariant::UStringIter(/* ... */)")),
            CodeVariant::Mutation => try!(write!(w, "CodeVariant::Mutation"))
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Code(Mutex<CodeVariant>);

impl Code {
    fn peek(&mut self) -> Option<char> {
        let mut lock = self.0.lock().unwrap();
        match *&mut *lock {
            CodeVariant::Empty => None,
            CodeVariant::UString { ref s, ref mut peek_index } => {
                if *peek_index < s.len() {
                    *peek_index += 1;
                    Some(s[*peek_index - 1])
                } else {
                    None
                }
            }
            CodeVariant::UStringIter(ref mut it) => {
                it.peek().map(|&c| c)
            }
            CodeVariant::Mutation => panic!("code mutex has been emptied")
        }
    }
}

impl Default for Code {
    fn default() -> Code {
        Code(Mutex::new(CodeVariant::Empty))
    }
}

impl<T: Into<UString>> From<T> for Code {
    fn from(code_string: T) -> Code {
        Code(Mutex::new(CodeVariant::UString { s: code_string.into(), peek_index: 0 }))
    }
}

impl Clone for Code {
    fn clone(&self) -> Code {
        let mut lock = self.0.lock().unwrap();
        match *&mut *lock {
            CodeVariant::Empty => Code(Mutex::new(CodeVariant::Empty)),
            CodeVariant::UString { ref s, .. } => Code(Mutex::new(CodeVariant::UString { s: s.clone(), peek_index: 0 })),
            ref mut code_variant @ CodeVariant::UStringIter(_) => {
                if let CodeVariant::UStringIter(it) = mem::replace(code_variant, CodeVariant::Mutation) {
                    let s = it.collect::<UString>();
                    *code_variant = CodeVariant::UString { s: s.clone(), peek_index: 0 };
                    Code(Mutex::new(CodeVariant::UString { s: s, peek_index: 0 }))
                } else {
                    unreachable!()
                }
            }
            CodeVariant::Mutation => panic!("code mutex has been emptied")
        }
    }
}

impl Iterator for Code {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let mut lock = self.0.lock().unwrap();
        match *&mut *lock {
            CodeVariant::Empty => None,
            ref mut code_variant @ CodeVariant::UString { .. } => {
                if let CodeVariant::UString { s, .. } = mem::replace(code_variant, CodeVariant::Mutation) {
                    let mut iter = s.into_iter().multipeek();
                    let result = iter.next();
                    *code_variant = CodeVariant::UStringIter(iter);
                    result
                } else {
                    unreachable!()
                }
            }
            CodeVariant::UStringIter(ref mut iter) => iter.next(),
            CodeVariant::Mutation => panic!("code mutex has been emptied")
        }
    }
}

struct Tokens {
    code: Code,
    // context: Context
}

impl Tokens {
    fn new<T: Into<Code>>(code: T, _: Context) -> Tokens {
        Tokens {
            code: code.into(),
            // context: context
        }
    }
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        use self::Token::*;

        match self.code.next() {
            Some('\t') |
            Some('\n') |
            Some('\r') |
            Some(' ') => Some(Whitespace),
            Some('#') => {
                while self.code.peek().map(|c| c != '\n').unwrap_or(false) {
                    self.code.next(); // discard comment contents
                }
                Some(Whitespace) // comments are treated as whitespace
            }
            Some('(') => Some(OpenParen),
            Some(')') => Some(CloseParen),
            Some(';') => {
                if self.code.peek() == Some(';') {
                    self.code.next(); // discard the second semicolon
                    Some(AndThen(mem::replace(&mut self.code, Code::default())))
                } else {
                    Some(Invalid(';'))
                }
            }
            Some(c) => Some(Invalid(c)),
            None => None
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
pub fn parse<T: Into<Code>>(code: T, context: Context) -> Result<Filter, ParseError> {
    parse_inner(Tokens::new(code, context.clone()).map(Tf::Token), context)
}

fn parse_inner<I: IntoIterator<Item = Tf>>(tf_iter: I, context: Context) -> Result<Filter, ParseError> {
    let mut tf = tf_iter.into_iter().collect::<Vec<_>>(); // the list of tokens and filters on which the in-place parsing algorithm operates
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
            match $f {
                f => {
                    if (context.filter_allowed)(&f) {
                        f
                    } else {
                        return Err(ParseError::NotAllowed(f));
                    }
                }
            }
        }
    }
    // remove leading and trailing whitespace as it is semantically irrelevant
    while let Some(&Tf::Token(Token::Whitespace)) = tf.first() { tf.remove(0); }
    while let Some(&Tf::Token(Token::Whitespace)) = tf.last() { tf.pop(); }
    // return an empty filter if the token list is empty
    if tf.len() == 0 { return Ok(try_filter!(Filter::Empty)); }
    // parse operators in decreasing precedence
    for (_, precedence_group) in context.operators.clone().into_iter().rev() { // iterate from highest to lowest precedence
        match precedence_group {
            PrecedenceGroup::AndThen => {
                let mut found = None; // flag any AndThen tokens and remember their contents
                for idx in (0..tf.len()).rev() { // iterate right-to-left for in-place manipulation
                    if let Some(remaining_code) = mem::replace(&mut found, None) {
                        if let Tf::Token(Token::Whitespace) = tf[idx] {
                            // ignore whitespace between `;;` and its left operand
                            tf.remove(idx);
                            continue;
                        }
                        tf[idx] = Tf::Filter(try_filter!(Filter::AndThen {
                            lhs: Box::new(if let Tf::Filter(ref lhs) = tf[idx] { lhs.clone() } else { try_filter!(Filter::Empty) }),
                            remaining_code: remaining_code
                        }));
                    } else {
                        match tf.remove(idx) {
                            Tf::Token(Token::AndThen(remaining_code)) => {
                                found = Some(remaining_code); // found an AndThen (`;;`), will be merged into a syntax tree with the element to its left
                            }
                            tf_item => {
                                tf.insert(idx, tf_item);
                            }
                        }
                    }
                }
                if let Some(remaining_code) = found {
                    // the code begins with an `;;`
                    tf.insert(0, Tf::Filter(try_filter!(Filter::AndThen {
                        lhs: Box::new(try_filter!(Filter::Empty)),
                        remaining_code: remaining_code
                    })));
                }
            }
            PrecedenceGroup::Circumfix => {
                let mut paren_balance = 0; // how many closing parens have not been matched by opening parens
                let mut paren_start = None; // the index of the outermost closing paren
                for idx in (0..tf.len()).rev() { // iterate right-to-left for in-place manipulation
                    match tf[idx] {
                        Tf::Token(Token::CloseParen) => {
                            if paren_balance == 0 {
                                paren_start = Some(idx);
                            }
                            paren_balance += 1;
                        }
                        Tf::Token(Token::OpenParen) => {
                            paren_balance -= 1;
                            if paren_balance < 0 {
                                return Err(ParseError::UnbalancedParen(Token::OpenParen));
                            } else if paren_balance == 0 {
                                if let Some(paren_start) = paren_start {
                                    if let Tf::Token(Token::CloseParen) = tf[paren_start] {
                                        tf.remove(paren_start);
                                        //let inner = tf.drain(idx + 1..paren_start).collect::<Vec<_>>(); //TODO use this when stabilized
                                        let mut inner = vec![];
                                        for _ in idx + 1..paren_start {
                                            inner.push(tf.remove(idx + 1));
                                        }
                                        tf[idx] = Tf::Filter(try_filter!(Filter::Custom {
                                            attributes: vec![try!(parse_inner(inner, context.clone()))],
                                            run: Box::new(Labeled::new("<filter group (α)>", Arc::new(|attrs, input, output| {
                                                assert_eq!(attrs.len(), 1);
                                                attrs[0].run(input, output)
                                            })))
                                        }));
                                    } else {
                                        return Err(ParseError::MismatchedParens(Token::OpenParen, tf.remove(paren_start)));
                                    }
                                } else {
                                    unreachable!();
                                }
                                paren_start = None;
                            }
                        }
                        _ => { continue; }
                    }
                }
                if paren_balance > 0 {
                    if let Some(paren_start) = paren_start {
                        if let Tf::Token(Token::CloseParen) = tf[paren_start] {
                            return Err(ParseError::UnbalancedParen(Token::CloseParen));
                        } else {
                            unreachable!();
                        }
                    } else {
                        unreachable!();
                    }
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
