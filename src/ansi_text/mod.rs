//! [`AnsiText`] and its token representation. Each public method lives in
//! `impl_<method>.rs`.
mod escape;
mod impl_crop;
mod impl_ink_bounds;
mod impl_parse;
mod ink;
mod token;

pub use impl_parse::{ParseError, ParseErrorKind};
use token::Token;

/// ANSI text parsed into positioned cells; see the crate docs for accepted input.
#[derive(Debug)]
pub struct AnsiText<'a> {
  tokens: Vec<Token<'a>>,
}
