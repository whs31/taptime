use std::time::Duration;

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use prost::Message;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use taptime_core::{Balance, Day, DayFlags, Event};
use taptime_schema::{
  Date, User,
  services::{AuthResponse, DashboardRequest, DashboardResponse, LoginRequest},
};
use thiserror::Error;

use crate::auth::{LoginResult, Session};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_MESSAGE_SIZE: usize = 8 * 1024 * 1024;
const GRPC_WEB_CONTENT_TYPE: &str = "application/grpc-web+proto";

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
  #[error("invalid TapTime endpoint: {0}")]
  InvalidEndpoint(String),
  #[error("HTTP request failed: {0}")]
  Network(#[from] reqwest::Error),
  #[error("API returned HTTP {0}")]
  HttpStatus(u16),
  #[error("TapTime login is missing or expired")]
  Unauthenticated,
  #[error("gRPC status {code}: {message}")]
  Grpc { code: u32, message: String },
  #[error("invalid gRPC-Web reply: {0}")]
  Wire(&'static str),
  #[error("protobuf decoding failed: {0}")]
  Protobuf(#[from] prost::DecodeError),
  #[error("invalid protobuf duration: {0}")]
  Duration(#[from] prost_types::DurationError),
  #[error("invalid TapTime day: {0}")]
  Day(#[from] taptime_core::Error),
  #[error("invalid TapTime balance: {0}")]
  Balance(#[from] taptime_schema::Error),
  #[error("missing API field: {0}")]
  Missing(&'static str),
  #[error("unknown or invalid user timezone: {0}")]
  TimeZone(String),
  #[error("invalid login response: {0}")]
  Session(#[from] std::io::Error),
}

pub fn decode_worked_time(
  remote: prost_types::Duration,
) -> std::result::Result<Duration, prost_types::DurationError> {
  remote.try_into()
}

#[derive(Clone)]
pub struct TapTimeClient {
  http: reqwest::Client,
  base_url: String,
}

impl TapTimeClient {
  pub fn new(base_url: &str) -> Result<Self> {
    let url = reqwest::Url::parse(base_url)
      .map_err(|error| ApiError::InvalidEndpoint(error.to_string()))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if (url.scheme() != "https" && !(local && url.scheme() == "http"))
      || url.username() != ""
      || url.password().is_some()
      || url.query().is_some()
      || url.fragment().is_some()
      || url.path() != "/"
    {
      return Err(ApiError::InvalidEndpoint(
        "expected https://host[:port] (plain HTTP allowed for localhost only)".into(),
      ));
    }
    let http = reqwest::Client::builder()
      .timeout(DEFAULT_TIMEOUT)
      .redirect(reqwest::redirect::Policy::none())
      .build()?;
    Ok(Self {
      http,
      base_url: url.as_str().trim_end_matches('/').to_string(),
    })
  }

  pub fn production() -> Result<Self> {
    Self::new("https://api.time.whs31.com")
  }

  pub async fn login(&self, email: &str, password: &str) -> Result<LoginResult> {
    let request = LoginRequest {
      email: email.trim().to_owned(),
      password: password.to_owned(),
    };
    let response: AuthResponse = self
      .unary("AuthService", "Login", &request.encode_to_vec(), None)
      .await?;
    let user = response
      .user
      .ok_or(ApiError::Missing("AuthResponse.user"))?;
    let session = Session::new(response.jwt)?;
    Ok(LoginResult { session, user })
  }

  pub async fn get_user(&self, session: &Session) -> Result<User> {
    self
      .unary("AuthService", "GetUser", &[], Some(session))
      .await
  }

  pub async fn get_dashboard(
    &self,
    session: &Session,
    date: NaiveDate,
  ) -> Result<DashboardResponse> {
    let day = Date::from(date);
    let request = DashboardRequest {
      range_start: Some(day),
      range_end: Some(day),
      month_start: Some(day),
      month_end: Some(day),
      today: Some(day),
    };
    self
      .unary(
        "StoreService",
        "GetDashboard",
        &request.encode_to_vec(),
        Some(session),
      )
      .await
  }

  pub async fn get_work_hours(&self, session: &Session, date: NaiveDate) -> Result<Duration> {
    let request = Date::from(date);
    let remote: prost_types::Duration = self
      .unary(
        "StoreService",
        "GetWorkHours",
        &request.encode_to_vec(),
        Some(session),
      )
      .await?;
    Ok(decode_worked_time(remote)?)
  }

  pub async fn get_today(&self, session: &Session) -> Result<TodaySnapshot> {
    let user = self.get_user(session).await?;
    let zone_name = &user
      .time_zone
      .as_ref()
      .ok_or(ApiError::Missing("User.time_zone"))?
      .time_zone;
    let zone: Tz = zone_name
      .parse()
      .map_err(|_| ApiError::TimeZone(zone_name.to_owned()))?;
    self.get_today_in_zone(session, zone).await
  }

  pub async fn get_today_in_zone(&self, session: &Session, zone: Tz) -> Result<TodaySnapshot> {
    let sampled_at = Utc::now();
    let today = sampled_at.with_timezone(&zone).date_naive();
    let dashboard = self.get_dashboard(session, today).await?;
    TodaySnapshot::from_dashboard(dashboard, today, zone)
  }

  async fn unary<T: Message + Default>(
    &self,
    service: &str,
    method: &str,
    payload: &[u8],
    session: Option<&Session>,
  ) -> Result<T> {
    let url = format!(
      "{}/com.whs31.taptime.services.{service}/{method}",
      self.base_url
    );
    let mut frame = Vec::with_capacity(5 + payload.len());
    frame.push(0);
    let len = u32::try_from(payload.len()).map_err(|_| ApiError::Wire("request too big"))?;
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(payload);

    let mut request = self
      .http
      .post(url)
      .header(CONTENT_TYPE, GRPC_WEB_CONTENT_TYPE)
      .header("accept", GRPC_WEB_CONTENT_TYPE)
      .header("x-grpc-web", "1")
      .body(frame);
    if let Some(session) = session {
      request = request.header(AUTHORIZATION, format!("Bearer {}", session.token()));
    }
    let mut response = request.send().await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED
      || response.status() == reqwest::StatusCode::FORBIDDEN
    {
      return Err(ApiError::Unauthenticated);
    }
    if !response.status().is_success() {
      return Err(ApiError::HttpStatus(response.status().as_u16()));
    }
    let header_status = response
      .headers()
      .get("grpc-status")
      .and_then(|value| value.to_str().ok())
      .map(str::to_owned);
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
      if chunk.len() > MAX_MESSAGE_SIZE - body.len() {
        return Err(ApiError::Wire("response exceeds size limit"));
      }
      body.extend_from_slice(&chunk);
    }
    let message = extract_grpc_web_message(&body, header_status.as_deref())?;
    Ok(T::decode(message)?)
  }
}

fn extract_grpc_web_message<'a>(body: &'a [u8], header_status: Option<&str>) -> Result<&'a [u8]> {
  let mut cursor = 0;
  let mut message: Option<&[u8]> = None;
  let mut status = header_status.map(str::to_owned);
  let mut grpc_message = String::new();

  while cursor < body.len() {
    if body.len() - cursor < 5 {
      return Err(ApiError::Wire("truncated gRPC-Web frame header"));
    }
    let flag = body[cursor];
    let len = u32::from_be_bytes(body[cursor + 1..cursor + 5].try_into().unwrap()) as usize;
    cursor += 5;
    let end = cursor
      .checked_add(len)
      .ok_or(ApiError::Wire("frame length overflow"))?;
    if end > body.len() || len > MAX_MESSAGE_SIZE {
      return Err(ApiError::Wire("truncated or oversized frame"));
    }
    let bytes = &body[cursor..end];
    cursor = end;
    match flag {
      0 => {
        if message.replace(bytes).is_some() {
          return Err(ApiError::Wire("unexpected second unary message"));
        }
      }
      0x80 => {
        if cursor != body.len() {
          return Err(ApiError::Wire("trailers must be the final frame"));
        }
        let trailer =
          std::str::from_utf8(bytes).map_err(|_| ApiError::Wire("invalid trailer text"))?;
        for line in trailer.lines() {
          if let Some((name, value)) = line.trim_end_matches('\r').split_once(':') {
            match name.trim().to_ascii_lowercase().as_str() {
              "grpc-status" => status = Some(value.trim().to_owned()),
              "grpc-message" => grpc_message = value.trim().to_owned(),
              _ => {}
            }
          }
        }
      }
      _ => return Err(ApiError::Wire("compressed or unknown gRPC-Web frame")),
    }
  }
  let code: u32 = status
    .ok_or(ApiError::Wire("missing grpc-status"))?
    .parse()
    .map_err(|_| ApiError::Wire("invalid grpc-status"))?;
  if code == 16 {
    return Err(ApiError::Unauthenticated);
  }
  if code != 0 {
    return Err(ApiError::Grpc {
      code,
      message: grpc_message,
    });
  }
  message.ok_or(ApiError::Wire("missing gRPC response message"))
}

#[derive(Debug, Clone)]
pub struct TodaySnapshot {
  pub date: NaiveDate,
  pub time_zone: Tz,
  pub day: Day,
  pub clocked_work: Duration,
  pub work_target: Duration,
  pub server_balance: Balance,
}

impl TodaySnapshot {
  pub fn from_dashboard(dashboard: DashboardResponse, date: NaiveDate, zone: Tz) -> Result<Self> {
    let epoch_day = Date::from(date).days_since_epoch;
    let summary = dashboard
      .days
      .into_iter()
      .find(|item| {
        item
          .day
          .as_ref()
          .and_then(|day| day.date.as_ref())
          .is_some_and(|day| day.days_since_epoch == epoch_day)
      })
      .ok_or(ApiError::Missing("DashboardResponse.days[today]"))?;

    let proto_day = summary
      .day
      .as_ref()
      .ok_or(ApiError::Missing("DaySummary.day"))?;
    let day = Day::try_from(proto_day)?;
    let clocked_work = decode_worked_time(
      summary
        .clocked_work
        .ok_or(ApiError::Missing("DaySummary.clocked_work"))?,
    )?;
    let work_target = decode_worked_time(
      summary
        .work_target
        .ok_or(ApiError::Missing("DaySummary.work_target"))?,
    )?;
    let proto_balance = summary
      .balance
      .as_ref()
      .ok_or(ApiError::Missing("DaySummary.balance"))?;
    let server_balance = Balance::try_from(proto_balance)?;
    Ok(Self {
      date,
      time_zone: zone,
      day,
      clocked_work,
      work_target,
      server_balance,
    })
  }

  pub fn is_active(&self) -> bool {
    matches!(self.day.events.last(), Some(Event::CheckIn(_)))
  }

  pub fn worked_at(&self, now: DateTime<Utc>) -> Duration {
    let mut worked = self.clocked_work;
    if !self.is_active() || now.with_timezone(&self.time_zone).date_naive() != self.date {
      return worked;
    }
    if let Some(Event::CheckIn(start)) = self.day.events.last() {
      let local = now.with_timezone(&self.time_zone).time();
      let current_seconds = i64::from(local.num_seconds_from_midnight());
      let start_seconds = i64::from(start.hour * 3600 + start.minute * 60 + start.second);
      let seconds = current_seconds.saturating_sub(start_seconds).max(0);
      worked = worked.saturating_add(Duration::from_secs(seconds as u64));
    }
    worked
  }

  pub fn balance_at(&self, now: DateTime<Utc>) -> Balance {
    if !self.is_active() || now.with_timezone(&self.time_zone).date_naive() != self.date {
      return self.server_balance;
    }
    let non_regular = self
      .day
      .flags
      .intersects(DayFlags::WEEKEND | DayFlags::DAY_OFF | DayFlags::VACATION | DayFlags::REMOTE)
      || self.day.required_work_hours <= chrono::Duration::zero();
    if non_regular {
      return self.server_balance;
    }
    if let Some(first) = self.day.first_check_in() {
      let current_seconds = i64::from(
        now
          .with_timezone(&self.time_zone)
          .time()
          .num_seconds_from_midnight(),
      );
      let first_seconds = i64::from(first.hour * 3600 + first.minute * 60 + first.second);
      let presence =
        chrono::Duration::seconds(current_seconds.saturating_sub(first_seconds).max(0));
      Balance::calculate(presence, self.day.required_day_duration())
    } else {
      self.server_balance
    }
  }
}

#[cfg(test)]
mod tests {
  use taptime_schema::{
    Day as ProtoDay, Event as ProtoEvent, LocalTime as ProtoLocalTime, balance, event,
  };

  use super::*;

  fn framed_reply(payload: &[u8], grpc_status: &str) -> Vec<u8> {
    let mut out = vec![0];
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    let trailer = format!("grpc-status: {grpc_status}\r\n");
    out.push(0x80);
    out.extend_from_slice(&(trailer.len() as u32).to_be_bytes());
    out.extend_from_slice(trailer.as_bytes());
    out
  }

  #[test]
  fn rejects_frames_after_trailers() {
    let mut reply = framed_reply(b"abc", "0");
    reply.extend_from_slice(&[0, 0, 0, 0, 0]);
    assert!(matches!(
      extract_grpc_web_message(&reply, None),
      Err(ApiError::Wire(_))
    ));
  }

  #[test]
  fn rejects_truncated_and_compressed_frames() {
    for reply in [&[0, 0][..], &[0, 0, 0, 0, 1][..], &[1, 0, 0, 0, 0][..]] {
      assert!(matches!(
        extract_grpc_web_message(reply, Some("0")),
        Err(ApiError::Wire(_))
      ));
    }
  }

  #[test]
  fn validates_endpoints() {
    assert!(TapTimeClient::new("http://[::1]:8080").is_ok());
    assert!(TapTimeClient::new("http://example.com").is_err());
    assert!(TapTimeClient::new("https://user:password@example.com").is_err());
    assert!(TapTimeClient::new("https://example.com/?token=x").is_err());
  }

  #[test]
  fn grpc_web_extracts_message_and_checks_status() {
    assert_eq!(
      extract_grpc_web_message(&framed_reply(b"abc", "0"), None).unwrap(),
      b"abc"
    );
    assert!(matches!(
      extract_grpc_web_message(&framed_reply(b"abc", "16"), None),
      Err(ApiError::Unauthenticated)
    ));
  }

  #[test]
  fn decode_duration_checks_sign_and_fraction() {
    assert_eq!(
      decode_worked_time(prost_types::Duration {
        seconds: 3600,
        nanos: 500_000_000
      })
      .unwrap(),
      Duration::new(3600, 500_000_000)
    );
    assert!(
      decode_worked_time(prost_types::Duration {
        seconds: -600,
        nanos: 0
      })
      .is_err()
    );
  }

  #[test]
  fn snapshot_adds_open_session_and_computes_presence_balance() {
    let date = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
    let proto = ProtoDay {
      date: Some(Date::from(date)),
      events: vec![ProtoEvent {
        id: None,
        event_type: Some(event::EventType::CheckIn(ProtoLocalTime {
          hour: 8,
          minute: 30,
          second: 0,
        })),
      }],
      flags: 0,
      required_work_hours: Some(prost_types::Duration {
        seconds: 8 * 3600,
        nanos: 0,
      }),
      lunch_break_duration: Some(prost_types::Duration {
        seconds: 30 * 60,
        nanos: 0,
      }),
    };
    let summary = taptime_schema::services::DaySummary {
      day: Some(proto),
      clocked_work: Some(prost_types::Duration {
        seconds: 0,
        nanos: 0,
      }),
      balance: Some(taptime_schema::Balance {
        balance_type: Some(balance::BalanceType::Exact(())),
      }),
      work_target: Some(prost_types::Duration {
        seconds: 8 * 3600,
        nanos: 0,
      }),
      ..Default::default()
    };
    let dashboard = DashboardResponse {
      days: vec![summary],
      month_stats: None,
    };
    let snapshot = TodaySnapshot::from_dashboard(dashboard, date, chrono_tz::UTC).unwrap();
    let now = date.and_hms_opt(17, 30, 0).unwrap().and_utc();
    assert!(snapshot.is_active());
    assert_eq!(snapshot.worked_at(now), Duration::from_secs(9 * 3600));
    assert_eq!(
      snapshot.balance_at(now),
      Balance::Overtime(chrono::Duration::minutes(30))
    );
  }
}
