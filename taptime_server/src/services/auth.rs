use std::sync::Arc;

use argon2::{
  Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
  password_hash::{SaltString, rand_core::OsRng},
};
use taptime_schema::{
  Uid as ProtoUid, User,
  services::{
    AuthResponse, DeleteAccountRequest, LoginRequest, RegisterUserRequest, UpdateProfileRequest,
    UpdateRfidUidRequest, UpdateSettingsRequest, auth_service_server::AuthService,
  },
  user,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::db::{CredRow, fetch_core_user, weekday_to_iso};

pub struct AuthServiceImpl {
  db: sqlx::PgPool,
  jwt_secret: String,
}

impl AuthServiceImpl {
  pub fn new(db: sqlx::PgPool, jwt_secret: String) -> Self {
    Self { db, jwt_secret }
  }

  fn make_jwt(&self, user_id: Uuid) -> Result<String, Status> {
    crate::jwt::sign(user_id, &self.jwt_secret)
  }

  fn decode_jwt(&self, token: &str) -> Result<Uuid, Status> {
    crate::jwt::verify(token, &self.jwt_secret)
  }

  async fn fetch_user(&self, id: Uuid) -> Result<taptime_core::User, Status> {
    fetch_core_user(&self.db, id).await
  }

  fn authenticated_user_id<T>(&self, request: &Request<T>) -> Result<Uuid, Status> {
    let token = extract_bearer(request)?;
    self.decode_jwt(&token)
  }

  async fn verify_user_password(&self, user_id: Uuid, password: &str) -> Result<(), Status> {
    if password.is_empty() {
      return Err(Status::invalid_argument("Password cannot be empty"));
    }

    let cred = sqlx::query_as::<_, CredRow>(include_str!(
      "queries/fetch_user_credentials_by_user_id.sql"
    ))
    .bind(user_id)
    .fetch_optional(&self.db)
    .await
    .map_err(|e| Status::internal(e.to_string()))?
    .ok_or_else(|| Status::not_found("User not found"))?;

    verify_password(password, &cred.password_hash)
  }
}

fn extract_bearer<T>(request: &Request<T>) -> Result<String, Status> {
  request
    .metadata()
    .get("authorization")
    .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?
    .to_str()
    .map_err(|_| Status::unauthenticated("Invalid authorization header encoding"))?
    .strip_prefix("Bearer ")
    .map(str::to_owned)
    .ok_or_else(|| Status::unauthenticated("Expected Bearer token"))
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), Status> {
  let parsed_hash =
    PasswordHash::new(password_hash).map_err(|e| Status::internal(e.to_string()))?;
  Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .map_err(|_| Status::unauthenticated("Invalid password"))
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
  value
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn uid_bytes_from_proto(uid: Option<ProtoUid>) -> Result<Option<Vec<u8>>, Status> {
  uid
    .map(|uid| {
      let core_uid =
        taptime_core::Uid::try_from(uid).map_err(|e| Status::invalid_argument(e.to_string()))?;
      Ok(core_uid.as_bytes().to_vec())
    })
    .transpose()
}

fn settings_from_update_request(
  req: &UpdateSettingsRequest,
) -> Result<(chrono_tz::Tz, taptime_core::UserSettings), Status> {
  let time_zone: chrono_tz::Tz = req
    .time_zone
    .clone()
    .ok_or_else(|| Status::invalid_argument("Missing time_zone"))?
    .try_into()
    .map_err(|e: taptime_schema::Error| Status::invalid_argument(e.to_string()))?;

  let proto_settings = user::Settings {
    required_work_hours: req.required_work_hours,
    lunch_break_duration: req.lunch_break_duration,
    weekends: req.weekends.clone(),
    remote_days: req.remote_days.clone(),
  };
  let settings = taptime_core::UserSettings::try_from(&proto_settings)
    .map_err(|e: taptime_schema::Error| Status::invalid_argument(e.to_string()))?;

  Ok((time_zone, settings))
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
  async fn register_user(
    self: Arc<Self>,
    request: Request<RegisterUserRequest>,
  ) -> Result<Response<AuthResponse>, Status> {
    let req = request.into_inner();
    let proto_user = req
      .user
      .ok_or_else(|| Status::invalid_argument("Missing user"))?;

    if req.password.is_empty() {
      return Err(Status::invalid_argument("Password cannot be empty"));
    }
    if proto_user.name.is_empty() || proto_user.email.is_empty() {
      return Err(Status::invalid_argument("Name and email are required"));
    }

    let time_zone: chrono_tz::Tz = proto_user
      .time_zone
      .ok_or_else(|| Status::invalid_argument("Missing time_zone"))?
      .try_into()
      .map_err(|e: taptime_schema::Error| Status::invalid_argument(e.to_string()))?;

    let settings = proto_user
      .settings
      .as_ref()
      .map(taptime_core::UserSettings::try_from)
      .transpose()
      .map_err(|e: taptime_schema::Error| Status::invalid_argument(e.to_string()))?
      .unwrap_or_default();

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
      .bind(&proto_user.email)
      .fetch_one(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    if exists {
      return Err(Status::already_exists("Email already registered"));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
      .hash_password(req.password.as_bytes(), &salt)
      .map_err(|e| Status::internal(format!("Hashing failed: {e}")))?
      .to_string();

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let weekends: Vec<i32> = settings
      .weekends
      .iter()
      .copied()
      .map(weekday_to_iso)
      .collect();
    let remote_days: Vec<i32> = settings
      .remote_days
      .iter()
      .copied()
      .map(weekday_to_iso)
      .collect();

    sqlx::query(include_str!("queries/create_user.sql"))
      .bind(user_id)
      .bind(&proto_user.name)
      .bind(&proto_user.email)
      .bind(&proto_user.organization)
      .bind(time_zone.name())
      .bind(now)
      .bind(now)
      .bind(None::<Vec<u8>>)
      .bind(settings.required_work_hours.num_seconds())
      .bind(settings.lunch_break_duration.num_seconds())
      .bind(&weekends)
      .bind(&remote_days)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    sqlx::query(include_str!("queries/write_user_credentials.sql"))
      .bind(user_id)
      .bind(&proto_user.email)
      .bind(&password_hash)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    tracing::info!(user_id = %user_id, email = %proto_user.email, "user registered");

    let user = self.fetch_user(user_id).await?;
    let jwt = self.make_jwt(user_id)?;
    Ok(Response::new(AuthResponse {
      jwt,
      user: Some(User::from(&user)),
    }))
  }

  async fn login(
    self: Arc<Self>,
    request: Request<LoginRequest>,
  ) -> Result<Response<AuthResponse>, Status> {
    let req = request.into_inner();

    let cred = sqlx::query_as::<_, CredRow>(include_str!("queries/fetch_user_credentials.sql"))
      .bind(&req.email)
      .fetch_optional(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?
      .ok_or_else(|| Status::not_found("User not found"))?;

    verify_password(&req.password, &cred.password_hash)?;

    sqlx::query(include_str!("queries/update_user_last_seen.sql"))
      .bind(cred.user_id)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    tracing::info!(user_id = %cred.user_id, email = %req.email, "user logged in");

    let user = self.fetch_user(cred.user_id).await?;
    let jwt = self.make_jwt(cred.user_id)?;
    Ok(Response::new(AuthResponse {
      jwt,
      user: Some(User::from(&user)),
    }))
  }

  async fn get_user(self: Arc<Self>, request: Request<()>) -> Result<Response<User>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    tracing::debug!(user_id = %user_id, "fetching authenticated user");
    let user = self.fetch_user(user_id).await?;
    Ok(Response::new(User::from(&user)))
  }

  async fn update_profile(
    self: Arc<Self>,
    request: Request<UpdateProfileRequest>,
  ) -> Result<Response<User>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    let req = request.into_inner();
    let name = req.name.trim().to_string();
    let email = req.email.trim().to_string();
    let organization = clean_optional_text(req.organization);

    if name.is_empty() || email.is_empty() {
      return Err(Status::invalid_argument("Name and email are required"));
    }

    let email_exists: bool =
      sqlx::query_scalar(include_str!("queries/email_exists_for_other_user.sql"))
        .bind(&email)
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    if email_exists {
      return Err(Status::already_exists("Email already registered"));
    }

    let mut tx = self
      .db
      .begin()
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    sqlx::query(include_str!("queries/update_user_profile.sql"))
      .bind(user_id)
      .bind(&name)
      .bind(&email)
      .bind(&organization)
      .execute(&mut *tx)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    sqlx::query(include_str!("queries/update_user_credentials_email.sql"))
      .bind(user_id)
      .bind(&email)
      .execute(&mut *tx)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    tx.commit()
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    let user = self.fetch_user(user_id).await?;
    Ok(Response::new(User::from(&user)))
  }

  async fn update_settings(
    self: Arc<Self>,
    request: Request<UpdateSettingsRequest>,
  ) -> Result<Response<User>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    let req = request.into_inner();
    let (time_zone, settings) = settings_from_update_request(&req)?;
    let weekends: Vec<i32> = settings
      .weekends
      .iter()
      .copied()
      .map(weekday_to_iso)
      .collect();
    let remote_days: Vec<i32> = settings
      .remote_days
      .iter()
      .copied()
      .map(weekday_to_iso)
      .collect();

    sqlx::query(include_str!("queries/update_user_settings.sql"))
      .bind(user_id)
      .bind(time_zone.name())
      .bind(settings.required_work_hours.num_seconds())
      .bind(settings.lunch_break_duration.num_seconds())
      .bind(&weekends)
      .bind(&remote_days)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    let user = self.fetch_user(user_id).await?;
    Ok(Response::new(User::from(&user)))
  }

  async fn update_rfid_uid(
    self: Arc<Self>,
    request: Request<UpdateRfidUidRequest>,
  ) -> Result<Response<User>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    let uid = uid_bytes_from_proto(request.into_inner().rfid_uid)?;

    sqlx::query(include_str!("queries/update_user_rfid_uid.sql"))
      .bind(user_id)
      .bind(uid)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    let user = self.fetch_user(user_id).await?;
    Ok(Response::new(User::from(&user)))
  }

  async fn delete_time_data(self: Arc<Self>, request: Request<()>) -> Result<Response<()>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    let mut tx = self
      .db
      .begin()
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    sqlx::query(include_str!("queries/delete_events.sql"))
      .bind(user_id)
      .execute(&mut *tx)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    sqlx::query(include_str!("queries/delete_day_flags.sql"))
      .bind(user_id)
      .execute(&mut *tx)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;
    tx.commit()
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(()))
  }

  async fn delete_account(
    self: Arc<Self>,
    request: Request<DeleteAccountRequest>,
  ) -> Result<Response<()>, Status> {
    let user_id = self.authenticated_user_id(&request)?;
    self
      .verify_user_password(user_id, &request.into_inner().password)
      .await?;

    sqlx::query(include_str!("queries/delete_user.sql"))
      .bind(user_id)
      .execute(&self.db)
      .await
      .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clean_optional_text_trims_and_clears_empty_values() {
    assert_eq!(
      clean_optional_text(Some("  Acme  ".into())),
      Some("Acme".into())
    );
    assert_eq!(clean_optional_text(Some("   ".into())), None);
    assert_eq!(clean_optional_text(None), None);
  }

  #[test]
  fn uid_bytes_from_proto_accepts_supported_lengths() {
    assert_eq!(
      uid_bytes_from_proto(Some(ProtoUid {
        value: vec![0xA1, 0xB2, 0xC3, 0xD4],
      }))
      .unwrap(),
      Some(vec![0xA1, 0xB2, 0xC3, 0xD4])
    );
    assert!(
      uid_bytes_from_proto(Some(ProtoUid {
        value: vec![1, 2, 3]
      }))
      .is_err()
    );
    assert_eq!(uid_bytes_from_proto(None).unwrap(), None);
  }
}
