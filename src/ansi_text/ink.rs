#[cfg(test)]
#[path = "ink.test.rs"]
mod tests;

/// SGR attributes that make a whitespace cell visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Ink {
  background: bool,
  reverse: bool,
  underline: bool,
  strikethrough: bool,
  overline: bool,
}

impl Ink {
  /// The ink state after an SGR with parameter string `params`; never fails,
  /// because the SGR itself is passed through verbatim.
  pub(crate) fn apply_sgr(mut self, params: &str) -> Ink {
    let mut param_iter = params.split(';').peekable();
    while let Some(param) = param_iter.next() {
      let (code, subparams) = match param.split_once(':') {
        Some((code, subparams)) => (code, Some(subparams)),
        None => (param, None),
      };
      match parse_param(code) {
        Some(0) => self = Ink::default(),
        Some(4) if subparams.and_then(|style| parse_param(style.split(':').next()?)) == Some(0) => {
          self.underline = false
        }
        Some(24) => self.underline = false,
        Some(4 | 21) => self.underline = true,
        Some(7) => self.reverse = true,
        Some(27) => self.reverse = false,
        Some(9) => self.strikethrough = true,
        Some(29) => self.strikethrough = false,
        Some(53) => self.overline = true,
        Some(55) => self.overline = false,
        Some(40..=47 | 100..=107) => self.background = true,
        Some(49) => self.background = false,
        Some(code @ (38 | 48 | 58)) => {
          self.background |= code == 48;
          // The colon form carries its colour arguments as subparameters.
          if subparams.is_none() {
            match param_iter.peek().copied().and_then(parse_param) {
              Some(5) => _ = param_iter.nth(1),
              Some(2) => _ = param_iter.nth(3),
              _ => {}
            }
          }
        }
        _ => {}
      }
    }
    self
  }

  pub(crate) fn is_visible(self) -> bool {
    let Ink {
      background,
      reverse,
      underline,
      strikethrough,
      overline,
    } = self;
    background || reverse || underline || strikethrough || overline
  }
}

/// A decimal SGR parameter; empty means 0, and `None` is out of range.
fn parse_param(param: &str) -> Option<u32> {
  if param.is_empty() {
    Some(0)
  } else {
    param.parse().ok()
  }
}
