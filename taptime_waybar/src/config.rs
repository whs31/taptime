use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Result, format::Clock, notifications::NotificationConfig};

/// The whole `config.toml` document shared by both binaries.
#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
  pub display: DisplayConfig,
  pub notifications: NotificationConfig,
}

impl Config {
  /// `$XDG_CONFIG_HOME/taptime-waybar/config.toml`.
  pub fn path() -> Result<PathBuf> {
    let root = match std::env::var_os("XDG_CONFIG_HOME") {
      Some(path) => PathBuf::from(path),
      None => std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config"))
        .ok_or("HOME or XDG_CONFIG_HOME is required")?,
    };
    if !root.is_absolute() {
      return Err("XDG_CONFIG_HOME must be absolute".into());
    }
    Ok(root.join("taptime-waybar/config.toml"))
  }

  /// Reads the config file; a missing file yields the defaults. Sections are
  /// not validated here, so each binary only rejects the settings it uses.
  pub fn load() -> Result<Self> {
    let path = Self::path()?;
    match fs::read_to_string(&path) {
      Ok(contents) => {
        toml::from_str(&contents).map_err(|error| format!("{}: {error}", path.display()).into())
      }
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
      Err(error) => Err(format!("{}: {error}", path.display()).into()),
    }
  }

  pub fn validate(&self) -> Result<()> {
    self.display.validate()?;
    self.notifications.validate()
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unavailable {
  /// Keep showing the last known status.
  Last,
  /// Print an empty module.
  Hide,
  /// Print `unavailable_text`.
  Indicator,
}

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplayConfig {
  pub format: String,
  pub icon: String,
  pub tooltip: String,
  #[serde(alias = "refresh_ms")]
  pub interval_ms: u64,
  pub dashboard_url: String,
  #[serde(alias = "show_seconds")]
  pub seconds: bool,
  pub hours: bool,
  pub minutes: bool,
  pub show_balance: bool,
  pub unavailable: Unavailable,
  pub unavailable_text: String,
  pub active_text: String,
  pub inactive_text: String,
  pub stale_class: String,
  pub positive_class: String,
  pub inactive_class: String,
  pub disconnected_class: String,
  pub positive: String,
  pub negative: String,
}

impl Default for DisplayConfig {
  fn default() -> Self {
    Self {
      format: "{icon} {worked}  {balance}".into(),
      icon: "󰥔".into(),
      tooltip: "{date}\n{worked} / {work_target}\n{presence} / {presence_target}".into(),
      interval_ms: 1000,
      dashboard_url: "https://time.whs31.com/dashboard".into(),
      seconds: false,
      hours: true,
      minutes: true,
      show_balance: true,
      unavailable: Unavailable::Last,
      unavailable_text: "Daemon unavailable".into(),
      active_text: "true".into(),
      inactive_text: "false".into(),
      stale_class: "stale".into(),
      positive_class: "positive".into(),
      inactive_class: "inactive".into(),
      disconnected_class: "disconnected".into(),
      positive: "+{value}".into(),
      negative: "-{value}".into(),
    }
  }
}

impl DisplayConfig {
  pub fn clock(&self) -> Clock {
    Clock {
      hours: self.hours,
      minutes: self.minutes,
      seconds: self.seconds,
    }
  }

  pub fn validate(&self) -> Result<()> {
    if !(100..=60000).contains(&self.interval_ms) {
      return Err("display.interval_ms must be between 100 and 60000".into());
    }
    if !(self.hours || self.minutes || self.seconds) {
      return Err("display: at least one of hours, minutes, seconds must be enabled".into());
    }
    if !is_http(&self.dashboard_url) {
      return Err("display.dashboard_url must use HTTP(S)".into());
    }
    Ok(())
  }
}

pub(crate) fn is_http(url: &str) -> bool {
  url.starts_with("https://") || url.starts_with("http://")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn optional_settings_and_unknown_keys() {
    let display: DisplayConfig = toml::from_str("seconds = true\nicon = ''").unwrap();
    assert!(display.seconds);
    assert!(display.icon.is_empty());
    assert_eq!(display.interval_ms, 1000);
    assert!(toml::from_str::<DisplayConfig>("unknown = true").is_err());
    assert!(toml::from_str::<DisplayConfig>("interval_ms = 'bad'").is_err());
    assert!(toml::from_str::<DisplayConfig>("unavailable = 'nope'").is_err());
    assert!(
      toml::from_str::<DisplayConfig>("show_seconds = true")
        .unwrap()
        .seconds
    );

    let config: Config =
      toml::from_str("[display]\ninterval_ms = 2000\n[notifications]\nenabled = false").unwrap();
    assert_eq!(config.display.interval_ms, 2000);
    assert!(!config.notifications.enabled);
    assert!(toml::from_str::<Config>("[display]\nunknown = true").is_err());
  }

  #[test]
  fn example_is_valid() {
    let example: Config = toml::from_str(include_str!("../examples/config.toml")).unwrap();
    example.validate().unwrap();
    assert_eq!(example.notifications.thresholds.len(), 4);
  }

  #[test]
  fn validation() {
    let mut display = DisplayConfig::default();
    display.validate().unwrap();
    display.hours = false;
    display.minutes = false;
    assert!(display.validate().is_err());
    let display = DisplayConfig {
      dashboard_url: "file:///etc/passwd".into(),
      ..DisplayConfig::default()
    };
    assert!(display.validate().is_err());
  }
}
