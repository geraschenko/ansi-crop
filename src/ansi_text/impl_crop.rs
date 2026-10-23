use super::{AnsiText, Token};
use crate::bounds::Bounds;

#[cfg(test)]
#[path = "impl_crop.test.rs"]
mod tests;

const RESET: &str = "\x1b[0m";

impl AnsiText<'_> {
  /// The text inside `bounds`, between resets, with every escape kept so that
  /// the styling of the visible cells is unchanged; `""` for empty input.
  pub fn crop(&self, bounds: &Bounds) -> String {
    if self.tokens.is_empty() {
      return String::new();
    }
    let mut cropped_text = String::from(RESET);
    for token in &self.tokens {
      match *token {
        Token::Sgr { text, .. } | Token::Passthrough(text) => cropped_text.push_str(text),
        Token::LineEnd { text, row } => {
          if bounds.rows.contains(&row) {
            cropped_text.push_str(text);
          }
        }
        Token::Glyph {
          text,
          row,
          col,
          width,
        } => {
          if !bounds.rows.contains(&row) {
            continue;
          }
          if width == 0 {
            if bounds.cols.contains(&col) {
              cropped_text.push_str(text);
            }
            continue;
          }
          let first_visible_col = col.max(*bounds.cols.start());
          let last_visible_col = (col + width - 1).min(*bounds.cols.end());
          let visible_width = (last_visible_col + 1).saturating_sub(first_visible_col);
          if visible_width == width {
            cropped_text.push_str(text);
          } else {
            cropped_text.extend(std::iter::repeat_n(' ', visible_width));
          }
        }
      }
    }
    cropped_text.push_str(RESET);
    cropped_text
  }
}
