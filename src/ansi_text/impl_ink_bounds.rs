use std::ops::RangeInclusive;

use super::ink::Ink;
use super::{AnsiText, Token};
use crate::bounds::Bounds;

#[cfg(test)]
#[path = "impl_ink_bounds.test.rs"]
mod tests;

impl AnsiText<'_> {
  /// The smallest bounds containing every inked cell; None if nothing is inked.
  pub fn ink_bounds(&self) -> Option<Bounds> {
    let mut ink = Ink::default();
    let mut ink_bounds: Option<Bounds> = None;
    for token in &self.tokens {
      match *token {
        Token::Sgr { params, .. } => ink = ink.apply_sgr(params),
        Token::Glyph {
          text,
          row,
          col,
          width,
        } if width > 0 => {
          let is_whitespace = text.starts_with(char::is_whitespace);
          if !is_whitespace || ink.is_visible() {
            let cell_bounds = Bounds {
              rows: row..=row,
              cols: col..=col + width - 1,
            };
            ink_bounds = Some(match ink_bounds {
              None => cell_bounds,
              Some(ink_bounds) => union(ink_bounds, cell_bounds),
            });
          }
        }
        _ => {}
      }
    }
    ink_bounds
  }
}

/// The smallest bounds containing both `a` and `b`.
fn union(a: Bounds, b: Bounds) -> Bounds {
  let hull = |a: RangeInclusive<usize>, b: RangeInclusive<usize>| {
    *a.start().min(b.start())..=*a.end().max(b.end())
  };
  Bounds {
    rows: hull(a.rows, b.rows),
    cols: hull(a.cols, b.cols),
  }
}
