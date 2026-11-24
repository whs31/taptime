use std::{
  fs, io,
  io::Write,
  os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
  path::PathBuf,
};

pub fn root(variable: &str, fallback: &str) -> io::Result<PathBuf> {
  let path = std::env::var_os(variable)
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback));
  if !path.is_absolute() {
    return Err(io::Error::other("XDG path must be absolute"));
  }
  Ok(path.join("taptime"))
}

pub fn read(name: &str) -> Option<serde_json::Value> {
  let path = root("XDG_STATE_HOME", ".local/state").ok()?.join(name);
  let metadata = fs::symlink_metadata(&path).ok()?;
  if !metadata.is_file()
    || metadata.permissions().mode() & 0o077 != 0
    || metadata.uid() != fs::metadata("/proc/self").ok()?.uid()
    || metadata.len() > 1024 * 1024
  {
    return None;
  }
  serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub fn write(name: &str, value: &serde_json::Value) -> io::Result<()> {
  let dir = root("XDG_STATE_HOME", ".local/state")?;
  fs::create_dir_all(&dir)?;
  let meta = fs::symlink_metadata(&dir)?;
  if !meta.is_dir() || meta.uid() != fs::metadata("/proc/self")?.uid() {
    return Err(io::Error::other("unsafe state directory"));
  }
  fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
  let nonce = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_err(io::Error::other)?
    .as_nanos();
  let tmp = dir.join(format!(".{name}.{}.{nonce}.tmp", std::process::id()));
  let mut file = fs::OpenOptions::new()
    .create_new(true)
    .write(true)
    .mode(0o600)
    .open(&tmp)?;
  let result = (|| {
    file.write_all(value.to_string().as_bytes())?;
    file.sync_all()?;
    fs::rename(&tmp, dir.join(name))
  })();
  if result.is_err() {
    let _ = fs::remove_file(tmp);
  }
  result
}
