use std::{fs, io, os::unix::fs::PermissionsExt, time::Duration};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use taptime_core::Balance;
use tokio::{io::AsyncWriteExt, net::UnixListener, sync::watch};

pub use crate::ipc::runtime_dir;
use crate::{ApiError, TapTimeClient, TodaySnapshot, load_session};

#[derive(Default)]
pub struct BackendState {
  pub snapshot: Option<TodaySnapshot>,
  pub last_sync: Option<DateTime<Utc>>,
  pub cached: Option<Value>,
  pub sampled: Option<std::time::Instant>,
  pub error: Option<&'static str>,
}

impl BackendState {
  pub fn json_at(&self, now: DateTime<Utc>) -> Value {
    let mut value = self
      .cached
      .clone()
      .filter(|v| v.is_object() && v["version"] == 1)
      .unwrap_or_else(|| {
        json!({
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
          "month_balance_seconds": null,
          "first_check_in": null,
          "last_check_out": null,
          "active": false
        })
      });
    value["stale"] = json!(true);
    value["active"] = json!(false);
    value["error"] = json!(self.error);
    if let Some(s) = &self.snapshot {
      let same_day = now.with_timezone(&s.time_zone).date_naive() == s.date;
      let elapsed = self.sampled.map(|t| t.elapsed()).unwrap_or_default();
      let measured_at = self.last_sync.map_or(now, |last| {
        last + chrono::Duration::seconds(elapsed.as_secs().min(90) as i64)
      });
      let measured_at = if measured_at.with_timezone(&s.time_zone).date_naive() != s.date {
        s.date
          .succ_opt()
          .and_then(|date| date.and_hms_opt(0, 0, 0))
          .and_then(|end| s.time_zone.from_local_datetime(&end).earliest())
          .map(|end| end.with_timezone(&Utc) - chrono::Duration::seconds(1))
          .unwrap_or(measured_at)
      } else {
        measured_at
      };
      value["local_time"] = json!(now.with_timezone(&s.time_zone).to_rfc3339());
      let active = same_day && s.is_active();
      let presence = s.presence_at(measured_at);
      let balance = match s.balance_at(measured_at) {
        Balance::Exact => 0,
        Balance::Overtime(d) => d.num_seconds(),
        Balance::UnderTime(d) => -d.num_seconds(),
      };
      value["date"] = json!(s.date.to_string());
      value["worked_seconds"] = json!(s.worked_at(measured_at).as_secs());
      value["presence_seconds"] = json!(presence);
      value["work_target_seconds"] = json!(s.work_target.as_secs());
      value["presence_target_seconds"] = json!(s.day.required_day_duration().num_seconds());
      value["balance_seconds"] = json!(balance);
      value["active"] = json!(active);
      value["month_balance_seconds"] = json!(s.month_balance_at(measured_at));
      value["first_check_in"] = json!(s.first_check_in_text());
      value["last_check_out"] = json!(s.last_check_out_text());
      value["stale"] = json!(
        !same_day || self.error.is_some() || self.last_sync.is_none() || elapsed.as_secs() > 90
      );
    }
    value
  }
}

pub use crate::ipc::status;

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
  let mut initial = BackendState::default();
  if load_session().ok().flatten().is_some() {
    initial.cached = crate::storage::read("status.json");
  }
  let (tx, rx) = watch::channel(initial);
  let client = TapTimeClient::production().map_err(io::Error::other)?;
  let worker = tokio::spawn(async move {
    let mut current_token = load_session().ok().flatten().map(|s| s.token().to_owned());
    let mut next_sync = tokio::time::Instant::now();
    let mut pending = tokio::task::JoinSet::new();
    let mut poll = tokio::time::interval(Duration::from_secs(1));
    poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
      poll.tick().await;
      let session = load_session();
      let token = session
        .as_ref()
        .ok()
        .and_then(|s| s.as_ref())
        .map(|s| s.token().to_owned());
      if token != current_token || (token.is_none() && tx.borrow().error.is_none()) {
        pending.abort_all();
        while pending.join_next().await.is_some() {}
        tx.send_modify(|state| {
          *state = BackendState::default();
          state.error = if session.is_err() {
            Some("session_unavailable")
          } else {
            Some("unauthenticated")
          };
        });
        let _ = crate::storage::write("status.json", &json!(null));
        current_token = token;
        next_sync = tokio::time::Instant::now();
      }
      if let Some(result) = pending.try_join_next() {
        tx.send_modify(|state| match result {
          Ok(Ok(snapshot)) => {
            state.snapshot = Some(snapshot);
            state.last_sync = Some(Utc::now());
            state.sampled = Some(std::time::Instant::now());
            state.cached = None;
            state.error = None;
          }
          Ok(Err(ApiError::Unauthenticated)) => {
            *state = BackendState::default();
            state.error = Some("unauthenticated");
          }
          _ => state.error = Some("api_unavailable"),
        });
        let value = tx.borrow().json_at(Utc::now());
        if value["error"] == "unauthenticated" {
          let _ = crate::storage::write("status.json", &json!(null));
        }
        if value["stale"] == false {
          let _ = crate::storage::write("status.json", &value);
        }
      }
      if pending.is_empty()
        && tokio::time::Instant::now() >= next_sync
        && let Ok(Some(session)) = session
      {
        let client = client.clone();
        pending.spawn(async move { client.get_today(&session).await });
        next_sync = tokio::time::Instant::now() + Duration::from_secs(60);
      }
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
  fn extrapolation_is_bounded_and_new_day_is_inactive() {
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
    let last = date.and_hms_opt(12, 0, 0).unwrap().and_utc();
    let snapshot = TodaySnapshot {
      date,
      time_zone: chrono_tz::UTC,
      day: taptime_core::Day {
        date,
        events: vec![taptime_core::Event::CheckIn(taptime_core::LocalTime {
          hour: 8,
          minute: 0,
          second: 0,
        })],
        flags: taptime_core::DayFlags::empty(),
        required_work_hours: chrono::Duration::hours(8),
        lunch_break_duration: chrono::Duration::minutes(30),
      },
      clocked_work: Duration::ZERO,
      work_target: Duration::from_secs(28800),
      server_balance: Balance::Exact,
      month_balance_closed: 0,
    };
    let state = BackendState {
      snapshot: Some(snapshot),
      last_sync: Some(last),
      sampled: Some(std::time::Instant::now() - Duration::from_secs(3600)),
      ..BackendState::default()
    };
    let value = state.json_at(last + chrono::Duration::hours(1));
    assert_eq!(value["worked_seconds"], 14490);
    assert_eq!(value["stale"], true);
    let next = state.json_at(last + chrono::Duration::days(1));
    assert_eq!(next["active"], false);
    assert_eq!(next["worked_seconds"], 14490);
    assert_eq!(value["presence_target_seconds"], 30600);
  }

  #[test]
  fn invalid_cache_and_cached_state_are_safe() {
    for cached in [json!(null), json!([]), json!(42), json!({"version":99})] {
      let state = BackendState {
        cached: Some(cached),
        ..BackendState::default()
      }
      .json_at(Utc::now());
      assert_eq!(state["version"], 1);
      assert_eq!(state["stale"], true);
    }
    let cached = json!({"version":1,"active":true,"stale":false,"worked_seconds":123});
    let state = BackendState {
      cached: Some(cached),
      ..BackendState::default()
    };
    assert_eq!(state.json_at(Utc::now())["worked_seconds"], 123);
    assert_eq!(state.json_at(Utc::now())["active"], false);
    assert_eq!(state.json_at(Utc::now())["stale"], true);
  }

  #[test]
  fn empty_state_is_stale_and_has_no_counters() {
    let state = BackendState::default().json_at(Utc::now());
    assert_eq!(state["stale"], true);
    assert!(state["worked_seconds"].is_null());
    assert_eq!(state["active"], false);
  }
}
