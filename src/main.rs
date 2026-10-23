use std::borrow::Cow;
use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ansi_crop::{AnsiText, Bounds};
use clap::Parser;

/// Crop ANSI-styled text, such as `tmux capture-pane -e` cropped_text, keeping its styling.
///
/// By default the crop is the smallest rectangle containing every visible
/// cell. Each bound flag replaces its own end of that rectangle. Rows and
/// columns are 1-indexed and inclusive.
#[derive(Parser)]
#[command(version)]
struct Args {
  /// Input file; `-` or absent reads stdin.
  input: Option<PathBuf>,
  /// First row to keep.
  #[arg(long)]
  row_min: Option<NonZeroUsize>,
  /// Last row to keep.
  #[arg(long)]
  row_max: Option<NonZeroUsize>,
  /// First column to keep.
  #[arg(long)]
  col_min: Option<NonZeroUsize>,
  /// Last column to keep.
  #[arg(long)]
  col_max: Option<NonZeroUsize>,
}

fn main() -> ExitCode {
  let args = Args::parse();
  let (source_name, read_result) = match args.input.as_deref() {
    Some(path) if path != Path::new("-") => (path.display().to_string(), std::fs::read(path)),
    _ => {
      let mut input_bytes = Vec::new();
      let read_result = io::stdin()
        .read_to_end(&mut input_bytes)
        .map(|_| input_bytes);
      ("<stdin>".to_string(), read_result)
    }
  };
  let input_bytes = match read_result {
    Ok(input_bytes) => input_bytes,
    Err(error) => {
      eprintln!("ansi-crop: {source_name}: {error}");
      return ExitCode::FAILURE;
    }
  };
  let input = String::from_utf8_lossy(&input_bytes);
  if let Cow::Owned(_) = input {
    eprintln!("ansi-crop: warning: {source_name}: invalid UTF-8 replaced with U+FFFD");
  }
  let ansi_text = match AnsiText::parse(&input) {
    Ok(ansi_text) => ansi_text,
    Err(error) => {
      eprintln!("ansi-crop: {source_name}: {error}");
      return ExitCode::FAILURE;
    }
  };
  let Some(base_bounds) = ansi_text.ink_bounds() else {
    return ExitCode::SUCCESS;
  };
  let cropped_text = ansi_text.crop(&apply_overrides(base_bounds, &args));
  let mut stdout = io::stdout().lock();
  match stdout
    .write_all(cropped_text.as_bytes())
    .and_then(|()| stdout.flush())
  {
    Ok(()) => ExitCode::SUCCESS,
    // The reader stopping early, as in `| head`, is not an error.
    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("ansi-crop: <stdout>: {error}");
      ExitCode::FAILURE
    }
  }
}

/// Replaces each end of `base` that the corresponding flag sets.
fn apply_overrides(base: Bounds, args: &Args) -> Bounds {
  let flag_or =
    |flag: Option<NonZeroUsize>, base_value: usize| flag.map_or(base_value, NonZeroUsize::get);
  Bounds {
    rows: flag_or(args.row_min, *base.rows.start())..=flag_or(args.row_max, *base.rows.end()),
    cols: flag_or(args.col_min, *base.cols.start())..=flag_or(args.col_max, *base.cols.end()),
  }
}
