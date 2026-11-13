use std::{fs, io, time::Duration};

use taptime_waybar::notifications::{Config, DeliveryState, send};
#[tokio::main]
async fn main() -> io::Result<()> {
  match std::env::args().nth(1).as_deref() {
    Some("config") => {
      let config = Config::load().map_err(io::Error::other)?;
      println!(
        "{}",
        toml::to_string_pretty(&config).map_err(io::Error::other)?
      );
      return Ok(());
    }
    Some("--help") => {
      println!("taptime-notifier [config]");
      return Ok(());
    }
    Some(_) => return Err(io::Error::other("usage: taptime-notifier [config]")),
    None => {}
  }
  let dir = taptime_client::ipc::runtime_dir()?;
  let lock = fs::OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .open(dir.join("notifier.lock"))?;
  lock.try_lock().map_err(io::Error::other)?;
  let mut state = DeliveryState::load();
  let mut timer = tokio::time::interval(Duration::from_secs(5));
  timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
  let work = async {
    loop {
      timer.tick().await;
      let Ok(config) = Config::load() else {
        continue;
      };
      let value = taptime_client::ipc::status()
        .await
        .unwrap_or_else(|_| serde_json::json!({"stale":true,"active":false}));
      let before = serde_json::to_value(&state).ok();
      for threshold in state.candidates(&value, &config) {
        if send(&threshold, &config, &value).await.is_ok() {
          state.confirm(value["date"].as_str().unwrap_or_default(), &threshold);
          state.save()?;
        }
      }
      if serde_json::to_value(&state).ok() != before {
        state.save()?;
      }
    }
  };
  tokio::select! {
    result = work => result,
    _ = terminate.recv() => Ok(()),
    _ = tokio::signal::ctrl_c() => Ok(()),
  }
}
