use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use taptime_client::storage;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
  pub persistent: bool,
  pub timeout_ms: u32,
  pub urgency: String,
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
      urgency: "normal".into(),
      icon: "clock".into(),
      sound: false,
      sound_name: "message".into(),
      action_url: String::new(),
    }
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
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
  pub enabled: bool,
  pub language: String,
  pub missed_policy: String,
  pub stale_policy: String,
  pub notify_on_completion: bool,
  pub notify_on_overtime: bool,
  pub quiet_hours: Vec<[u32; 2]>,
  pub options: Options,
  pub thresholds: Vec<Threshold>,
}
impl Default for Config {
  fn default() -> Self {
    Self {
      enabled: true,
      language: "ru".into(),
      missed_policy: "skip".into(),
      stale_policy: "skip".into(),
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
impl Config {
  pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let config = crate::config::Document::load()?.notifications;
    let mut ids = BTreeSet::new();
    if !["skip", "latest"].contains(&config.missed_policy.as_str())
      || !["skip", "allow"].contains(&config.stale_policy.as_str())
      || !["ru", "en"].contains(&config.language.as_str())
    {
      return Err("invalid notification policy or language".into());
    }
    for threshold in &config.thresholds {
      if !(0..=10080).contains(&threshold.remaining_minutes)
        || !ids.insert(threshold.key())
        || threshold.weekdays.iter().any(|d| !(1..=7).contains(d))
      {
        return Err("invalid or duplicate threshold".into());
      }
    }
    for options in std::iter::once(&config.options)
      .chain(config.thresholds.iter().filter_map(|t| t.options.as_ref()))
    {
      if !options.action_url.is_empty()
        && !options.action_url.starts_with("https://")
        && !options.action_url.starts_with("http://")
      {
        return Err("action_url must use HTTP(S)".into());
      }
      if !["low", "normal", "critical"].contains(&options.urgency.as_str())
        || options.timeout_ms > i32::MAX as u32
      {
        return Err("invalid notification options".into());
      }
    }
    if config.quiet_hours.iter().flatten().any(|m| *m >= 1440) {
      return Err("quiet hours use minutes 0..1439".into());
    }
    Ok(config)
  }
}
impl Threshold {
  fn key(&self) -> String {
    if self.id.is_empty() {
      format!("remaining:{}", self.remaining_minutes)
    } else {
      format!("custom:{}", self.id)
    }
  }
}
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DeliveryState {
  dates: BTreeMap<String, BTreeSet<String>>,
  #[serde(skip)]
  previous: Option<(String, i64)>,
  pending: BTreeSet<String>,
}
impl DeliveryState {
  pub fn load() -> Self {
    storage::read("notifications.json")
      .and_then(|v| serde_json::from_value(v).ok())
      .unwrap_or_default()
  }
  pub fn candidates(&mut self, value: &Value, config: &Config) -> Vec<Threshold> {
    if value["presence_target_seconds"]
      .as_i64()
      .is_none_or(|v| v <= 0)
    {
      return vec![];
    }
    if !config.enabled
      || value["active"] != true
      || (value["stale"] != false && config.stale_policy == "skip")
    {
      self.previous = None;
      return vec![];
    }
    let Some(date) = value["date"].as_str() else {
      return vec![];
    };
    let Some(remaining) = value["presence_target_seconds"]
      .as_i64()
      .zip(value["presence_seconds"].as_i64())
      .map(|(a, b)| a.saturating_sub(b))
    else {
      return vec![];
    };
    let local = value["local_time"]
      .as_str()
      .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
    let weekday = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
      .ok()
      .map(|d| d.weekday().number_from_monday())
      .unwrap_or(0);
    let quiet = local.is_some_and(|d| {
      let minute = d.hour() * 60 + d.minute();
      config.quiet_hours.iter().any(|[start, end]| {
        if start <= end {
          minute >= *start && minute < *end
        } else {
          minute >= *start || minute < *end
        }
      })
    });
    let previous = self
      .previous
      .as_ref()
      .filter(|(d, _)| d == date)
      .map(|(_, p)| *p);
    self.previous = Some((date.into(), remaining));
    let delivered = self.dates.entry(date.into()).or_default();
    let mut thresholds = config.thresholds.clone();
    thresholds.retain(|t| {
      (t.id != "completion" || config.notify_on_completion)
        && (t.id != "overtime" || config.notify_on_overtime)
    });
    if config.notify_on_completion && !thresholds.iter().any(|t| t.id == "completion") {
      thresholds.push(Threshold {
        id: "completion".into(),
        remaining_minutes: 0,
        body: if config.language == "ru" {
          "Дневная цель выполнена"
        } else {
          "Daily target reached"
        }
        .into(),
        ..Threshold::default()
      });
    }
    if config.notify_on_overtime && remaining < 0 && !thresholds.iter().any(|t| t.id == "overtime")
    {
      thresholds.push(Threshold {
        id: "overtime".into(),
        remaining_minutes: 0,
        body: if config.language == "ru" {
          "Началась переработка"
        } else {
          "Overtime started"
        }
        .into(),
        ..Threshold::default()
      });
    }
    let mut due = Vec::new();
    for threshold in thresholds {
      let limit = if threshold.id == "overtime" {
        -1
      } else {
        threshold.remaining_minutes * 60
      };
      if delivered.contains(&threshold.key()) {
        continue;
      }
      if remaining <= limit {
        let pending_key = format!("{date}:{}", threshold.key());
        let crossed = previous.is_some_and(|p| p > limit) || self.pending.contains(&pending_key);
        if threshold.enabled
          && threshold.weekdays.contains(&weekday)
          && !quiet
          && (crossed || config.missed_policy == "latest")
        {
          self.pending.insert(pending_key);
          due.push(threshold);
        } else {
          delivered.insert(threshold.key());
        }
      }
    }
    if previous.is_none() && config.missed_policy == "latest" {
      due.sort_by_key(|t| t.remaining_minutes);
      for threshold in due.iter().skip(1) {
        delivered.insert(threshold.key());
      }
      due.truncate(1);
    }
    due
  }
  pub fn confirm(&mut self, date: &str, threshold: &Threshold) {
    self.pending.remove(&format!("{date}:{}", threshold.key()));
    self
      .dates
      .entry(date.into())
      .or_default()
      .insert(threshold.key());
  }
  pub fn save(&self) -> std::io::Result<()> {
    storage::write("notifications.json", &json!(self))
  }
}

pub async fn send(
  threshold: &Threshold,
  config: &Config,
  value: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let options = threshold.options.as_ref().unwrap_or(&config.options);
  let body = if threshold.body.is_empty() {
    if config.language == "ru" {
      "До дневной цели осталось {remaining}"
    } else {
      "Time remaining: {remaining}"
    }
  } else {
    &threshold.body
  };
  let format = |s: &str| crate::format::format(value, s, "", false, "+{value}", "-{value}");
  let mut notification = notify_rust::Notification::new();
  notification
    .appname("TapTime")
    .summary(&format(&threshold.title))
    .body(&format(body))
    .icon(&options.icon)
    .urgency(match options.urgency.as_str() {
      "low" => notify_rust::Urgency::Low,
      "critical" => notify_rust::Urgency::Critical,
      _ => notify_rust::Urgency::Normal,
    })
    .timeout(if options.persistent {
      0
    } else {
      options.timeout_ms as i32
    })
    .hint(notify_rust::Hint::SuppressSound(!options.sound));
  if options.sound {
    notification.sound_name(&options.sound_name);
  }
  if !options.action_url.is_empty() {
    notification.action("default", "Open");
  }
  let handle =
    tokio::time::timeout(std::time::Duration::from_secs(5), notification.show_async()).await??;
  if !options.action_url.is_empty() {
    let url = options.action_url.clone();
    tokio::spawn(async move {
      handle
        .wait_for_action_async(|response| {
          if matches!(response, notify_rust::NotificationResponse::Default) {
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
          }
        })
        .await;
    });
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  fn snapshot(remaining: i64) -> Value {
    json!({"date":"2026-11-13", "active":true, "stale":false, "presence_target_seconds":30600,"presence_seconds":30600-remaining,"local_time":"2026-11-13T12:00:00+03:00"})
  }
  #[test]
  fn thresholds_retry_restart_and_edit() {
    let mut state = DeliveryState::default();
    let mut config = Config::default();
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
    let config = Config::default();
    for key in ["active", "stale"] {
      let mut state = DeliveryState::default();
      state.candidates(&snapshot(15000), &config);
      let mut value = snapshot(14000);
      value[key] = json!(key == "stale");
      assert!(state.candidates(&value, &config).is_empty());
    }
    let config = Config {
      enabled: false,
      ..config
    };
    assert!(
      DeliveryState::default()
        .candidates(&snapshot(0), &config)
        .is_empty()
    );
    let config = Config {
      enabled: true,
      quiet_hours: vec![[0, 1439]],
      missed_policy: "latest".into(),
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
    let config = Config {
      missed_policy: "latest".into(),
      ..Config::default()
    };
    let mut state = DeliveryState::default();
    let due = state.candidates(&snapshot(1000), &config);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].remaining_minutes, 30);
    state.confirm("2026-11-13", &due[0]);
    let mut next = snapshot(1000);
    next["date"] = json!("2026-11-14");
    assert_eq!(state.candidates(&next, &config).len(), 1);
  }
}
