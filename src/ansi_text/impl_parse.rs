use std::fmt;
use std::iter::Peekable;
use std::str::CharIndices;

use unicode_width::UnicodeWidthChar;

use super::escape;
use super::{AnsiText, Token};

#[cfg(test)]
#[path = "impl_parse.test.rs"]
mod tests;

pub(super) type PeekableCharIndices<'a> = Peekable<CharIndices<'a>>;

impl<'a> AnsiText<'a> {
  /// Parses `input`, rejecting anything that breaks the mapping from
  /// characters to cells.
  pub fn parse(input: &'a str) -> Result<Self, ParseError> {
    let mut tokens: Vec<Token<'a>> = Vec::new();
    let mut row = 1;
    let mut col = 1;
    let mut char_indices = input.char_indices().peekable();
    while let Some((char_start, character)) = char_indices.next() {
      let error_here = move |kind: ParseErrorKind| ParseError { row, col, kind };
      match character {
        '\n' | '\r' => {
          if character == '\r' && char_indices.next_if(|&(_, next)| next == '\n').is_none() {
            return Err(error_here(ParseErrorKind::UnsupportedControl('\r')));
          }
          let text = &input[char_start..consumed_end(input, &mut char_indices)];
          tokens.push(Token::LineEnd { text, row });
          row += 1;
          col = 1;
        }
        '\x1b' => {
          tokens.push(escape::scan(input, char_start, &mut char_indices).map_err(error_here)?)
        }
        '\0'..='\x1f' | '\x7f'..='\u{9f}' => {
          return Err(error_here(ParseErrorKind::UnsupportedControl(character)));
        }
        _ => {
          let char_end = char_start + character.len_utf8();
          let width = character
            .width()
            .expect("non-controls have a width; see impl_parse.test.rs only_controls_lack_a_width");
          match tokens.last_mut() {
            // Every character yields a token or extends the last one, so a
            // last-token glyph ends exactly at `char_start`.
            Some(Token::Glyph { text, .. }) if width == 0 => {
              *text = &input[char_start - text.len()..char_end];
            }
            _ => {
              tokens.push(Token::Glyph {
                text: &input[char_start..char_end],
                row,
                col,
                width,
              });
              col += width;
            }
          }
        }
      }
    }
    Ok(AnsiText { tokens })
  }
}

/// The byte offset just past everything `char_indices` has yielded.
pub(super) fn consumed_end(input: &str, char_indices: &mut PeekableCharIndices<'_>) -> usize {
  char_indices.peek().map_or(input.len(), |&(index, _)| index)
}

/// Why and where [`AnsiText::parse`] rejected its input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
  /// The 1-indexed row where the offending token starts.
  pub row: usize,
  /// The 1-indexed column where the offending token starts.
  pub col: usize,
  /// What was rejected.
  pub kind: ParseErrorKind,
}

/// The kinds of input [`AnsiText::parse`] rejects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
  /// An escape sequence other than SGR, an nF escape or a string, from its
  /// `ESC` through its final byte or the first character that makes it invalid.
  UnsupportedEscape(String),
  /// A control character other than a line end.
  UnsupportedControl(char),
  /// An escape sequence cut off by the end of the input.
  UnterminatedEscape,
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "row {}, column {}: ", self.row, self.col)?;
    match &self.kind {
      ParseErrorKind::UnsupportedEscape(text) => write!(f, "unsupported escape {text:?}"),
      ParseErrorKind::UnsupportedControl(control) => write!(f, "unsupported control {control:?}"),
      ParseErrorKind::UnterminatedEscape => write!(f, "unterminated escape"),
    }
  }
}

impl std::error::Error for ParseError {}
