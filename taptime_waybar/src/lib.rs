pub mod config;
pub mod format;
pub mod notifications;
pub mod render;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Opens `url` in the default browser. The child is reaped on a background
/// thread so long-running callers do not accumulate zombies.
pub fn open_url(url: &str) -> std::io::Result<()> {
  let mut child = std::process::Command::new("xdg-open").arg(url).spawn()?;
  std::thread::spawn(move || child.wait());
  Ok(())
}

/// Turns a `main` result into an exit code, printing the error for humans
/// (and the journal) instead of its `Debug` form.
pub fn exit_code(program: &str, result: Result<()>) -> std::process::ExitCode {
  match result {
    Ok(()) => std::process::ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("{program}: {error}");
      std::process::ExitCode::FAILURE
    }
  }
}
