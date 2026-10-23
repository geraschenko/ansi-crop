# ansi-crop

# SPEC

## Problem

`tmux capture-pane -e` produces a pane's text with its SGR styling, padded
with blank rows and columns. Tools that turn ANSI text into HTML, images or
anything else want just the region that matters. `ansi-crop` crops ANSI text
to a rectangle of rows and columns while keeping the styling correct. By
default the rectangle is the smallest one containing every visible cell.

The tool exists as `~/git/muninn/ansi/src/bin/ansi-crop.rs` (343 lines) inside
the muninn workspace. This repo extracts it into a standalone package,
published on crates.io under Apache-2.0. The package has a library and a
binary named `ansi-crop`. The code is cleaned up to the usual standards, and
the behaviour changes listed below are made.

## Success criteria

1. `cargo install ansi-crop` installs a binary named `ansi-crop`.
2. `cargo tree -e normal --no-default-features` lists `unicode-width` as the
   only dependency; `clap` is reached only through the `cli` feature.
3. `scripts/presubmit.sh` passes: treefmt, `cargo ratchet --fail-on-change`,
   clippy with `-D warnings` on all targets and features, nextest,
   `cargo test --doc` (nextest skips doctests; this also compiles the
   README's Rust snippet), and `cargo doc --no-deps`.
4. `cargo package` succeeds, and `cargo package --list` holds only `src/`,
   `tests/`, `LICENSE`, `README.md` and the files cargo adds (`Cargo.toml`,
   `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`).
5. The fixtures in `tests/fixtures/` come from real tmux 3.4 captures. They
   crop to the committed expected output through the CLI, read both from a
   file and from stdin.
6. Each of the examples below is a unit or integration test.

## Definitions

- **Row, column**: 1-indexed terminal coordinates. Row _r_ is the _r_-th line
  of the input. A line end at the very end of the input closes the last row;
  it does not start an empty one. A glyph of width _w_ starting at column _c_
  occupies columns _c_ to _c_+_w_−1. Columns are counted with
  `unicode-width`.
- **Glyph**: a character of width ≥ 1, together with any zero-width
  characters (such as combining marks) that immediately follow it, with no
  escape or line end between. Any other zero-width character, at the start of
  a line or after an escape, is a glyph of width 0.
- **Ink state**: the SGR attributes that make a whitespace cell visible:
  background colour, reverse video, underline, strikethrough and overline.
  As in a terminal, the state carries across line ends.
- **Inked cell**: every column of a glyph of width ≥ 1 whose base character is
  not whitespace, or that is drawn while some ink attribute is on. Width-0
  glyphs are never inked. Any explicit background counts, including
  `48;5;0`, because the tool cannot know the terminal's default background,
  and an HTML renderer paints an explicit background anyway.
- **Ink bounds**: the smallest rows × columns rectangle containing every
  inked cell. There are none if no cell is inked.

## Accepted input

Parsing accepts the following and rejects everything else with a
`ParseError` that gives the row and column where the offending token starts:

- Text: characters other than controls.
- Line ends: `\n` and `\r\n`, kept exactly as written.
- SGR: `ESC [`, parameter bytes, final byte `m`. The parameter bytes are only
  digits, `;` and `:`, so there is no private marker (`<=>?`) and there are no
  intermediate bytes.
- nF escapes: `ESC`, one or more bytes in 0x20–0x2F, then a final byte in
  0x30–0x7E (for example the charset designation `ESC ( B` that `tput sgr0`
  emits).
- Strings: OSC (`ESC ]`), terminated by BEL or ST (`ESC \`), and DCS
  (`ESC P`), SOS (`ESC X`), PM (`ESC ^`) and APC (`ESC _`), terminated by ST.
  A string's content can be any characters except `ESC`. One case is OSC 8
  hyperlinks, which tmux emits with ST.

These are rejected because they move the cursor or otherwise break the
mapping from characters to cells:

- Any other CSI, such as cursor movement or erase: `UnsupportedEscape`.
- Any other ESC sequence, such as `ESC 7`, `ESC D` or `ESC M`, and an `ESC`
  inside a string that does not start ST: `UnsupportedEscape`.
- Controls other than the above: tab, backspace, a lone `\r`, BEL outside
  OSC, other C0 controls, DEL, and C1 (U+0080–U+009F): `UnsupportedControl`.
- A sequence cut off by the end of the input: `UnterminatedEscape`.

The text in `UnsupportedEscape` runs from the `ESC` through the final byte,
or through the first character that makes the sequence invalid.

tmux 3.4's `capture-pane -e` emits only text, `\n`, SGR and OSC 8. It expands
tabs into spaces.

## Crop semantics

`AnsiText::crop(bounds)`:

- Returns `""` for empty input. Otherwise the output starts and ends with
  `ESC[0m`.
- Emits every SGR and passthrough token (nF escapes and strings) verbatim and
  in input order, including those in cropped-out rows and columns. This keeps
  the style state, and any open hyperlink, correct for the visible cells.
- For rows within `bounds.rows`:
  - A glyph that lies entirely within `bounds.cols` is emitted as is.
  - A glyph that lies partly within them is replaced by one space per column
    inside the bounds, so that columns stay aligned.
  - A glyph that lies entirely outside them is dropped.
  - The row's line end is emitted if it has one. Short rows are not padded.
- Emits no glyphs or line ends for rows outside `bounds.rows`.
- An empty range (start > end) selects nothing.

## SGR ink rules

`Ink::apply_sgr` reads the parameters left to right, split at `;`. An empty
parameter means 0. A parameter can carry `:` subparameters.

| Parameters                    | Effect                                                                                                     |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `0`, empty                    | all ink off                                                                                                |
| `4`, `4:n` with n ≠ 0, `21`   | underline on                                                                                               |
| `4:0`, `24`                   | underline off                                                                                              |
| `7` / `27`                    | reverse on / off                                                                                           |
| `9` / `29`                    | strikethrough on / off                                                                                     |
| `53` / `55`                   | overline on / off                                                                                          |
| `40`–`47`, `100`–`107`, `48…` | background on                                                                                              |
| `49`                          | background off                                                                                             |
| `38…`, `48…`, `58…`           | colour arguments skipped: the colon form is self-contained; the semicolon form consumes `5;N` or `2;R;G;B` |
| anything else                 | ignored                                                                                                    |

Missing or truncated colour arguments are ignored. `Ink` never fails, because
the SGR text is passed through verbatim anyway.

## Examples

`·` stands for a space and `⏎` for `\n`. `R` is `ESC[0m`.

| Input                    | Operation                   | Output                                              |
| ------------------------ | --------------------------- | --------------------------------------------------- |
| `⏎··ab··⏎··c⏎⏎`          | ink bounds                  | rows 2..=3, cols 3..=4                              |
| same                     | crop to ink bounds          | `R` `ab⏎` `c⏎` `R`                                  |
| `··ESC[41m···ESC[49m·x⏎` | ink bounds                  | rows 1..=1, cols 3..=7                              |
| `ESC[4m··ESC[24m`        | ink bounds                  | rows 1..=1, cols 1..=2                              |
| `ESC[38;5;4m··`          | ink bounds                  | none (the `4` is a colour index, not underline)     |
| `ESC[38;5;1mab⏎cd⏎`      | crop rows 2..=2, cols 1..=2 | `R` `ESC[38;5;1m` `cd⏎` `R`                         |
| `a好b`                   | crop rows 1..=1, cols 3..=4 | `R` `·b` `R` (the right half of 好 becomes a space) |
| `a好b`                   | crop rows 1..=1, cols 1..=2 | `R` `a·` `R`                                        |
| `······`                 | ink bounds                  | none; the CLI prints nothing                        |
| `""`                     | ink bounds                  | none; crop gives `""`                               |
| `ab⏎ESC[5C`              | parse                       | error: row 2, column 1, unsupported escape `ESC[5C` |
| `a\tb`                   | parse                       | error: row 1, column 2, unsupported control `\t`    |
| `aESC]8;;x`              | parse                       | error: row 1, column 2, unterminated escape         |

## Type design

Package `ansi-crop`, library crate `ansi_crop`, binary `ansi-crop`.

```toml
[features]
default = ["cli"]
cli = ["dep:clap"]

[[bin]]
name = "ansi-crop"
path = "src/main.rs"
required-features = ["cli"]

[dependencies]
clap = { version = ">=4", features = ["derive"], optional = true }
unicode-width = ">=0.2"
```

```rust
// src/lib.rs: crate docs (purpose, accepted input, crop semantics in brief)
mod ansi_text;
mod bounds;
pub use ansi_text::{AnsiText, ParseError, ParseErrorKind};
pub use bounds::Bounds;

// src/ansi_text/mod.rs: AnsiText; each public method lives in
// impl_<method>.rs, and helper files (escape.rs, ink.rs, token.rs) keep plain
// names.
/// ANSI text parsed into positioned cells; see the crate docs for accepted input.
pub struct AnsiText<'a> { tokens: Vec<Token<'a>> }

// src/ansi_text/impl_parse.rs
impl<'a> AnsiText<'a> {
  pub fn parse(input: &'a str) -> Result<Self, ParseError> { todo!() }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError { pub row: usize, pub col: usize, pub kind: ParseErrorKind }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
  UnsupportedEscape(String),
  UnsupportedControl(char),
  UnterminatedEscape,
}
impl fmt::Display for ParseError { … }   // "row 2, column 1: unsupported escape \"\\u{1b}[5C\""
impl std::error::Error for ParseError {}

// src/ansi_text/token.rs
pub(super) enum Token<'a> {
  Glyph { text: &'a str, row: usize, col: usize, width: usize },
  Sgr { text: &'a str, params: &'a str },
  /// nF escapes and strings: verbatim, zero-width.
  Passthrough(&'a str),
  /// "\n" or "\r\n"; ends `row`.
  LineEnd { text: &'a str, row: usize },
}

// src/bounds.rs
/// 1-indexed, inclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds { pub rows: RangeInclusive<usize>, pub cols: RangeInclusive<usize> }

// src/ansi_text/impl_ink_bounds.rs
impl AnsiText<'_> {
  /// The smallest bounds containing every inked cell; None if nothing is inked.
  pub fn ink_bounds(&self) -> Option<Bounds> { todo!() }   // folds Ink::apply_sgr
}

// src/ansi_text/ink.rs
/// SGR attributes that make a whitespace cell visible.
#[derive(Clone, Copy, Default)]
pub(crate) struct Ink {
  background: bool, reverse: bool, underline: bool, strikethrough: bool, overline: bool,
}
impl Ink {
  pub(crate) fn apply_sgr(self, params: &str) -> Ink { todo!() }
  pub(crate) fn is_visible(self) -> bool { todo!() }
}

// src/ansi_text/impl_crop.rs
impl AnsiText<'_> {
  pub fn crop(&self, bounds: &Bounds) -> String { todo!() }
}

// src/main.rs
#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
  /// Input file; `-` or absent reads stdin.
  input: Option<PathBuf>,
  #[arg(long)] row_min: Option<NonZeroUsize>,
  #[arg(long)] row_max: Option<NonZeroUsize>,
  #[arg(long)] col_min: Option<NonZeroUsize>,
  #[arg(long)] col_max: Option<NonZeroUsize>,
}
fn main() -> ExitCode { todo!() }
/// Replaces each end of `base` that the corresponding flag sets.
fn apply_overrides(base: Bounds, args: &Args) -> Bounds { todo!() }
```

Dependencies: `ink_bounds` folds `Ink::apply_sgr` over the `Sgr` tokens and
asks `Ink::is_visible` for each whitespace glyph. `main` calls `parse`, then
`ink_bounds`, then `apply_overrides`, then `crop`.

## Data flow

1. `main` reads the bytes of FILE, or of stdin when FILE is `-` or absent.
2. `String::from_utf8_lossy`. If anything was replaced, it prints
   `ansi-crop: warning: <source>: invalid UTF-8 replaced with U+FFFD` to
   stderr, where `<source>` is the path or `<stdin>`.
3. `AnsiText::parse` produces the token vector. This is the only place input
   is validated. On error it prints `ansi-crop: <source>: <ParseError>` and
   exits with 1.
4. The base bounds are `ink_bounds()`.
   If there are none, it writes nothing and exits with 0.
5. `apply_overrides` lets each explicit flag replace its own end of the range.
6. `crop` builds the output `String`, which is written to stdout. If the
   write fails with `ErrorKind::BrokenPipe`, it exits with 0 silently: the
   reader stopping early, as in `| head`, is not an error. Rust ignores
   SIGPIPE, so the write returns `EPIPE` instead of the process being
   killed.

Exit codes: 0 for success, 1 for I/O or parse errors, 2 for usage errors
(clap).

## Cost

- **Memory**: one `Vec<Token>` of about 40 bytes per input character, plus
  the output string. That is a few hundred KB for a 200×60 capture.
- **Compute**: one parse pass, then two passes over the tokens (bounds, then
  crop). The second pass is justified because cropping needs the bounds of
  the whole input first.
- **Review focus**: the escape grammar in `parse` and the SGR parameter
  handling in `Ink::apply_sgr`. These are where the logic is and where most
  of the tests go.

## Repo contents

- The code above. Unit tests go in `<file>.test.rs` beside the file they test, declared there
  as `#[cfg(test)] #[path = "<file>.test.rs"] mod tests;`; the CLI tests go
  in `tests/cli.rs` and use `tests/fixtures/`.
- Formatting:
  - `rustfmt.toml`: `tab_spaces = 2`, `reorder_imports = true`, otherwise
    default.
  - `treefmt.toml` and `dprint.json`, as in `~/git/geraschenko/pi-rpc-rs`,
    except that rustfmt runs on the stable toolchain (no `+nightly`), because
    every option in `rustfmt.toml` is stable.
- Scripts:
  - `scripts/presubmit.sh`, as in pi-rpc-rs.
  - `scripts/release.sh`, as in pi-rpc-rs but without the pi credential
    step: it runs presubmit, `cargo package`, `cargo package --list`, asks
    for confirmation, then runs `cargo publish` and pushes the `v<version>`
    tag.
- `Cargo.lock` and the ratchet baseline, both committed.
- `Cargo.toml` metadata:
  - `authors = ["Anton Geraschenko <geraschenko@gmail.com>"]`
  - `license = "Apache-2.0"`, `readme`, `include = ["/src/", "/tests/", "/LICENSE", "/README.md"]`
  - `repository = "https://github.com/geraschenko/ansi-crop"`
  - a `description`, `keywords` and `categories`
- `LICENSE`: the Apache-2.0 text.
- `README.md`:
  - GitHub, crates.io and docs.rs badges, as in pi-rpc-rs.
  - The tool's role: a pre-processing step between `tmux capture-pane -e`
    and an HTML (or other) converter, with an example pipeline.
  - CLI usage, the library feature flag, the accepted input, and the known
    limitations.

## Edge cases and limitations

- `unicode-width` and tmux can disagree on the width of emoji and
  ambiguous-width characters. In that case the columns are tmux's and the
  crop is off. This is documented, not fixed.
- Width-0 glyphs are kept or dropped by their column, like any glyph.
- Explicit bounds beyond the text are allowed; they select nothing extra.
- Redundant SGRs from cropped-out regions stay in the output.

## Non-goals

- HTML output, and SGR normalization for particular converters
  (muninn's `normalize_sgr`).
- A terminal emulator: cursor movement, erase and overwriting are rejected,
  not modelled.
- Changes to muninn.

# IMPLEMENTATION IDEAS

- **Provenance**: muninn's `ansi-crop.rs`.
  - Logic to keep: escapes pass through even in cropped regions, and there
    is a leading and a trailing reset.
  - Logic to replace: its two hand-written escape scanners (in
    `detect_bounds` and `crop`) become the single `parse`.
  - Dependencies it used that this repo drops: `tokio`, `anyhow`, `base`,
    `prelude`.
- **tmux 3.4 probe** (2026-10-22), `capture-pane -ep` on a 40×8 pane:
  - Background-coloured and underlined spaces are emitted:
    `ESC[41m···ESC[49m`, `ESC[4m··ESC[0m`.
  - `ESC[7m … ESC[0m` is emitted for reverse video.
  - Tabs come out as spaces.
  - `38;5;N` and `48;2;R;G;B` are emitted.
  - OSC 8 is terminated with `ESC \`.
  - Empty pane rows come out as empty lines.
  - These are the facts the fixtures should pin. Make the fixtures with a
    detached `tmux -L <socket> -f /dev/null` server, and wait for the content
    to appear before capturing.
- **muninn's HTML path**, for the record (out of scope):
  - `ccc/web_dx/build.rs` runs `ansi::normalize_sgr` and then
    `ansi_to_html::convert`, using `ansi-to-html` 0.2.3.
  - With 0.2.4, the span leak that `normalize_sgr` works around
    (`38;5;12` A, `38;5;2` B, `38;5;12` C) renders correctly, and reverse
    video renders with the colours swapped in every combination tested
    (`7` alone, fg then `7`, `7` then fg, bg then `7` then `27`).
  - So muninn can likely bump to 0.2.4 and delete `normalize_sgr`, after
    checking that the web_dx pages render the same.
- **Parsing**: walk `char_indices`. Slice `text` out of the input so that
  tokens borrow and are never copied. Zero-width characters extend the
  previous glyph on the same row, which is the `text` slice of the last
  token if it is a `Glyph`. Classify controls by explicit code-point ranges
  before asking `unicode-width`.
- **unicode-width 0.2.2, checked 2026-10-22**:
  - `width()` is `None` for `\n`, `\t`, ESC, BEL and U+0085.
  - It is `Some(0)` for U+0301 (a combining accent) and U+200D (ZWJ), and
    `Some(2)` for 好.
  - The explicit control check therefore agrees with `unicode-width`. A
    `None` that reaches the width lookup is a bug.

# WORK LOG

**Instructions**: Update this section during each work session. Add new tasks, mark completed ones with [x], document decisions and problems encountered.

- [x] 2026-10-22: Derisk and type design approved. Fixed the authors-email
      typo in `~/git/geraschenko/pi-rpc-rs/Cargo.toml` (committed by Anton).
- [x] 2026-10-22: Spec approved for implementation. Decided: stable rustfmt,
      and a broken pipe on stdout exits 0 silently.
- [x] 2026-10-22: Type stubs compile with and without the `cli` feature.
- [x] 2026-10-22: Library and binary implemented; the tracer (example 1 via
      the CLI) worked on the first run. 36 unit tests pass. Bug found by a
      test: the semicolon colour form consumed an unknown selector (`38;9;4`
      swallowed the `9`); it now consumes only `5;N` and `2;R;G;B`.
- [x] 2026-10-22: Repo contents in place: tooling, scripts, LICENSE
      (apache.org text), README, fixtures and `tests/cli.rs` (45 tests).
      Presubmit passes. `cargo tree` and `cargo package --list` meet criteria
      2 and 4. The broken-pipe test was mutation-checked: removing the
      `BrokenPipe` arm makes it fail.
  - Fixtures: `pane.ansi` is `tmux -L ansi-crop-fixture -f /dev/null
    new-session -d -x 40 -y 10` running a printf script, captured with
    `capture-pane -ep` once polling (bounded retries) saw its last line
    `done`. The two expected crops were generated by the binary and checked by
    hand: auto crop is rows 3–8 × cols 3–31; `--no-auto-crop --row-min 5
    --row-max 5 --col-max 22` turns the half-cut 好 into one space (the test
    now passes `--col-min 1` instead, with byte-identical output).
- [x] 2026-10-22: Fresh-context review. No panics or wrong outputs on the
      spec'd behaviour; ~20 adversarial inputs were probed through the CLI.
      Findings and resolutions:
  - Fixed: the SGR parameter-byte check was untested (deleting it passed all
    tests). Added `csi_ending_in_m_with_non_sgr_parameters_is_rejected`
    (`ESC[?1m`, `ESC[>4;2m`, …); mutation-checked.
  - Fixed: the unicode-width `expect` had no keeper.
    `parse::tests::only_controls_lack_a_width` checks every `char`, and the
    `expect` message names it.
  - Fixed: `extent` took two passes; now one.
  - Fixed: `escape.rs` helpers took `(input, start, chars)` only to slice. They
    now return whether the sequence is accepted, and `scan` slices once. The
    CSI parameters are walked once.
  - Fixed: `ink_bounds` folds each cell's bounds in with a private `union`,
    instead of relying on rows being monotonic.
  - Fixed: `apply_sgr` takes `mut self` and matches on the code alone, with
    guards on `4:0` and the colon colour form. The parameter iterator is
    `param_iter`, so that `params` keeps meaning the string.
  - Fixed: the `\n` and `\r\n` arms were merged.
  - Fixed: added `#![warn(missing_docs)]` and docs on every public item, and
    `AnsiText` derives `Debug`.
  - Fixed: names (`error_here`, `char_end`, `PeekableCharIndices` rather than
    `Chars`, which collides with `std::str::Chars`, `is_control_string`,
    `source_name`, `base_bounds`, `flag_or`). The test helpers were renamed to
    `parse_error`, `ink_after`, `ink_with` and `some_bounds`.
  - Kept: `consumed_end` stays in `parse.rs`, not `escape.rs`, because the
    merged line-end arm uses it too.
  - Kept: blank input prints nothing even when all four bound flags are given.
    That is Data flow step 4 as approved.
  - Resolved by Anton (definition amended): a zero-width character after an
    escape (`a ESC[1m U+0301`). The old Glyph definition joined the mark to
    `a`, but a glyph's `text` is one contiguous slice, so it cannot span the
    SGR. The definition now joins only zero-width characters that immediately
    follow, so this mark is a width-0 glyph at the next column, cropped by
    that column, as the code already did. It is pinned by
    `zero_width_character_after_an_escape_is_a_width_0_glyph`. tmux stores a
    combining mark in its base's cell, so a capture never puts an SGR between
    them.
- [x] 2026-10-22: Doctests, approved by Anton. nextest does not run them, so
      presubmit now also runs `cargo test --doc` (success criterion 3
      amended). `lib.rs` includes `README.md` as a `#[cfg(doctest)]` doc, so
      the README's Rust snippet is compiled too.

- [x] 2026-10-22: Anton's review (commit c58ccb0, TDC comments). Decisions:
  - Drop `--no-auto-crop` (YAGNI; explicit bounds cover it).
  - Tests: flat files beside their module, `foo.test.rs`, declared in
    `foo.rs` as `#[cfg(test)] #[path = "foo.test.rs"] mod tests;`. This keeps
    them child modules with private access. CLAUDE.md is unchanged; Anton is
    still deciding test layout in general.
  - Layout: `src/ansi_text/mod.rs` holds `AnsiText` and `Token`. Each file
    implementing a method is `impl_<method>.rs` (`impl_parse.rs` with
    `ParseError`, `impl_extent.rs`, `impl_ink_bounds.rs` with `union`,
    `impl_crop.rs`). Small, coherent helper files keep plain names
    (`escape.rs`, `ink.rs`). `src/bounds.rs` holds `Bounds`.
  - Naming audit per CLAUDE.md: no single-letter long-lived names (`c` →
    `character`, `start` → `char_start` / `escape_start`), `parse_number` →
    `parse_param`.
  - Progress: layout moved and `--no-auto-crop` removed; 48 tests pass. The
    CLI fixture test uses `--col-min 1` in its place, and the output is
    byte-identical. `tokens` and `Token` are now private to `ansi_text`:
    child modules see a parent's private items, which supersedes the
    `pub(crate)` decision below. The naming audit also renamed `chars` →
    `char_indices` (it yields index pairs), `output` → `cropped_text`,
    `bytes` → `input_bytes`, and in `tests/cli.rs` `ansi_crop` →
    `run_ansi_crop`, `fixture` → `fixture_path`, and `expected` →
    `expected_output`. The 3-line `some_bounds` test helper is duplicated in
    the extent and ink-bounds test files rather than shared. Spec type design,
    data flow and README updated; presubmit passes.
  - `AnsiText::extent` had no caller left once `--no-auto-crop` went; Anton
    chose to drop it (YAGNI), with its tests and the Extent definition. The Row
    definition loses "any token after the last line end, even a lone SGR,
    makes a row of its own": only `extent` could observe it. That leaves one
    `some_bounds` helper, in `impl_ink_bounds.test.rs`.
- [x] 2026-10-22: Anton's review (commit 47bb2ee): README flag table became
      prose (his edit). `Token` moved to `src/ansi_text/token.rs` as
      `pub(super)`; `mod.rs` imports it, so siblings keep `super::Token`.
- [x] 2026-10-22: Package contents. `exclude` was a denylist, so the
      tooling configs (`.Cargo.lock.ratchet`, `dprint.json`, `rustfmt.toml`,
      `treefmt.toml`, `.gitignore`) shipped. An `include` allowlist replaces it.
      Tests stay in the package: distro packagers run them, and the `#[path]`
      test modules would break `cargo test` on the published source without
      them. Checked: all 46 tests and 2 doctests pass in
      `target/package/ansi-crop-0.1.0`.

## Implementation-Time Decisions

- **`AnsiText::tokens` was `pub(crate)`** (superseded: under `ansi_text/` it is private, since child modules see it): `bounds.rs` and
  `crop.rs` are sibling modules that implement methods on `AnsiText`, and a
  private field is visible only inside `parse.rs`.
- **Escape grammar in `src/ansi_text/escape.rs`**: a private module with one
  predicate per sequence family (`is_sgr`, `is_control_string`, `is_nf`). Each
  consumes its sequence and reports whether it is accepted; `scan` slices the
  text and builds the token or the error. It is the part with a
  meaning of its own, and this keeps `parse.rs` at the cell walk. It adds no
  types.
- **`Token` and `Ink` derive `Debug, PartialEq, Eq`** so that the tests can
  compare them directly.
- **The semicolon colour form peeks at the selector**: for `38;N` with N other
  than 5 or 2, N is not consumed and is read as an SGR of its own. This
  follows the SGR table literally. The alternative, consuming N, would hide a
  following attribute behind a malformed colour.
- **`48` with missing arguments still turns background on**: the table says
  `48…` is background on, and "missing arguments are ignored" means only that
  nothing more is consumed.
- **`[[test]] cli` has `required-features = ["cli"]`**: without it,
  `cargo test --no-default-features` fails to compile `tests/cli.rs`, which
  needs `CARGO_BIN_EXE_ansi-crop`.
- **`#[command(version)]` without `about`**: a bare `about` makes clap use the
  Cargo description and drop the `Args` doc comment, which explains the
  auto-crop and 1-indexing. Without it, `-h` shows the doc comment's first
  line and `--help` all of it.
- **Empty-range tests `#[expect(clippy::reversed_empty_ranges)]`**: `1..=0`
  and `2..=1` are the subject of those tests.
- **The binary has no `run` helper**: `main` handles each failure with an early
  return, which keeps the binary to the two functions in the type design.
