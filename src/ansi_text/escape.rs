//! The escape sequences `AnsiText::parse` accepts; see the crate docs.
use super::Token;
use super::impl_parse::{ParseErrorKind, PeekableCharIndices, consumed_end};

/// Scans the escape sequence whose `ESC`, at byte `escape_start`, `char_indices` has just yielded.
pub(super) fn scan<'a>(
  input: &'a str,
  escape_start: usize,
  char_indices: &mut PeekableCharIndices<'a>,
) -> Result<Token<'a>, ParseErrorKind> {
  let introducer = next_char(char_indices)?;
  let is_accepted = match introducer {
    '[' => is_sgr(char_indices)?,
    ']' => is_control_string(char_indices, true)?,
    'P' | 'X' | '^' | '_' => is_control_string(char_indices, false)?,
    '\x20'..='\x2f' => is_nf(char_indices)?,
    _ => false,
  };
  let text = &input[escape_start..consumed_end(input, char_indices)];
  match (is_accepted, introducer) {
    (false, _) => Err(ParseErrorKind::UnsupportedEscape(text.to_string())),
    (true, '[') => Ok(Token::Sgr {
      text,
      params: &text[2..text.len() - 1],
    }),
    (true, _) => Ok(Token::Passthrough(text)),
  }
}

/// Consumes a CSI after its `ESC [`; it is SGR if its parameters are digits,
/// `;` and `:`, it has no intermediates, and its final byte is `m`.
fn is_sgr(char_indices: &mut PeekableCharIndices<'_>) -> Result<bool, ParseErrorKind> {
  let mut has_sgr_params = true;
  while let Some((_, c)) = char_indices.next_if(|&(_, c)| matches!(c, '\x30'..='\x3f')) {
    has_sgr_params &= matches!(c, '0'..='9' | ';' | ':');
  }
  let mut has_intermediates = false;
  while char_indices.next_if(|&(_, c)| is_intermediate(c)).is_some() {
    has_intermediates = true;
  }
  Ok(next_char(char_indices)? == 'm' && has_sgr_params && !has_intermediates)
}

/// Consumes the content and terminator of an OSC (which `bel_terminates`),
/// DCS, SOS, PM or APC; false if an `ESC` in it does not start ST.
fn is_control_string(
  char_indices: &mut PeekableCharIndices<'_>,
  bel_terminates: bool,
) -> Result<bool, ParseErrorKind> {
  loop {
    match next_char(char_indices)? {
      '\x07' if bel_terminates => return Ok(true),
      '\x1b' => return Ok(next_char(char_indices)? == '\\'),
      _ => {}
    }
  }
}

/// Consumes the rest of an nF escape after its first intermediate byte.
fn is_nf(char_indices: &mut PeekableCharIndices<'_>) -> Result<bool, ParseErrorKind> {
  while char_indices.next_if(|&(_, c)| is_intermediate(c)).is_some() {}
  Ok(matches!(next_char(char_indices)?, '\x30'..='\x7e'))
}

fn is_intermediate(character: char) -> bool {
  matches!(character, '\x20'..='\x2f')
}

/// The next character; the end of the input cuts the sequence off.
fn next_char(char_indices: &mut PeekableCharIndices<'_>) -> Result<char, ParseErrorKind> {
  char_indices
    .next()
    .map(|(_, c)| c)
    .ok_or(ParseErrorKind::UnterminatedEscape)
}
