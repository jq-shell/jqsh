use unicode::UString;

use lang::{Context, Filter};

pub enum Token {
    Invalid(UString),
    Whitespace
}

// struct Tokens {
//     code: UString,
//     context: Context
// }
//
// impl Tokens {
//     fn new<T: Into<UString>>(code: T, context: Context) -> Tokens {
//         Tokens {
//             code: code.into(),
//             context: context
//         }
//     }
// }
//
// impl Iterator for Tokens {
//     type Item = Token;
//
//     fn next(&mut self) -> Option<Token> {
//         if self.code.len() > 0 {
//             use self::Token::*;
//
//             Some(match self.code[0] {
//                 ' ' => { self.code.remove(0); Whitespace }
//                 _ => {
//                     let code = self.code.clone();
//                     self.code = UString::default();
//                     Invalid(code)
//                 }
//             })
//         } else {
//             None
//         }
//     }
// }

pub fn parse<T: Into<UString> + ?Sized>(_: T, _: Context) -> Result<(Filter, Option<UString>), ()> {
    Err(()) //TODO
}
