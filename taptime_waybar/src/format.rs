use taptime_client::ipc::Status;

const MISSING: &str = "--:--";

/// Which components of a duration to print.
#[derive(Clone, Copy, Debug)]
pub struct Clock {
  pub hours: bool,
  pub minutes: bool,
  pub seconds: bool,
}

impl Clock {
  pub const HM: Self = Self {
    hours: true,
    minutes: true,
    seconds: false,
  };

  /// Formats the absolute value of `seconds`. The leading component absorbs
  /// the ones that are disabled, e.g. minutes-only prints `135` for 2h15m.
  pub fn format(self, seconds: i64) -> String {
    let n = seconds.unsigned_abs();
    let mut parts = Vec::with_capacity(3);
    if self.hours {
      parts.push(n / 3600);
    }
    if self.minutes {
      parts.push(if self.hours { n / 60 % 60 } else { n / 60 });
    }
    if self.seconds {
      parts.push(if self.hours || self.minutes {
        n % 60
      } else {
        n
      });
    }
    parts
      .iter()
      .map(|part| format!("{part:02}"))
      .collect::<Vec<_>>()
      .join(":")
  }
}

/// Expands `{name}` placeholders in user templates. Unknown placeholders are
/// left untouched.
pub struct Formatter<'a> {
  pub clock: Clock,
  pub icon: &'a str,
  /// Templates for signed balances; `{value}` is the unsigned duration.
  pub positive: &'a str,
  pub negative: &'a str,
  pub active_text: &'a str,
  pub inactive_text: &'a str,
  pub show_balance: bool,
}

impl Formatter<'static> {
  /// Fixed `HH:MM` formatting used for notification texts.
  pub const PLAIN: Self = Self {
    clock: Clock::HM,
    icon: "",
    positive: "+{value}",
    negative: "-{value}",
    active_text: "true",
    inactive_text: "false",
    show_balance: true,
  };
}

impl Formatter<'_> {
  pub fn expand(&self, template: &str, status: Option<&Status>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
      out.push_str(&rest[..start]);
      rest = &rest[start..];
      let value = rest
        .find('}')
        .and_then(|end| Some((end, self.field(&rest[1..end], status)?)));
      match value {
        Some((end, value)) => {
          out.push_str(&value);
          rest = &rest[end + 1..];
        }
        None => {
          out.push('{');
          rest = &rest[1..];
        }
      }
    }
    out.push_str(rest);
    out
  }

  fn field(&self, name: &str, status: Option<&Status>) -> Option<String> {
    let duration = |get: fn(&Status) -> Option<i64>| {
      status
        .and_then(get)
        .map_or_else(|| MISSING.into(), |seconds| self.clock.format(seconds))
    };
    Some(match name {
      "icon" => self.icon.into(),
      "date" => status.and_then(|s| s.date.clone()).unwrap_or_default(),
      "active" if status.is_some_and(|s| s.active) => self.active_text.into(),
      "active" => self.inactive_text.into(),
      "worked" => duration(|s| s.worked_seconds),
      "presence" => duration(|s| s.presence_seconds),
      "work_target" => duration(|s| s.work_target_seconds),
      "presence_target" => duration(|s| s.presence_target_seconds),
      "remaining" => duration(remaining_seconds),
      "balance" if !self.show_balance => String::new(),
      "balance" => self.signed(status.and_then(|s| s.balance_seconds)),
      "month_balance" => self.signed(status.and_then(|s| s.month_balance_seconds)),
      _ => return None,
    })
  }

  pub fn signed(&self, seconds: Option<i64>) -> String {
    let Some(seconds) = seconds else {
      return MISSING.into();
    };
    let template = if seconds >= 0 {
      self.positive
    } else {
      self.negative
    };
    template.replace("{value}", &self.clock.format(seconds))
  }
}

/// Presence left until the daily target, never negative.
pub fn remaining_seconds(status: &Status) -> Option<i64> {
  let target = status.presence_target_seconds?;
  let presence = status.presence_seconds?;
  Some(target.saturating_sub(presence).max(0))
}

#[cfg(test)]
pub(crate) fn status(value: serde_json::Value) -> Status {
  let mut base = serde_json::json!({
    "version": taptime_client::ipc::VERSION,
    "sampled_at": "2026-11-13T09:00:00Z",
    "stale": false,
    "active": false,
  });
  for (key, value) in value.as_object().unwrap() {
    base[key] = value.clone();
  }
  serde_json::from_value(base).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clock() {
    assert_eq!(Clock::HM.format(-8100), "02:15");
    let all = Clock {
      seconds: true,
      ..Clock::HM
    };
    assert_eq!(all.format(3661), "01:01:01");
    let minutes = Clock {
      hours: false,
      ..Clock::HM
    };
    assert_eq!(minutes.format(8100), "135");
  }

  #[test]
  fn lunch_target_negative_balance_and_unknown_tokens() {
    let status = status(serde_json::json!({
      "worked_seconds": 22500,
      "presence_seconds": 22500,
      "presence_target_seconds": 30600,
      "balance_seconds": -8100,
    }));
    assert_eq!(
      Formatter::PLAIN.expand(
        "{worked} {balance} {remaining} {custom} {{worked}} {",
        Some(&status)
      ),
      "06:15 -02:15 02:15 {custom} {06:15} {"
    );
  }

  #[test]
  fn missing_values() {
    assert_eq!(
      Formatter::PLAIN.expand("[{date}] {worked} {balance} {remaining} {active}", None),
      "[] --:-- --:-- --:-- false"
    );
  }
}
