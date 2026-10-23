#[derive(Debug, PartialEq, Eq)]
pub(super) enum Token<'a> {
  Glyph {
    text: &'a str,
    row: usize,
    col: usize,
    width: usize,
  },
  Sgr {
    text: &'a str,
    params: &'a str,
  },
  /// nF escapes and strings: verbatim, zero-width.
  Passthrough(&'a str),
  /// "\n" or "\r\n"; ends `row`.
  LineEnd {
    text: &'a str,
    row: usize,
  },
}
