use serde::Serialize;
use taptime_client::ipc::Status;

use crate::{
  config::{DisplayConfig, Unavailable},
  format::{Formatter, remaining_seconds},
};

/// One line of the Waybar `return-type: json` protocol, plus extra fields
/// consumed by the Quickshell frontends.
#[derive(Debug, Serialize)]
pub struct Output {
  pub text: String,
  pub tooltip: String,
  pub class: Vec<String>,
  pub alt: &'static str,
  pub worked: String,
  pub balance: String,
  pub balance_positive: bool,
  pub month_balance: String,
  pub month_balance_positive: bool,
  pub start_time: Option<String>,
  pub end_time: Option<String>,
  pub active: Option<bool>,
  pub date: Option<String>,
  /// Raw counters for frontends that draw progress themselves.
  pub seconds: Seconds,
  /// Daemon sync error: `unauthenticated`, `session_unavailable`,
  /// `api_unavailable`.
  pub error: Option<String>,
  /// The daemon socket was unreachable; everything else is the last snapshot.
  pub disconnected: bool,
  pub last_sync: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Seconds {
  pub worked: Option<i64>,
  pub presence: Option<i64>,
  pub work_target: Option<i64>,
  pub presence_target: Option<i64>,
  pub balance: Option<i64>,
  pub month_balance: Option<i64>,
  pub remaining: Option<i64>,
}

impl Seconds {
  fn new(status: &Status) -> Self {
    Self {
      worked: status.worked_seconds,
      presence: status.presence_seconds,
      work_target: status.work_target_seconds,
      presence_target: status.presence_target_seconds,
      balance: status.balance_seconds,
      month_balance: status.month_balance_seconds,
      remaining: remaining_seconds(status),
    }
  }
}

/// Renders the last known `status`. `disconnected` means the latest IPC
/// request failed, so `status` (if any) is an older snapshot.
pub fn render(status: Option<&Status>, config: &DisplayConfig, disconnected: bool) -> Output {
  let formatter = Formatter {
    clock: config.clock(),
    icon: &config.icon,
    positive: &config.positive,
    negative: &config.negative,
    active_text: &config.active_text,
    inactive_text: &config.inactive_text,
    show_balance: config.show_balance,
  };
  let expand = |template: &str| formatter.expand(template, status);

  let stale = disconnected || status.is_none_or(|s| s.stale);
  let active = status.is_some_and(|s| s.active);
  let balance_positive = status
    .and_then(|s| s.balance_seconds)
    .is_some_and(|b| b > 0);
  let month_balance_positive = status
    .and_then(|s| s.month_balance_seconds)
    .is_some_and(|b| b > 0);

  let mut class = vec!["taptime".to_owned()];
  for (enabled, name) in [
    (stale, &config.stale_class),
    (disconnected, &config.disconnected_class),
    (balance_positive, &config.positive_class),
    (!active, &config.inactive_class),
  ] {
    if enabled {
      class.push(name.clone());
    }
  }

  let text = match (disconnected, config.unavailable) {
    (true, Unavailable::Hide) => String::new(),
    (true, Unavailable::Indicator) => config.unavailable_text.clone(),
    _ => expand(&config.format),
  };
  let mut tooltip = expand(&config.tooltip);
  if disconnected {
    if !tooltip.is_empty() {
      tooltip.push('\n');
    }
    tooltip.push_str(&config.unavailable_text);
  }

  Output {
    text,
    tooltip,
    class,
    alt: if stale { "stale" } else { "ready" },
    worked: expand("{worked}"),
    balance: expand("{balance}"),
    balance_positive,
    month_balance: expand("{month_balance}"),
    month_balance_positive,
    start_time: status.and_then(|s| s.first_check_in.clone()),
    end_time: status.and_then(|s| s.last_check_out.clone()),
    active: status.map(|s| s.active),
    date: status.and_then(|s| s.date.clone()),
    seconds: status.map(Seconds::new).unwrap_or_default(),
    error: status.and_then(|s| s.error.clone()),
    disconnected,
    last_sync: status.and_then(|s| s.last_sync.clone()),
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;
  use crate::format::status;

  #[test]
  fn signed_balance_and_unknown_variables() {
    let config = DisplayConfig {
      format: "{worked} {balance} {unknown}".into(),
      ..DisplayConfig::default()
    };
    let mut value = status(json!({
      "worked_seconds": 32580,
      "balance_seconds": 1980,
      "active": true,
    }));
    let output = render(Some(&value), &config, false);
    assert_eq!(output.text, "09:03 +00:33 {unknown}");
    assert_eq!(output.worked, "09:03");
    assert_eq!(output.balance, "+00:33");
    assert!(output.balance_positive);
    assert_eq!(output.alt, "ready");
    assert_eq!(output.class, ["taptime", "positive"]);

    assert_eq!(output.seconds.worked, Some(32580));
    assert_eq!(output.seconds.remaining, None);
    assert!(!output.disconnected);

    value.month_balance_seconds = Some(2340);
    let output = render(Some(&value), &config, false);
    assert_eq!(output.month_balance, "+00:39");
    assert!(output.month_balance_positive);
  }

  #[test]
  fn disconnected() {
    let value = status(json!({"active": true}));
    let config = DisplayConfig {
      tooltip: "{date}".into(),
      ..DisplayConfig::default()
    };
    let output = render(Some(&value), &config, true);
    assert_eq!(output.alt, "stale");
    assert_eq!(output.class, ["taptime", "stale", "disconnected"]);
    assert_eq!(output.tooltip, "Daemon unavailable");

    let config = DisplayConfig {
      unavailable: Unavailable::Indicator,
      ..config
    };
    assert_eq!(render(None, &config, true).text, "Daemon unavailable");
    let config = DisplayConfig {
      unavailable: Unavailable::Hide,
      ..config
    };
    assert_eq!(render(None, &config, true).text, "");
    let output = render(None, &config, false);
    assert_eq!(output.alt, "stale");
    assert_eq!(output.active, None);
  }

  #[test]
  fn hidden_balance() {
    let config = DisplayConfig {
      show_balance: false,
      ..DisplayConfig::default()
    };
    let value = status(json!({"worked_seconds": 60, "balance_seconds": 60}));
    let output = render(Some(&value), &config, false);
    assert_eq!(output.text, "󰥔 00:01  ");
    assert_eq!(output.balance, "");
  }
}
