use std::{fs, io, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

use chrono::{DateTime, Timelike, Utc};
use serde_json::{Value, json};
use taptime_core::Balance;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{UnixListener, UnixStream},
  sync::watch,
};

use crate::{ApiError, TapTimeClient, TodaySnapshot, load_session};

pub fn runtime_dir() -> io::Result<PathBuf> {
  let root = std::env::var_os("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .filter(|p| p.is_absolute())
    .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR must be absolute"))?;
  let path = root.join("taptime");
  match fs::create_dir(&path) {
    Ok(()) => fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?,
    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
    Err(e) => return Err(e),
  }
  use std::os::unix::fs::MetadataExt;
  let meta = fs::symlink_metadata(&path)?;
  let owner = fs::metadata("/proc/self")?.uid();
  if !meta.is_dir() || meta.uid() != owner || meta.permissions().mode() & 0o077 != 0 {
    return Err(io::Error::other("unsafe runtime directory"));
  }
  Ok(path)
}

#[derive(Default)]
pub struct BackendState {
  pub snapshot: Option<TodaySnapshot>,
  pub last_sync: Option<DateTime<Utc>>,
  pub error: Option<&'static str>,
}

impl BackendState {
  pub fn json_at(&self, now: DateTime<Utc>) -> Value {
    let mut value = json!({
      "version": 1,
      "sampled_at": now.to_rfc3339(),
      "last_sync": self.last_sync.map(|v| v.to_rfc3339()),
      "stale": true,
      "error": self.error,
      "date": null,
      "worked_seconds": null,
      "presence_seconds": null,
      "work_target_seconds": null,
      "presence_target_seconds": null,
      "balance_seconds": null,
      "active": false
    });
    if let Some(s) = &self.snapshot {
      let same_day = now.with_timezone(&s.time_zone).date_naive() == s.date;
      let active = same_day && s.is_active();
      let presence = if active {
        s.day
          .first_check_in()
          .map(|start| {
            let current = i64::from(now.with_timezone(&s.time_zone).num_seconds_from_midnight());
            (current - i64::from(start.hour * 3600 + start.minute * 60 + start.second)).max(0)
          })
          .unwrap_or(0)
      } else {
        s.day.presence_duration().unwrap_or_default().num_seconds()
      };
      let balance = match s.balance_at(now) {
        Balance::Exact => 0,
        Balance::Overtime(d) => d.num_seconds(),
        Balance::UnderTime(d) => -d.num_seconds(),
      };
      value["date"] = json!(s.date.to_string());
      value["worked_seconds"] = json!(s.worked_at(now).as_secs());
      value["presence_seconds"] = json!(presence);
      value["work_target_seconds"] = json!(s.work_target.as_secs());
      value["presence_target_seconds"] = json!(s.day.required_day_duration().num_seconds());
      value["balance_seconds"] = json!(balance);
      value["active"] = json!(active);
      value["stale"] = json!(
        !same_day
          || self.error.is_some()
          || self.last_sync.is_none_or(|t| (now - t).num_seconds() > 90)
      );
    }
    value
  }
}

pub async fn status() -> io::Result<Value> {
  let socket = UnixStream::connect(runtime_dir()?.join("status.sock")).await?;
  let mut data = Vec::new();
  tokio::time::timeout(
    Duration::from_secs(3),
    socket.take(65536).read_to_end(&mut data),
  )
  .await
  .map_err(io::Error::other)??;
  serde_json::from_slice(&data).map_err(io::Error::other)
}

pub async fn run_daemon() -> io::Result<()> {
  let dir = runtime_dir()?;
  let lock = fs::OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .open(dir.join("daemon.lock"))?;
  lock.try_lock().map_err(io::Error::other)?;
  let path = dir.join("status.sock");
  match fs::remove_file(&path) {
    Ok(()) => {}
    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
    Err(e) => return Err(e),
  }
  let listener = UnixListener::bind(&path)?;
  fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
  let (tx, rx) = watch::channel(BackendState::default());
  let client = TapTimeClient::production().map_err(io::Error::other)?;
  let worker = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
      interval.tick().await;
      let result = match load_session() {
        Ok(Some(session)) => client.get_today(&session).await,
        Ok(None) => Err(ApiError::Unauthenticated),
        Err(_) => {
          tx.send_modify(|s| s.error = Some("session_unavailable"));
          continue;
        }
      };
      tx.send_modify(|state| match result {
        Ok(snapshot) => {
          state.snapshot = Some(snapshot);
          state.last_sync = Some(Utc::now());
          state.error = None;
        }
        Err(ApiError::Unauthenticated) => {
          state.snapshot = None;
          state.last_sync = None;
          state.error = Some("unauthenticated");
        }
        Err(_) => state.error = Some("api_unavailable"),
      });
    }
  });
  let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
  loop {
    tokio::select! {
      _ = terminate.recv() => break,
      _ = tokio::signal::ctrl_c() => break,
      incoming = listener.accept() => {
        let (mut stream, _) = incoming?;
        let mut body = rx.borrow().json_at(Utc::now()).to_string();
        body.push('\n');
        tokio::spawn(async move {
          let _ = tokio::time::timeout(Duration::from_secs(2), stream.write_all(body.as_bytes())).await;
        });
      }
    }
  }
  worker.abort();
  let _ = worker.await;
  fs::remove_file(path)?;
  drop(lock);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_state_is_stale_and_has_no_counters() {
    let state = BackendState::default().json_at(Utc::now());
    assert_eq!(state["stale"], true);
    assert!(state["worked_seconds"].is_null());
    assert_eq!(state["active"], false);
  }
}
