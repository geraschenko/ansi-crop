//! The `ansi-crop` binary end to end. The fixture `pane.ansi` is a tmux 3.4
//! `capture-pane -ep` of a 40×10 pane; the other fixtures are its expected crops.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn fixture_path(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(name)
}

fn run_ansi_crop(args: &[&str], stdin: &[u8]) -> Output {
  let mut child = Command::new(env!("CARGO_BIN_EXE_ansi-crop"))
    .args(args)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(stdin).unwrap();
  child.wait_with_output().unwrap()
}

fn stderr(output: &Output) -> &str {
  std::str::from_utf8(&output.stderr).unwrap()
}

fn assert_crops_fixture(args: &[&str], expected_fixture: &str) {
  let pane_path = fixture_path("pane.ansi");
  let expected_output = fs::read(fixture_path(expected_fixture)).unwrap();
  let pane_arg = pane_path.to_str().unwrap();
  let from_file = run_ansi_crop(&[args, &[pane_arg]].concat(), b"");
  let from_stdin = run_ansi_crop(args, &fs::read(&pane_path).unwrap());
  let from_dash = run_ansi_crop(&[args, &["-"]].concat(), &fs::read(&pane_path).unwrap());
  for output in [from_file, from_stdin, from_dash] {
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&expected_output)
    );
  }
}

#[test]
fn auto_crop_of_a_tmux_capture() {
  assert_crops_fixture(&[], "pane.cropped.ansi");
}

#[test]
fn explicit_bounds_on_a_tmux_capture() {
  assert_crops_fixture(
    &[
      "--col-min",
      "1",
      "--row-min",
      "5",
      "--row-max",
      "5",
      "--col-max",
      "22",
    ],
    "pane.row-5-cols-1-22.ansi",
  );
}

#[test]
fn flags_replace_their_end_of_the_ink_bounds() {
  let output = run_ansi_crop(&["--col-min", "1"], b"\n  ab\n");
  assert_eq!(output.stdout, b"\x1b[0m  ab\n\x1b[0m");
}

#[test]
fn blank_input_prints_nothing() {
  for input in [&b"      "[..], b""] {
    let output = run_ansi_crop(&[], input);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(stderr(&output), "");
  }
}

#[test]
fn parse_error_names_the_source_and_position() {
  let output = run_ansi_crop(&[], b"ab\n\x1b[5C");
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(output.stdout, b"");
  assert_eq!(
    stderr(&output),
    "ansi-crop: <stdin>: row 2, column 1: unsupported escape \"\\u{1b}[5C\"\n"
  );
}

#[test]
fn missing_file_is_an_io_error() {
  let output = run_ansi_crop(&["/nonexistent/ansi-crop-input"], b"");
  assert_eq!(output.status.code(), Some(1));
  assert!(
    stderr(&output).starts_with("ansi-crop: /nonexistent/ansi-crop-input: "),
    "{}",
    stderr(&output)
  );
}

#[test]
fn invalid_utf8_is_replaced_with_a_warning() {
  let output = run_ansi_crop(&[], b"a\xffb");
  assert!(output.status.success());
  assert_eq!(output.stdout, "\x1b[0ma\u{fffd}b\x1b[0m".as_bytes());
  assert_eq!(
    stderr(&output),
    "ansi-crop: warning: <stdin>: invalid UTF-8 replaced with U+FFFD\n"
  );
}

#[test]
fn zero_bound_is_a_usage_error() {
  let output = run_ansi_crop(&["--row-min", "0"], b"a");
  assert_eq!(output.status.code(), Some(2));
}

#[test]
fn closed_stdout_is_not_an_error() {
  let mut child = Command::new(env!("CARGO_BIN_EXE_ansi-crop"))
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  // The binary reads all of stdin before writing, so closing its stdout first
  // makes the write fail with a broken pipe.
  drop(child.stdout.take());
  child.stdin.take().unwrap().write_all(b"ab\n").unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success(), "{}", stderr(&output));
  assert_eq!(stderr(&output), "");
}
