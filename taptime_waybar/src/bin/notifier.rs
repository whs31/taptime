use std::{fs, process::ExitCode, time::Duration};

use taptime_waybar::{
  Result,
  config::Config,
  exit_code,
  notifications::{DeliveryState, NotificationConfig, send},
};

const USAGE: &str = "\
usage: taptime-notifier [COMMAND]

Without a command, runs the notification loop (one instance per user).

commands:
  config  validate the [notifications] section and print the effective settings";

const POLL: Duration = Duration::from_secs(5);

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
  exit_code("taptime-notifier", run_command().await)
}

async fn run_command() -> Result<()> {
  match std::env::args().nth(1).as_deref() {
    None => run().await,
    Some("config") => {
      let config = Config::load()?.notifications;
      config.validate()?;
      print!("{}", toml::to_string_pretty(&config)?);
      Ok(())
    }
    Some("help" | "-h" | "--help") => {
      println!("{USAGE}");
      Ok(())
    }
    Some(other) => Err(format!("unknown command {other:?}\n\n{USAGE}").into()),
  }
}

async fn run() -> Result<()> {
  let lock = fs::OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .open(taptime_client::ipc::runtime_dir()?.join("notifier.lock"))?;
  lock
    .try_lock()
    .map_err(|_| "another taptime-notifier is already running")?;

  let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
  tokio::select! {
    result = poll() => result,
    _ = terminate.recv() => Ok(()),
    _ = tokio::signal::ctrl_c() => Ok(()),
  }
}

async fn poll() -> Result<()> {
  let mut state = DeliveryState::load();
  let mut timer = tokio::time::interval(POLL);
  timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  // Only log a config error when it changes, not on every tick.
  let mut config_error = None;
  loop {
    timer.tick().await;
    // Reloaded every tick so edits apply without a restart.
    let config = match load_config() {
      Ok(config) => {
        config_error = None;
        config
      }
      Err(error) => {
        let error = error.to_string();
        if config_error.as_ref() != Some(&error) {
          eprintln!("invalid config, notifications paused: {error}");
          config_error = Some(error);
        }
        continue;
      }
    };
    let Ok(status) = taptime_client::ipc::snapshot().await else {
      continue;
    };
    let before = serde_json::to_value(&state)?;
    for threshold in state.candidates(&status, &config) {
      match send(&threshold, &config, &status).await {
        Ok(()) => state.confirm(status.date.as_deref().unwrap_or_default(), &threshold),
        Err(error) => eprintln!("failed to show notification: {error}"),
      }
    }
    if serde_json::to_value(&state)? != before {
      state.save()?;
    }
  }
}

fn load_config() -> Result<NotificationConfig> {
  let config = Config::load()?.notifications;
  config.validate()?;
  Ok(config)
}
