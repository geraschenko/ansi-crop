# ansi-crop

[![GitHub](https://img.shields.io/badge/github-geraschenko%2Fansi--crop-blue?logo=github)](https://github.com/geraschenko/ansi-crop)
[![Crates.io](https://img.shields.io/crates/v/ansi-crop.svg)](https://crates.io/crates/ansi-crop)
[![Docs.rs](https://docs.rs/ansi-crop/badge.svg)](https://docs.rs/ansi-crop)

Crop ANSI-styled text to a rectangle of rows and columns while keeping its
styling.

`tmux capture-pane -e` gives a pane's text with its colours, padded with blank
rows and columns. `ansi-crop` sits between the capture and whatever turns ANSI
text into HTML, an image or anything else, and cuts the capture down to the
region that matters. By default that is the smallest rectangle containing
every visible cell:

```bash
tmux capture-pane -ep -t my-session | ansi-crop | aha --no-header > pane.html
```

Escape sequences from cropped-out rows and columns are kept, so a colour set
outside the rectangle still applies inside it.

## Install

```bash
cargo install ansi-crop
```

## Usage

```text
ansi-crop [OPTIONS] [INPUT]
```

`INPUT` is a file; `-` or no argument reads stdin. The output goes to stdout.

If you want to crop specific bounds rather than cropping out margins, use
`--row-min`/`--row-max`/`--col-min`/`--col-max` to set the minimum/maximum
row/column. Rows and columns are 1-indexed and inclusive. Each bound flag
replaces its own end of the starting rectangle, so `--col-min 1` keeps the left
margin and still trims the rest.

A cell is visible if it holds a non-whitespace character, or whitespace drawn
with a background colour, reverse video, underline, strikethrough or overline.
If nothing is visible, `ansi-crop` prints nothing.

Exit codes: 0 for success, 1 for I/O and parse errors, 2 for usage errors.

## Library

The crate is also a library. Without the default `cli` feature, its only
dependency is `unicode-width`:

```toml
[dependencies]
ansi-crop = { version = "0.1", default-features = false }
```

```rust
use ansi_crop::AnsiText;

let ansi_text = AnsiText::parse("\n  \x1b[1mab\x1b[0m  \n").unwrap();
if let Some(ink_bounds) = ansi_text.ink_bounds() {
  print!("{}", ansi_text.crop(&ink_bounds));
}
```

## Accepted input

- Text, with `\n` or `\r\n` line ends.
- SGR sequences: `ESC [`, digits, `;` and `:`, then `m`.
- nF escapes such as `ESC ( B`, which `tput sgr0` emits.
- OSC strings terminated by BEL or ST, including OSC 8 hyperlinks, and DCS,
  SOS, PM and APC strings terminated by ST.

Everything else is rejected with the row and column where it starts. That
includes cursor movement, erase, other escapes, tabs, backspace and a lone
`\r`, because they break the mapping from characters to cells. tmux 3.4's
`capture-pane -e` emits only accepted input; it expands tabs into spaces.

## Limitations

- Character widths come from `unicode-width`. Where it disagrees with the
  terminal, as it can for emoji and ambiguous-width characters, the crop is off
  by the difference.
- A wide character cut by the column bounds becomes one space per column kept.
- The output starts and ends with a reset (`ESC [0m`), and redundant SGRs from
  cropped-out regions stay in it.
- `ansi-crop` is not a terminal emulator: it does not model cursor movement,
  erase or overwriting.
