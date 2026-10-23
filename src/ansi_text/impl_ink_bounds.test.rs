use std::ops::RangeInclusive;

use super::*;

fn some_bounds(rows: RangeInclusive<usize>, cols: RangeInclusive<usize>) -> Option<Bounds> {
  Some(Bounds { rows, cols })
}

fn ink_bounds(input: &str) -> Option<Bounds> {
  AnsiText::parse(input).unwrap().ink_bounds()
}

#[test]
fn ink_bounds_skip_blank_rows_and_columns() {
  assert_eq!(ink_bounds("\n  ab  \n  c\n\n"), some_bounds(2..=3, 3..=4));
}

#[test]
fn ink_bounds_include_inked_whitespace() {
  assert_eq!(
    ink_bounds("  \x1b[41m   \x1b[49m x\n"),
    some_bounds(1..=1, 3..=7)
  );
  assert_eq!(ink_bounds("\x1b[4m  \x1b[24m"), some_bounds(1..=1, 1..=2));
  assert_eq!(
    ink_bounds("\x1b[48;5;0m \x1b[0m  "),
    some_bounds(1..=1, 1..=1)
  );
}

#[test]
fn ink_state_carries_across_line_ends() {
  assert_eq!(
    ink_bounds("\x1b[7m\n  \n\x1b[0m  "),
    some_bounds(2..=2, 1..=2)
  );
}

#[test]
fn colour_index_is_not_ink() {
  assert_eq!(ink_bounds("\x1b[38;5;4m  "), None);
}

#[test]
fn wide_glyphs_ink_every_column() {
  assert_eq!(ink_bounds(" 好"), some_bounds(1..=1, 2..=3));
}

#[test]
fn width_0_glyphs_are_never_inked() {
  assert_eq!(ink_bounds("\x1b[41m\u{301}\n"), None);
}

#[test]
fn blank_input_has_no_ink_bounds() {
  assert_eq!(ink_bounds("      "), None);
  assert_eq!(ink_bounds(""), None);
}
