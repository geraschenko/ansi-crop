use std::ops::RangeInclusive;

use super::*;

fn crop(input: &str, rows: RangeInclusive<usize>, cols: RangeInclusive<usize>) -> String {
  AnsiText::parse(input).unwrap().crop(&Bounds { rows, cols })
}

#[test]
fn crop_to_ink_bounds() {
  let ansi_text = AnsiText::parse("\n  ab  \n  c\n\n").unwrap();
  let ink_bounds = ansi_text.ink_bounds().unwrap();
  assert_eq!(ansi_text.crop(&ink_bounds), "\x1b[0mab\nc\n\x1b[0m");
}

#[test]
fn escapes_in_cropped_out_rows_are_kept() {
  assert_eq!(
    crop("\x1b[38;5;1mab\ncd\n", 2..=2, 1..=2),
    "\x1b[0m\x1b[38;5;1mcd\n\x1b[0m"
  );
  assert_eq!(
    crop("\x1b]8;;u\x1b\\ab\x1b]8;;\x1b\\\ncd", 2..=2, 1..=2),
    "\x1b[0m\x1b]8;;u\x1b\\\x1b]8;;\x1b\\cd\x1b[0m"
  );
}

#[test]
fn escapes_in_cropped_out_columns_are_kept() {
  assert_eq!(crop("a\x1b[1mbc", 1..=1, 3..=3), "\x1b[0m\x1b[1mc\x1b[0m");
}

#[test]
fn partly_visible_wide_glyph_becomes_spaces() {
  assert_eq!(crop("a好b", 1..=1, 3..=4), "\x1b[0m b\x1b[0m");
  assert_eq!(crop("a好b", 1..=1, 1..=2), "\x1b[0ma \x1b[0m");
  assert_eq!(crop("a好b", 1..=1, 2..=3), "\x1b[0m好\x1b[0m");
}

#[test]
fn width_0_glyphs_are_kept_by_their_column() {
  assert_eq!(crop("\u{301}\n\u{302}", 1..=2, 2..=2), "\x1b[0m\n\x1b[0m");
  assert_eq!(
    crop("\u{301}\n\u{302}", 2..=2, 1..=1),
    "\x1b[0m\u{302}\x1b[0m"
  );
}

#[test]
fn line_ends_are_kept_as_written_and_short_rows_unpadded() {
  assert_eq!(crop("abc\r\nd\n", 1..=2, 2..=3), "\x1b[0mbc\r\n\n\x1b[0m");
}

#[test]
fn bounds_beyond_the_text_select_nothing_extra() {
  assert_eq!(crop("ab\n", 1..=9, 1..=9), "\x1b[0mab\n\x1b[0m");
}

#[test]
#[expect(clippy::reversed_empty_ranges, reason = "empty ranges are the subject")]
fn empty_range_selects_nothing() {
  assert_eq!(crop("ab\ncd", 2..=1, 1..=2), "\x1b[0m\x1b[0m");
  assert_eq!(crop("ab\ncd", 1..=2, 1..=0), "\x1b[0m\n\x1b[0m");
}

#[test]
fn empty_input_crops_to_nothing() {
  assert_eq!(crop("", 1..=1, 1..=1), "");
}
