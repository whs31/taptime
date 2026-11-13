use taptime_waybar::config;
mod render;

use std::{io::Write, time::Duration};

use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let config = config::Config::load()?;
  let command = std::env::args().nth(1).unwrap_or_else(|| "once".into());
  if command == "open" {
    std::process::Command::new("xdg-open")
      .arg(&config.dashboard_url)
      .spawn()?;
    return Ok(());
  }
  if command == "config" {
    println!("{}", toml::to_string_pretty(&config)?);
    return Ok(());
  }
  if command != "once" && command != "watch" {
    return Err("usage: taptime-waybar once|watch|config".into());
  }
  let mut last = Value::Null;
  let mut stdout = std::io::stdout().lock();
  loop {
    let disconnected = match taptime_client::ipc::status().await {
      Ok(value) => {
        last = value;
        false
      }
      Err(_) => true,
    };
    writeln!(stdout, "{}", render::render(&last, &config, disconnected))?;
    stdout.flush()?;
    if command == "once" {
      break;
    }
    std::thread::sleep(Duration::from_millis(config.interval_ms));
  }
  Ok(())
}
