use std::ops::RangeInclusive;

/// A rectangle of rows and columns, 1-indexed and inclusive; a range whose
/// start exceeds its end is empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
  /// The rows, as line numbers of the input.
  pub rows: RangeInclusive<usize>,
  /// The terminal columns, counted with `unicode-width`.
  pub cols: RangeInclusive<usize>,
}
