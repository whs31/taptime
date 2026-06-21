use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tonic::Status;
use uuid::Uuid;

const ADMIN_SUBJECT: &str = "admin";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
  pub sub: String,
  pub iat: u64,
  pub exp: u64,
}

pub fn sign(user_id: Uuid, secret: &str) -> Result<String, Status> {
  let now = chrono::Utc::now();
  let claims = Claims {
    sub: user_id.to_string(),
    iat: now.timestamp() as u64,
    exp: (now + chrono::Duration::days(30)).timestamp() as u64,
  };
  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
  .map_err(|e| Status::internal(format!("JWT sign error: {e}")))
}

pub fn sign_admin(
  secret: &str,
  ttl: chrono::Duration,
) -> Result<(String, chrono::DateTime<chrono::Utc>), Status> {
  let now = chrono::Utc::now();
  let expires_at = now + ttl;
  let claims = Claims {
    sub: ADMIN_SUBJECT.to_string(),
    iat: now.timestamp() as u64,
    exp: expires_at.timestamp() as u64,
  };
  let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
  .map_err(|e| Status::internal(format!("JWT sign error: {e}")))?;
  Ok((token, expires_at))
}

pub fn verify(token: &str, secret: &str) -> Result<Uuid, Status> {
  decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )
  .map_err(|_| Status::unauthenticated("Invalid or expired token"))
  .and_then(|d| {
    Uuid::parse_str(&d.claims.sub).map_err(|_| Status::unauthenticated("Invalid token"))
  })
}

pub fn verify_admin(token: &str, secret: &str) -> Result<(), Status> {
  let claims = decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )
  .map_err(|_| Status::unauthenticated("Invalid or expired admin token"))?
  .claims;
  if claims.sub == ADMIN_SUBJECT {
    Ok(())
  } else {
    Err(Status::unauthenticated("Invalid admin token"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn admin_token_roundtrip_and_user_token_rejected_for_admin() {
    let secret = "secret";
    let (admin_token, expires_at) = sign_admin(secret, chrono::Duration::hours(8)).unwrap();
    assert!(expires_at > chrono::Utc::now());
    assert!(verify_admin(&admin_token, secret).is_ok());

    let user_token = sign(Uuid::new_v4(), secret).unwrap();
    assert!(verify_admin(&user_token, secret).is_err());
  }
}
