use std::{fs, io, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{io::AsyncReadExt, net::UnixStream};
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

pub const VERSION: u64 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
  pub version: u64,
  pub sampled_at: String,
  pub last_sync: Option<String>,
  pub stale: bool,
  pub error: Option<String>,
  pub date: Option<String>,
  pub active: bool,
  pub worked_seconds: Option<i64>,
  pub presence_seconds: Option<i64>,
  pub work_target_seconds: Option<i64>,
  pub presence_target_seconds: Option<i64>,
  pub balance_seconds: Option<i64>,
  #[serde(default)]
  pub month_balance_seconds: Option<i64>,
  #[serde(default)]
  pub first_check_in: Option<String>,
  #[serde(default)]
  pub last_check_out: Option<String>,
  pub local_time: Option<String>,
}
pub async fn status() -> io::Result<Value> {
  let socket = UnixStream::connect(runtime_dir()?.join("status.sock")).await?;
  let mut data = Vec::new();
  tokio::time::timeout(
    Duration::from_secs(3),
    socket.take(65537).read_to_end(&mut data),
  )
  .await
  .map_err(io::Error::other)??;
  if data.len() > 65536 {
    return Err(io::Error::other("IPC response too large"));
  }
  let value: Value = serde_json::from_slice(&data).map_err(io::Error::other)?;
  if value["version"] != VERSION {
    return Err(io::Error::other("unsupported IPC version"));
  }
  Ok(value)
}

pub async fn snapshot() -> io::Result<Status> {
  let value = status().await?;
  if value["version"] != VERSION {
    return Err(io::Error::other("unsupported IPC version"));
  }
  serde_json::from_value(value).map_err(io::Error::other)
}
