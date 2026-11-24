use std::io;

#[cfg(all(unix, feature = "backend"))]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
#[cfg(feature = "backend")]
use std::{
  env,
  fs::{self, OpenOptions},
  io::Write,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use taptime_schema::User;

pub struct Session {
  jwt: String,
}

impl Session {
  pub fn new(jwt: impl Into<String>) -> io::Result<Self> {
    let jwt = jwt.into();
    if jwt.is_empty() || !jwt.bytes().all(|byte| byte.is_ascii_graphic()) {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid JWT"));
    }
    Ok(Self { jwt })
  }

  pub fn token(&self) -> &str {
    &self.jwt
  }
}

pub struct LoginResult {
  pub session: Session,
  pub user: User,
}

#[cfg(feature = "backend")]
pub fn session_path() -> io::Result<PathBuf> {
  let data = match env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
    Some(path) => PathBuf::from(path),
    None => {
      let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is missing"))?;
      PathBuf::from(home).join(".local/share")
    }
  };
  Ok(data.join("taptime-client/session.jwt"))
}

#[cfg(feature = "backend")]
pub fn load_session() -> io::Result<Option<Session>> {
  let path = session_path()?;
  let metadata = match fs::symlink_metadata(&path) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
    Err(error) => return Err(error),
  };
  if !metadata.is_file() || metadata.file_type().is_symlink() {
    return Err(io::Error::new(
      io::ErrorKind::PermissionDenied,
      "unsafe session file",
    ));
  }
  #[cfg(unix)]
  if metadata.permissions().mode() & 0o077 != 0 {
    return Err(io::Error::new(
      io::ErrorKind::PermissionDenied,
      "session file must have permissions 0600",
    ));
  }
  let jwt = fs::read_to_string(path)?;
  Session::new(jwt.trim_end_matches(&['\r', '\n'][..]).to_owned()).map(Some)
}

#[cfg(feature = "backend")]
pub fn save_session(session: &Session) -> io::Result<()> {
  let path = session_path()?;
  let dir = path
    .parent()
    .ok_or_else(|| io::Error::other("invalid session path"))?;
  fs::create_dir_all(dir)?;
  #[cfg(unix)]
  fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;

  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(io::Error::other)?
    .as_nanos();
  let tmp = dir.join(format!("session.{}.{}.tmp", std::process::id(), nonce));
  let write_result = (|| -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&tmp)?;
    file.write_all(session.token().as_bytes())?;
    file.sync_all()?;
    fs::rename(&tmp, &path)?;
    Ok(())
  })();
  if write_result.is_err() {
    let _ = fs::remove_file(&tmp);
  }
  if write_result.is_ok() {
    let _ = crate::storage::write("status.json", &serde_json::Value::Null);
  }
  write_result
}

#[cfg(feature = "backend")]
pub fn clear_session() -> io::Result<()> {
  let _ = crate::storage::write("status.json", &serde_json::Value::Null);
  match fs::remove_file(session_path()?) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_empty_or_multiline_tokens() {
    assert!(Session::new("").is_err());
    assert!(Session::new("x\ty").is_err());
    assert!(Session::new(" x ").is_err());
    assert!(Session::new("x\ny").is_err());
    assert_eq!(Session::new("abc").unwrap().token(), "abc");
  }
}
