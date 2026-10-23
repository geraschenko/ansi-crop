use super::*;

fn tokens(input: &str) -> Vec<Token<'_>> {
  AnsiText::parse(input).unwrap().tokens
}

fn parse_error(input: &str) -> ParseError {
  AnsiText::parse(input).err().unwrap()
}

fn unsupported_escape(row: usize, col: usize, text: &str) -> ParseError {
  ParseError {
    row,
    col,
    kind: ParseErrorKind::UnsupportedEscape(text.to_string()),
  }
}

#[test]
fn glyphs_are_positioned_by_width() {
  assert_eq!(
    tokens("a好b"),
    [
      Token::Glyph {
        text: "a",
        row: 1,
        col: 1,
        width: 1
      },
      Token::Glyph {
        text: "好",
        row: 1,
        col: 2,
        width: 2
      },
      Token::Glyph {
        text: "b",
        row: 1,
        col: 4,
        width: 1
      },
    ]
  );
}

#[test]
fn zero_width_characters_join_the_preceding_glyph() {
  assert_eq!(
    tokens("e\u{301}x"),
    [
      Token::Glyph {
        text: "e\u{301}",
        row: 1,
        col: 1,
        width: 1
      },
      Token::Glyph {
        text: "x",
        row: 1,
        col: 2,
        width: 1
      },
    ]
  );
}

#[test]
fn zero_width_character_without_a_preceding_glyph_is_a_width_0_glyph() {
  assert_eq!(
    tokens("a\n\u{301}\u{200d}"),
    [
      Token::Glyph {
        text: "a",
        row: 1,
        col: 1,
        width: 1
      },
      Token::LineEnd { text: "\n", row: 1 },
      Token::Glyph {
        text: "\u{301}\u{200d}",
        row: 2,
        col: 1,
        width: 0
      },
    ]
  );
}

/// The mark is not joined to `a` across the SGR, because a glyph is one
/// contiguous slice of the input.
#[test]
fn zero_width_character_after_an_escape_is_a_width_0_glyph() {
  assert_eq!(
    tokens("a\x1b[1m\u{301}"),
    [
      Token::Glyph {
        text: "a",
        row: 1,
        col: 1,
        width: 1
      },
      Token::Sgr {
        text: "\x1b[1m",
        params: "1"
      },
      Token::Glyph {
        text: "\u{301}",
        row: 1,
        col: 2,
        width: 0
      },
    ]
  );
}

#[test]
fn line_ends_are_kept_as_written() {
  assert_eq!(
    tokens("a\r\n\n"),
    [
      Token::Glyph {
        text: "a",
        row: 1,
        col: 1,
        width: 1
      },
      Token::LineEnd {
        text: "\r\n",
        row: 1
      },
      Token::LineEnd { text: "\n", row: 2 },
    ]
  );
}

#[test]
fn sgr_params_are_sliced_out() {
  assert_eq!(
    tokens("\x1b[m\x1b[38:2::1:2:3;4m"),
    [
      Token::Sgr {
        text: "\x1b[m",
        params: ""
      },
      Token::Sgr {
        text: "\x1b[38:2::1:2:3;4m",
        params: "38:2::1:2:3;4"
      },
    ]
  );
}

#[test]
fn nf_escapes_and_strings_pass_through() {
  for input in [
    "\x1b(B",
    "\x1b $@",
    "\x1b]8;;https://example.com\x1b\\",
    "\x1b]0;title\x07",
    "\x1bPq\x07\n\x1b\\",
    "\x1bXsos\x1b\\",
    "\x1b^pm\x1b\\",
    "\x1b_apc\x1b\\",
  ] {
    assert_eq!(tokens(input), [Token::Passthrough(input)], "{input:?}");
  }
}

#[test]
fn non_sgr_csi_is_rejected_through_its_final_byte() {
  assert_eq!(
    parse_error("ab\n\x1b[5C"),
    unsupported_escape(2, 1, "\x1b[5C")
  );
  assert_eq!(
    parse_error("\x1b[?25hx"),
    unsupported_escape(1, 1, "\x1b[?25h")
  );
  assert_eq!(
    parse_error("\x1b[1 mx"),
    unsupported_escape(1, 1, "\x1b[1 m")
  );
  assert_eq!(parse_error("\x1b[2Jx"), unsupported_escape(1, 1, "\x1b[2J"));
}

#[test]
fn csi_ending_in_m_with_non_sgr_parameters_is_rejected() {
  for input in ["\x1b[?1m", "\x1b[>4;2m", "\x1b[<m", "\x1b[=m", "\x1b[1;?m"] {
    assert_eq!(
      parse_error(&format!("{input}x")),
      unsupported_escape(1, 1, input)
    );
  }
}

#[test]
fn malformed_csi_is_rejected_through_the_offending_character() {
  assert_eq!(parse_error("\x1b[1éx"), unsupported_escape(1, 1, "\x1b[1é"));
  assert_eq!(
    parse_error("\x1b[1 2m"),
    unsupported_escape(1, 1, "\x1b[1 2")
  );
}

#[test]
fn other_escapes_are_rejected() {
  assert_eq!(parse_error("ab\x1b7"), unsupported_escape(1, 3, "\x1b7"));
  assert_eq!(parse_error("\x1bD"), unsupported_escape(1, 1, "\x1bD"));
  assert_eq!(parse_error("\x1bM"), unsupported_escape(1, 1, "\x1bM"));
  assert_eq!(parse_error("\x1b(\n"), unsupported_escape(1, 1, "\x1b(\n"));
  assert_eq!(
    parse_error("\x1b]8;;\x1bx"),
    unsupported_escape(1, 1, "\x1b]8;;\x1bx")
  );
}

#[test]
fn controls_are_rejected() {
  for (input, control) in [
    ("a\tb", '\t'),
    ("a\x08", '\x08'),
    ("a\rb", '\r'),
    ("a\r", '\r'),
    ("a\x07", '\x07'),
    ("a\0", '\0'),
    ("a\x7f", '\x7f'),
    ("a\u{85}", '\u{85}'),
    ("a\u{9f}", '\u{9f}'),
  ] {
    assert_eq!(
      parse_error(input),
      ParseError {
        row: 1,
        col: 2,
        kind: ParseErrorKind::UnsupportedControl(control)
      },
      "{input:?}"
    );
  }
}

#[test]
fn cut_off_sequences_are_unterminated() {
  for input in [
    "a\x1b",
    "a\x1b[",
    "a\x1b[12;",
    "a\x1b[1 ",
    "a\x1b(",
    "a\x1b]8;;x",
    "a\x1b]8;;x\x1b",
    "a\x1bPq\x07",
  ] {
    assert_eq!(
      parse_error(input),
      ParseError {
        row: 1,
        col: 2,
        kind: ParseErrorKind::UnterminatedEscape
      },
      "{input:?}"
    );
  }
}

#[test]
fn error_display_names_the_position_and_the_problem() {
  assert_eq!(
    parse_error("ab\n\x1b[5C").to_string(),
    "row 2, column 1: unsupported escape \"\\u{1b}[5C\""
  );
  assert_eq!(
    parse_error("a\tb").to_string(),
    "row 1, column 2: unsupported control '\\t'"
  );
  assert_eq!(
    parse_error("a\x1b]8;;x").to_string(),
    "row 1, column 2: unterminated escape"
  );
}

/// Keeps the `expect` in `AnsiText::parse`: `unicode-width` gives a width to
/// exactly the characters the parser does not classify as controls.
#[test]
fn only_controls_lack_a_width() {
  for c in (0..=char::MAX as u32).filter_map(char::from_u32) {
    let is_control = matches!(c, '\0'..='\x1f' | '\x7f'..='\u{9f}');
    assert_eq!(c.width().is_none(), is_control, "{c:?}");
  }
}
