//! Crop ANSI-styled text to a rectangle of rows and columns, keeping its
//! styling. Typical input is a `tmux capture-pane -e` capture.
//!
//! ```
//! use ansi_crop::AnsiText;
//!
//! let ansi_text = AnsiText::parse("\n  \x1b[1mab\x1b[0m  \n").unwrap();
//! let ink_bounds = ansi_text.ink_bounds().unwrap();
//! assert_eq!(ansi_text.crop(&ink_bounds), "\x1b[0m\x1b[1mab\x1b[0m\n\x1b[0m");
//! ```
//!
//! # Coordinates
//!
//! Rows and columns are 1-indexed. Each line of the input is a row; a final
//! line end closes the last row rather than starting an empty one. Columns are
//! counted with `unicode-width`, so a wide glyph such as `好` spans two.
//! [`AnsiText::ink_bounds`] covers the visible cells: non-whitespace
//! glyphs, and whitespace drawn with a background colour, reverse video,
//! underline, strikethrough or overline.
//!
//! # Accepted input
//!
//! - Text, with line ends `\n` and `\r\n`.
//! - SGR sequences (`ESC [` digits, `;` and `:`, then `m`).
//! - nF escapes such as `ESC ( B`.
//! - OSC strings terminated by BEL or ST, and DCS, SOS, PM and APC strings
//!   terminated by ST. OSC 8 hyperlinks are one case.
//!
//! [`AnsiText::parse`] rejects everything else, such as cursor movement,
//! erase, tabs and a lone `\r`, because those break the mapping from characters
//! to cells. The [`ParseError`] gives the row and column where the offending
//! sequence starts.
//!
//! # Cropping
//!
//! [`AnsiText::crop`] emits every escape sequence in input order, including
//! those in cropped-out rows and columns, so that the styling of the visible
//! cells and any open hyperlink stay correct. Its output starts and ends with
//! a reset (`ESC [0m`). A wide glyph cut by the column bounds becomes one
//! space per visible column, which keeps columns aligned.
#![warn(missing_docs)]
mod ansi_text;
mod bounds;
pub use ansi_text::{AnsiText, ParseError, ParseErrorKind};
pub use bounds::Bounds;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
