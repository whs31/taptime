use std::{io::Write, process::ExitCode, time::Duration};

use taptime_waybar::{Result, config::Config, exit_code, open_url, render::render};

const USAGE: &str = "\
usage: taptime-waybar [COMMAND]

commands:
  once    print one Waybar JSON line (default)
  watch   print a JSON line every display.interval_ms
  open    open display.dashboard_url in the browser
  config  validate the config file and print the effective settings";

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
  exit_code("taptime-waybar", run_command().await)
}

async fn run_command() -> Result<()> {
  let command = std::env::args().nth(1);
  let command = command.as_deref().unwrap_or("once");
  if matches!(command, "help" | "-h" | "--help") {
    println!("{USAGE}");
    return Ok(());
  }

  let config = Config::load()?;
  match command {
    "config" => {
      config.validate()?;
      print!("{}", toml::to_string_pretty(&config)?);
      Ok(())
    }
    "open" => {
      config.display.validate()?;
      Ok(open_url(&config.display.dashboard_url)?)
    }
    "once" | "watch" => {
      config.display.validate()?;
      stream(&config, command == "watch").await
    }
    other => Err(format!("unknown command {other:?}\n\n{USAGE}").into()),
  }
}

async fn stream(config: &Config, watch: bool) -> Result<()> {
  let mut ticker = tokio::time::interval(Duration::from_millis(config.display.interval_ms));
  ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  let mut last = None;
  let mut stdout = std::io::stdout().lock();
  loop {
    ticker.tick().await;
    let disconnected = match taptime_client::ipc::snapshot().await {
      Ok(status) => {
        last = Some(status);
        false
      }
      Err(_) => true,
    };
    let output = render(last.as_ref(), &config.display, disconnected);
    serde_json::to_writer(&mut stdout, &output)?;
    writeln!(stdout)?;
    stdout.flush()?;
    if !watch {
      return Ok(());
    }
  }
}
