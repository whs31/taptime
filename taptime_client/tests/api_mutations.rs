use std::time::Duration;

use chrono::NaiveDate;
use prost::Message;
use taptime_client::{ApiError, Session, TapTimeClient, api::*};
use taptime_schema::{User, services};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::TcpListener,
};

async fn mock(
  method: &str,
  payload: Vec<u8>,
  status: u32,
) -> (TapTimeClient, tokio::task::JoinHandle<Vec<u8>>) {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let endpoint = format!("http://{}", listener.local_addr().unwrap());
  let method = method.to_owned();
  let task = tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut bytes = Vec::new();
    let header_end;
    loop {
      let mut chunk = [0; 4096];
      let count = stream.read(&mut chunk).await.unwrap();
      assert_ne!(count, 0);
      bytes.extend_from_slice(&chunk[..count]);
      if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
        header_end = index + 4;
        break;
      }
    }
    let header = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    assert!(
      header
        .lines()
        .next()
        .unwrap()
        .contains(&format!("/{method} HTTP/1.1"))
    );
    if method != "RegisterUser" {
      assert!(
        header
          .to_lowercase()
          .contains("authorization: bearer test-token")
      );
    }
    let size: usize = header
      .lines()
      .find_map(|line| {
        line
          .to_lowercase()
          .strip_prefix("content-length: ")
          .map(str::to_owned)
      })
      .unwrap()
      .parse()
      .unwrap();
    while bytes.len() < header_end + size {
      let mut chunk = [0; 4096];
      let count = stream.read(&mut chunk).await.unwrap();
      assert_ne!(count, 0);
      bytes.extend_from_slice(&chunk[..count]);
    }
    let mut body = vec![0];
    body.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    body.extend(payload);
    let trailer = format!("grpc-status: {status}\r\n");
    body.push(0x80);
    body.extend_from_slice(&(trailer.len() as u32).to_be_bytes());
    body.extend(trailer.as_bytes());
    stream
      .write_all(
        format!(
          "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
          body.len()
        )
        .as_bytes(),
      )
      .await
      .unwrap();
    stream.write_all(&body).await.unwrap();
    bytes[header_end + 5..header_end + size].to_vec()
  });
  (TapTimeClient::new(&endpoint).unwrap(), task)
}

fn date() -> NaiveDate {
  NaiveDate::from_ymd_opt(2026, 11, 14).unwrap()
}
fn session() -> Session {
  Session::new("test-token").unwrap()
}
fn event() -> EventRequest {
  EventRequest {
    date: date(),
    time: taptime_core::LocalTime::new(9, 12, 34).unwrap(),
  }
}
fn settings() -> SettingsUpdate {
  SettingsUpdate {
    time_zone: chrono_tz::Europe::Berlin,
    required_work_hours: Duration::from_secs(28800),
    lunch_break_duration: Duration::from_secs(1800),
    weekends: vec![taptime_schema::Weekday::Saturday],
    remote_days: vec![],
    start_date: Some(date()),
  }
}
fn profile() -> ProfileUpdate {
  ProfileUpdate {
    name: "Test".into(),
    email: "test@example.invalid".into(),
    organization: None,
  }
}

#[tokio::test]
async fn event_operations_encode_actual_proto() {
  for check_in in [true, false] {
    let (client, task) = mock(
      if check_in {
        "AddCheckIn"
      } else {
        "AddCheckOut"
      },
      vec![],
      0,
    )
    .await;
    if check_in {
      client.add_check_in(&session(), event()).await.unwrap();
    } else {
      client.add_check_out(&session(), event()).await.unwrap();
    }
    let request = services::EventRequest::decode(task.await.unwrap().as_slice()).unwrap();
    assert_eq!(request.date, Some(date().into()));
    assert_eq!(request.time.unwrap().second, 34);
  }
  let (client, task) = mock("DeleteEvent", vec![], 0).await;
  let id = taptime_schema::Uuid {
    most_significant_bits: 12,
    least_significant_bits: 34,
  };
  client.delete_event(&session(), id).await.unwrap();
  assert_eq!(
    services::DeleteEventRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .event_id,
    Some(id)
  );
}

#[tokio::test]
async fn override_clear_is_absent_not_zero_and_flag_is_toggle() {
  for duration in [None, Some(Duration::ZERO), Some(Duration::from_secs(25200))] {
    let (client, task) = mock("SetRequiredWorkHoursOverride", vec![], 0).await;
    client
      .set_work_override(
        &session(),
        WorkOverrideRequest {
          date: date(),
          required_work_hours: duration,
        },
      )
      .await
      .unwrap();
    let request =
      services::SetRequiredWorkHoursOverrideRequest::decode(task.await.unwrap().as_slice())
        .unwrap();
    assert_eq!(
      request.required_work_hours.map(|v| v.seconds),
      duration.map(|v| v.as_secs() as i64)
    );
  }
  let (client, task) = mock("SetFlag", vec![], 0).await;
  client
    .toggle_day_flag(
      &session(),
      FlagRequest {
        date: date(),
        flag: taptime_schema::DayFlag::Remote,
      },
    )
    .await
    .unwrap();
  assert_eq!(
    services::SetFlagRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .flag,
    taptime_schema::DayFlag::Remote as i32
  );
}

#[tokio::test]
async fn account_mutations_preserve_fields() {
  let user = User {
    name: "Returned".into(),
    ..Default::default()
  };
  let (client, task) = mock("UpdateProfile", user.encode_to_vec(), 0).await;
  assert_eq!(
    client
      .update_profile(&session(), profile())
      .await
      .unwrap()
      .name,
    "Returned"
  );
  assert_eq!(
    services::UpdateProfileRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .organization,
    None
  );
  let (client, task) = mock("UpdateSettings", user.encode_to_vec(), 0).await;
  client
    .update_settings(&session(), settings())
    .await
    .unwrap();
  let request = services::UpdateSettingsRequest::decode(task.await.unwrap().as_slice()).unwrap();
  assert_eq!(request.time_zone.unwrap().time_zone, "Europe/Berlin");
  assert_eq!(request.required_work_hours.unwrap().seconds, 28800);
  let (client, task) = mock("UpdateRfidUid", user.encode_to_vec(), 0).await;
  client.update_rfid(&session(), None).await.unwrap();
  assert!(
    services::UpdateRfidUidRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .rfid_uid
      .is_none()
  );
  let (client, task) = mock("DeleteTimeData", vec![], 0).await;
  client.delete_time_data(&session()).await.unwrap();
  assert!(task.await.unwrap().is_empty());
  let (client, task) = mock("DeleteAccount", vec![], 0).await;
  client
    .delete_account(&session(), "test-password".into())
    .await
    .unwrap();
  assert_eq!(
    services::DeleteAccountRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .password,
    "test-password"
  );
  let (client, task) = mock(
    "RegisterUser",
    services::AuthResponse {
      jwt: "registered-token".into(),
      user: Some(user),
    }
    .encode_to_vec(),
    0,
  )
  .await;
  assert_eq!(
    client
      .register(RegisterRequest {
        profile: profile(),
        settings: settings(),
        password: "test-password".into()
      })
      .await
      .unwrap()
      .session
      .token(),
    "registered-token"
  );
  assert_eq!(
    services::RegisterUserRequest::decode(task.await.unwrap().as_slice())
      .unwrap()
      .user
      .unwrap()
      .name,
    "Test"
  );
}

#[tokio::test]
async fn refused_mutation_returns_status() {
  let (client, task) = mock("AddCheckIn", vec![], 9).await;
  assert!(matches!(
    client.add_check_in(&session(), event()).await,
    Err(ApiError::Grpc { code: 9, .. })
  ));
  task.await.unwrap();
}

#[tokio::test]
async fn lost_response_does_not_repeat_mutation() {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let client = TapTimeClient::new(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
  let server = tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut buffer = [0; 4096];
    assert!(stream.read(&mut buffer).await.unwrap() > 0);
    drop(stream);
    assert!(
      tokio::time::timeout(Duration::from_millis(200), listener.accept())
        .await
        .is_err()
    );
  });
  assert!(matches!(
    client.add_check_in(&session(), event()).await,
    Err(ApiError::Network(_))
  ));
  server.await.unwrap();
}

#[tokio::test]
async fn timeout_after_submission_does_not_repeat_mutation() {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let client = TapTimeClient::new(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
  let server = tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut bytes = Vec::new();
    loop {
      let mut buffer = [0; 4096];
      let count = stream.read(&mut buffer).await.unwrap();
      assert_ne!(count, 0);
      bytes.extend_from_slice(&buffer[..count]);
      if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n")
        && bytes.len() >= end + 9
      {
        let length = u32::from_be_bytes(bytes[end + 5..end + 9].try_into().unwrap()) as usize;
        if bytes.len() >= end + 9 + length {
          break;
        }
      }
    }
    assert!(
      tokio::time::timeout(Duration::from_secs(11), listener.accept())
        .await
        .is_err()
    );
    drop(stream);
  });
  let error = client.add_check_out(&session(), event()).await.unwrap_err();
  assert!(matches!(error, ApiError::Network(ref value) if value.is_timeout()));
  server.await.unwrap();
}
