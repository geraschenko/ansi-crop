use super::*;

fn ink_after(sgrs: &[&str]) -> Ink {
  sgrs
    .iter()
    .fold(Ink::default(), |ink, params| ink.apply_sgr(params))
}

fn ink_with(set: impl FnOnce(&mut Ink)) -> Ink {
  let mut ink = Ink::default();
  set(&mut ink);
  ink
}

fn assert_toggles(on: &str, off: &str, expected_on: Ink) {
  assert_eq!(ink_after(&[on]), expected_on, "{on}");
  assert!(ink_after(&[on]).is_visible(), "{on}");
  assert_eq!(ink_after(&[on, off]), Ink::default(), "{on} then {off}");
}

#[test]
fn attributes_turn_on_and_off() {
  assert_toggles("41", "49", ink_with(|ink| ink.background = true));
  assert_toggles("7", "27", ink_with(|ink| ink.reverse = true));
  assert_toggles("4", "24", ink_with(|ink| ink.underline = true));
  assert_toggles("9", "29", ink_with(|ink| ink.strikethrough = true));
  assert_toggles("53", "55", ink_with(|ink| ink.overline = true));
}

#[test]
fn reset_clears_everything() {
  let all_on = "41;7;4;9;53";
  assert!(ink_after(&[all_on]).is_visible());
  for reset in ["0", "", "41;7;4;9;53;0", "41;7;4;9;53;"] {
    assert_eq!(ink_after(&[all_on, reset]), Ink::default(), "{reset:?}");
  }
  assert!(!Ink::default().is_visible());
}

#[test]
fn underline_styles() {
  let underline = ink_with(|ink| ink.underline = true);
  for on in ["4:1", "4:3", "21"] {
    assert_eq!(ink_after(&[on]), underline, "{on}");
  }
  for off in ["4:0", "4:"] {
    assert_eq!(ink_after(&["4", off]), Ink::default(), "{off}");
  }
}

#[test]
fn background_colours() {
  let background = ink_with(|ink| ink.background = true);
  for on in [
    "40",
    "47",
    "100",
    "107",
    "48;5;0",
    "48;2;0;0;0",
    "48:5:0",
    "48",
  ] {
    assert_eq!(ink_after(&[on]), background, "{on}");
  }
  for not_background in ["39", "50", "99", "108", "38;5;1", "58;5;1"] {
    assert_eq!(
      ink_after(&[not_background]),
      Ink::default(),
      "{not_background}"
    );
  }
}

#[test]
fn colour_arguments_are_not_read_as_attributes() {
  assert_eq!(ink_after(&["38;5;4"]), Ink::default());
  assert_eq!(ink_after(&["38;2;4;7;9"]), Ink::default());
  assert_eq!(ink_after(&["58;5;7"]), Ink::default());
  assert_eq!(
    ink_after(&["38;2;1;2;3;4"]),
    ink_with(|ink| ink.underline = true)
  );
  assert_eq!(ink_after(&["38:5:4;7"]), ink_with(|ink| ink.reverse = true));
  assert_eq!(
    ink_after(&["38:2::4:7:9;9"]),
    ink_with(|ink| ink.strikethrough = true)
  );
}

#[test]
fn malformed_parameters_are_ignored() {
  assert_eq!(ink_after(&["38"]), Ink::default());
  assert_eq!(ink_after(&["38;5"]), Ink::default());
  assert_eq!(ink_after(&["38;2;1"]), Ink::default());
  assert_eq!(
    ink_after(&["38;9;4"]),
    ink_with(|ink| {
      ink.strikethrough = true;
      ink.underline = true;
    })
  );
  assert_eq!(
    ink_after(&["99999999999;7"]),
    ink_with(|ink| ink.reverse = true)
  );
  assert_eq!(
    ink_after(&["7", "1;2;3;5;6"]),
    ink_with(|ink| ink.reverse = true)
  );
}
