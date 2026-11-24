use std::{
  collections::{BTreeMap, BTreeSet},
  time::Duration,
};

use chrono::{Datelike, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use taptime_client::{ipc::Status, storage};

use crate::{Result, config::is_http, format::Formatter};

const STATE_FILE: &str = "notifications.json";
/// Delivery history older than this many days is dropped.
const HISTORY_DAYS: u64 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
  Ru,
  En,
}

impl Language {
  fn pick(self, ru: &'static str, en: &'static str) -> &'static str {
    match self {
      Self::Ru => ru,
      Self::En => en,
    }
  }
}

/// What to do with thresholds that were already passed when first observed
/// (e.g. after login or a restart).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MissedPolicy {
  /// Mark them delivered silently.
  Skip,
  /// Deliver only the most recent one.
  Latest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StalePolicy {
  /// Do not notify while the daemon reports stale data.
  Skip,
  Allow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
  Low,
  Normal,
  Critical,
}

impl From<Urgency> for notify_rust::Urgency {
  fn from(urgency: Urgency) -> Self {
    match urgency {
      Urgency::Low => Self::Low,
      Urgency::Normal => Self::Normal,
      Urgency::Critical => Self::Critical,
    }
  }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
  pub persistent: bool,
  pub timeout_ms: u32,
  pub urgency: Urgency,
  pub icon: String,
  pub sound: bool,
  pub sound_name: String,
  pub action_url: String,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      persistent: true,
      timeout_ms: 5000,
      urgency: Urgency::Normal,
      icon: "clock".into(),
      sound: false,
      sound_name: "message".into(),
      action_url: String::new(),
    }
  }
}

impl Options {
  fn validate(&self) -> Result<()> {
    if !self.action_url.is_empty() && !is_http(&self.action_url) {
      return Err("notifications: action_url must use HTTP(S)".into());
    }
    if i32::try_from(self.timeout_ms).is_err() {
      return Err("notifications: timeout_ms is too large".into());
    }
    Ok(())
  }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Threshold {
  pub id: String,
  pub remaining_minutes: i64,
  pub enabled: bool,
  pub title: String,
  pub body: String,
  pub weekdays: Vec<u32>,
  pub options: Option<Options>,
}

impl Default for Threshold {
  fn default() -> Self {
    Self {
      id: String::new(),
      remaining_minutes: 30,
      enabled: true,
      title: "TapTime".into(),
      body: String::new(),
      weekdays: (1..=7).collect(),
      options: None,
    }
  }
}

impl Threshold {
  const COMPLETION: &str = "completion";
  const OVERTIME: &str = "overtime";

  /// Identity used in the delivery history; survives edits of the text.
  fn key(&self) -> String {
    if self.id.is_empty() {
      format!("remaining:{}", self.remaining_minutes)
    } else {
      format!("custom:{}", self.id)
    }
  }

  fn builtin(id: &str, body: &str) -> Self {
    Self {
      id: id.into(),
      remaining_minutes: 0,
      body: body.into(),
      ..Self::default()
    }
  }

  /// Remaining presence (in seconds) at or below which the threshold fires.
  fn limit(&self) -> i64 {
    if self.id == Self::OVERTIME {
      -1
    } else {
      self.remaining_minutes * 60
    }
  }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationConfig {
  pub enabled: bool,
  pub language: Language,
  pub missed_policy: MissedPolicy,
  pub stale_policy: StalePolicy,
  pub notify_on_completion: bool,
  pub notify_on_overtime: bool,
  /// `[start, end)` minute-of-day ranges; `start > end` wraps past midnight.
  pub quiet_hours: Vec<[u32; 2]>,
  pub options: Options,
  pub thresholds: Vec<Threshold>,
}

impl Default for NotificationConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      language: Language::Ru,
      missed_policy: MissedPolicy::Skip,
      stale_policy: StalePolicy::Skip,
      notify_on_completion: false,
      notify_on_overtime: false,
      quiet_hours: vec![],
      options: Options::default(),
      thresholds: [240, 120, 60, 30]
        .into_iter()
        .map(|remaining_minutes| Threshold {
          remaining_minutes,
          ..Threshold::default()
        })
        .collect(),
    }
  }
}

impl NotificationConfig {
  pub fn validate(&self) -> Result<()> {
    let mut keys = BTreeSet::new();
    for threshold in &self.thresholds {
      let name = threshold.key();
      if !(0..=10080).contains(&threshold.remaining_minutes) {
        return Err(format!("notifications: {name}: remaining_minutes must be 0..=10080").into());
      }
      if threshold.weekdays.iter().any(|d| !(1..=7).contains(d)) {
        return Err(format!("notifications: {name}: weekdays must be 1..=7").into());
      }
      if !keys.insert(name.clone()) {
        return Err(format!("notifications: duplicate threshold {name}").into());
      }
    }
    std::iter::once(&self.options)
      .chain(self.thresholds.iter().filter_map(|t| t.options.as_ref()))
      .try_for_each(Options::validate)?;
    if self.quiet_hours.iter().flatten().any(|m| *m >= 1440) {
      return Err("notifications: quiet_hours use minutes 0..1439".into());
    }
    Ok(())
  }

  fn is_quiet(&self, status: &Status) -> bool {
    let Some(local) = status
      .local_time
      .as_deref()
      .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
    else {
      return false;
    };
    let minute = local.hour() * 60 + local.minute();
    self.quiet_hours.iter().any(|&[start, end]| {
      if start <= end {
        (start..end).contains(&minute)
      } else {
        minute >= start || minute < end
      }
    })
  }

  /// Configured thresholds plus the built-in completion/overtime ones.
  fn thresholds(&self, remaining: i64) -> Vec<Threshold> {
    let mut thresholds = self.thresholds.clone();
    thresholds.retain(|t| {
      (t.id != Threshold::COMPLETION || self.notify_on_completion)
        && (t.id != Threshold::OVERTIME || self.notify_on_overtime)
    });
    let has = |thresholds: &[Threshold], id| thresholds.iter().any(|t| t.id == id);
    if self.notify_on_completion && !has(&thresholds, Threshold::COMPLETION) {
      let body = self
        .language
        .pick("Дневная цель выполнена", "Daily target reached");
      thresholds.push(Threshold::builtin(Threshold::COMPLETION, body));
    }
    if self.notify_on_overtime && remaining < 0 && !has(&thresholds, Threshold::OVERTIME) {
      let body = self
        .language
        .pick("Началась переработка", "Overtime started");
      thresholds.push(Threshold::builtin(Threshold::OVERTIME, body));
    }
    thresholds
  }
}

/// Persistent record of delivered thresholds, stored in
/// `$XDG_STATE_HOME/taptime/notifications.json`.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DeliveryState {
  /// Delivered (or deliberately skipped) threshold keys per date.
  dates: BTreeMap<String, BTreeSet<String>>,
  /// `{date}:{key}` of thresholds that are due but not yet confirmed.
  pending: BTreeSet<String>,
  /// Last observed `(date, remaining seconds)`, used to detect crossings.
  #[serde(skip)]
  previous: Option<(String, i64)>,
}

impl DeliveryState {
  pub fn load() -> Self {
    storage::read(STATE_FILE)
      .and_then(|v| serde_json::from_value(v).ok())
      .unwrap_or_default()
  }

  pub fn save(&self) -> std::io::Result<()> {
    storage::write(STATE_FILE, &serde_json::to_value(self)?)
  }

  /// Thresholds that should be shown now. Each must be passed to
  /// [`Self::confirm`] after a successful delivery, otherwise it is retried.
  pub fn candidates(&mut self, status: &Status, config: &NotificationConfig) -> Vec<Threshold> {
    if status.presence_target_seconds.is_none_or(|v| v <= 0) {
      return vec![];
    }
    if !config.enabled
      || !status.active
      || (status.stale && config.stale_policy == StalePolicy::Skip)
    {
      self.previous = None;
      return vec![];
    }
    let (Some(date), Some(remaining)) = (status.date.as_deref(), Self::remaining(status)) else {
      return vec![];
    };
    self.prune(date);
    let weekday =
      NaiveDate::parse_from_str(date, "%Y-%m-%d").map_or(0, |d| d.weekday().number_from_monday());
    let quiet = config.is_quiet(status);
    let previous = self
      .previous
      .replace((date.into(), remaining))
      .filter(|(d, _)| d == date)
      .map(|(_, p)| p);

    let delivered = self.dates.entry(date.into()).or_default();
    let mut due = Vec::new();
    for threshold in config.thresholds(remaining) {
      let key = threshold.key();
      let limit = threshold.limit();
      if delivered.contains(&key) || remaining > limit {
        continue;
      }
      let pending_key = format!("{date}:{key}");
      let crossed = previous.is_some_and(|p| p > limit) || self.pending.contains(&pending_key);
      if threshold.enabled
        && threshold.weekdays.contains(&weekday)
        && !quiet
        && (crossed || config.missed_policy == MissedPolicy::Latest)
      {
        self.pending.insert(pending_key);
        due.push(threshold);
      } else {
        delivered.insert(key);
      }
    }
    if previous.is_none() && config.missed_policy == MissedPolicy::Latest {
      due.sort_by_key(|t| t.remaining_minutes);
      for threshold in due.drain(1.min(due.len())..) {
        delivered.insert(threshold.key());
      }
    }
    due
  }

  pub fn confirm(&mut self, date: &str, threshold: &Threshold) {
    let key = threshold.key();
    self.pending.remove(&format!("{date}:{key}"));
    self.dates.entry(date.into()).or_default().insert(key);
  }

  fn remaining(status: &Status) -> Option<i64> {
    let target = status.presence_target_seconds?;
    Some(target.saturating_sub(status.presence_seconds?))
  }

  fn prune(&mut self, today: &str) {
    let Some(cutoff) = NaiveDate::parse_from_str(today, "%Y-%m-%d")
      .ok()
      .and_then(|d| d.checked_sub_days(chrono::Days::new(HISTORY_DAYS)))
      .map(|d| d.format("%Y-%m-%d").to_string())
    else {
      return;
    };
    self.dates.retain(|date, _| *date >= cutoff);
    self.pending.retain(|key| *key >= cutoff);
  }
}

pub async fn send(
  threshold: &Threshold,
  config: &NotificationConfig,
  status: &Status,
) -> Result<()> {
  let options = threshold.options.as_ref().unwrap_or(&config.options);
  let body = if threshold.body.is_empty() {
    config.language.pick(
      "До дневной цели осталось {remaining}",
      "Time remaining: {remaining}",
    )
  } else {
    &threshold.body
  };
  let expand = |template: &str| Formatter::PLAIN.expand(template, Some(status));
  let mut notification = notify_rust::Notification::new();
  notification
    .appname("TapTime")
    .summary(&expand(&threshold.title))
    .body(&expand(body))
    .icon(&options.icon)
    .urgency(options.urgency.into())
    .timeout(if options.persistent {
      notify_rust::Timeout::Never
    } else {
      notify_rust::Timeout::Milliseconds(options.timeout_ms)
    })
    .hint(notify_rust::Hint::SuppressSound(!options.sound));
  if options.sound {
    notification.sound_name(&options.sound_name);
  }
  if !options.action_url.is_empty() {
    notification.action("default", "Open");
  }
  let handle = tokio::time::timeout(Duration::from_secs(5), notification.show_async()).await??;
  if !options.action_url.is_empty() {
    let url = options.action_url.clone();
    tokio::spawn(async move {
      handle
        .wait_for_action_async(|response| {
          if matches!(response, notify_rust::NotificationResponse::Default) {
            let _ = crate::open_url(&url);
          }
        })
        .await;
    });
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;
  use crate::format::status;

  fn snapshot(remaining: i64) -> Status {
    status(json!({
      "date": "2026-11-13",
      "active": true,
      "presence_target_seconds": 30600,
      "presence_seconds": 30600 - remaining,
      "local_time": "2026-11-13T12:00:00+03:00",
    }))
  }

  #[test]
  fn thresholds_retry_restart_and_edit() {
    let mut state = DeliveryState::default();
    let mut config = NotificationConfig::default();
    assert!(state.candidates(&snapshot(15000), &config).is_empty());
    for remaining in [14400, 7200, 3600, 1800] {
      let due = state.candidates(&snapshot(remaining), &config);
      assert_eq!(due.len(), 1);
      assert_eq!(state.candidates(&snapshot(remaining), &config).len(), 1);
      state.confirm("2026-11-13", &due[0]);
      state = serde_json::from_value(json!(state)).unwrap();
      assert!(state.candidates(&snapshot(remaining), &config).is_empty());
      state.previous = Some(("2026-11-13".into(), remaining));
    }
    config.thresholds.reverse();
    for t in &mut config.thresholds {
      t.body = "new text".into();
    }
    assert!(state.candidates(&snapshot(1700), &config).is_empty());
  }

  #[test]
  fn inactive_stale_disabled_and_quiet() {
    let config = NotificationConfig::default();
    let mut state = DeliveryState::default();
    state.candidates(&snapshot(15000), &config);
    let mut value = snapshot(14000);
    value.active = false;
    assert!(state.candidates(&value, &config).is_empty());

    let mut state = DeliveryState::default();
    state.candidates(&snapshot(15000), &config);
    let mut value = snapshot(14000);
    value.stale = true;
    assert!(state.candidates(&value, &config).is_empty());

    let config = NotificationConfig {
      enabled: false,
      ..config
    };
    assert!(
      DeliveryState::default()
        .candidates(&snapshot(0), &config)
        .is_empty()
    );
    let config = NotificationConfig {
      enabled: true,
      quiet_hours: vec![[0, 1439]],
      missed_policy: MissedPolicy::Latest,
      ..config
    };
    assert!(
      DeliveryState::default()
        .candidates(&snapshot(0), &config)
        .is_empty()
    );
  }

  #[test]
  fn latest_only_and_new_day() {
    let config = NotificationConfig {
      missed_policy: MissedPolicy::Latest,
      ..NotificationConfig::default()
    };
    let mut state = DeliveryState::default();
    let due = state.candidates(&snapshot(1000), &config);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].remaining_minutes, 30);
    state.confirm("2026-11-13", &due[0]);
    let mut next = snapshot(1000);
    next.date = Some("2026-11-14".into());
    assert_eq!(state.candidates(&next, &config).len(), 1);
  }

  #[test]
  fn completion_and_overtime() {
    let config = NotificationConfig {
      notify_on_completion: true,
      notify_on_overtime: true,
      thresholds: vec![],
      ..NotificationConfig::default()
    };
    let mut state = DeliveryState::default();
    assert!(state.candidates(&snapshot(60), &config).is_empty());
    let due = state.candidates(&snapshot(0), &config);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, Threshold::COMPLETION);
    state.confirm("2026-11-13", &due[0]);
    let due = state.candidates(&snapshot(-60), &config);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, Threshold::OVERTIME);
  }

  #[test]
  fn history_is_pruned() {
    let mut state = DeliveryState::default();
    state.dates.insert("2026-11-01".into(), BTreeSet::new());
    state.pending.insert("2026-11-01:remaining:30".into());
    state.dates.insert("2026-11-10".into(), BTreeSet::new());
    state.candidates(&snapshot(15000), &NotificationConfig::default());
    assert_eq!(
      state.dates.keys().collect::<Vec<_>>(),
      ["2026-11-10", "2026-11-13"]
    );
    assert!(state.pending.is_empty());
  }

  #[test]
  fn validation() {
    let mut config = NotificationConfig::default();
    config.validate().unwrap();
    config.thresholds.push(Threshold::default());
    assert!(config.validate().is_err());
    let config = NotificationConfig {
      quiet_hours: vec![[0, 1440]],
      ..NotificationConfig::default()
    };
    assert!(config.validate().is_err());
    assert!(toml::from_str::<NotificationConfig>("language = 'de'").is_err());
  }
}
