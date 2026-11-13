use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
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
  pub unavailable: String,
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

impl Default for Config {
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
      unavailable: "last".into(),
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

#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Document {
  pub display: Config,
  pub notifications: crate::notifications::Config,
}

impl Document {
  pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let root = if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
      PathBuf::from(path)
    } else {
      let home = std::env::var_os("HOME")
        .ok_or_else(|| io::Error::other("HOME or XDG_CONFIG_HOME is required"))?;
      PathBuf::from(home).join(".config")
    };
    if !root.is_absolute() {
      return Err(io::Error::other("XDG_CONFIG_HOME must be absolute").into());
    }
    let path = root.join("taptime-waybar/config.toml");
    match fs::read_to_string(path) {
      Ok(contents) => Ok(toml::from_str(&contents)?),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
      Err(error) => Err(error.into()),
    }
  }
}

impl Config {
  pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let config = Document::load()?.display;
    if !(100..=60000).contains(&config.interval_ms) {
      return Err("interval_ms must be between 100 and 60000".into());
    }
    if !["last", "hide", "indicator"].contains(&config.unavailable.as_str())
      || !(config.hours || config.minutes || config.seconds)
    {
      return Err("invalid display settings".into());
    }
    if !config.dashboard_url.starts_with("https://") && !config.dashboard_url.starts_with("http://")
    {
      return Err("dashboard_url must use HTTP(S)".into());
    }
    Ok(config)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn optional_settings_and_unknown_keys() {
    let config: Config = toml::from_str("seconds = true\nicon = ''").unwrap();
    assert!(config.seconds);
    assert!(config.icon.is_empty());
    assert_eq!(config.interval_ms, 1000);
    assert!(toml::from_str::<Config>("unknown = true").is_err());
    assert!(toml::from_str::<Config>("interval_ms = 'bad'").is_err());
    let document: Document =
      toml::from_str("[display]\ninterval_ms = 2000\n[notifications]\nenabled = false").unwrap();
    assert_eq!(document.display.interval_ms, 2000);
    assert!(!document.notifications.enabled);
    let example: Document = toml::from_str(include_str!("../examples/config.toml")).unwrap();
    assert_eq!(example.notifications.thresholds.len(), 4);
    assert!(toml::from_str::<Document>("[display]\nunknown = true").is_err());
  }
}
