use std::{
  fs,
  process::{Child, Command, Stdio},
  time::Duration,
};
struct Process(Child);
impl Drop for Process {
  fn drop(&mut self) {
    let _ = self.0.kill();
    let _ = self.0.wait();
  }
}
#[test]
fn singleton_restart_and_sigterm_without_daemon() {
  let root = std::env::temp_dir().join(format!("taptime-notifier-test-{}", std::process::id()));
  fs::create_dir_all(&root).unwrap();
  let start = || {
    Process(
      Command::new(env!("CARGO_BIN_EXE_taptime-notifier"))
        .env("XDG_RUNTIME_DIR", &root)
        .env("XDG_CONFIG_HOME", &root)
        .env("XDG_STATE_HOME", &root)
        .env("DBUS_SESSION_BUS_ADDRESS", "unix:path=/nonexistent")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap(),
    )
  };
  for signal in ["-TERM", "-KILL", "-TERM"] {
    let mut first = start();
    let mut locked = false;
    for _ in 0..100 {
      if let Ok(lock) = fs::OpenOptions::new()
        .write(true)
        .open(root.join("taptime/notifier.lock"))
        && lock.try_lock().is_err()
      {
        locked = true;
        break;
      }
      std::thread::sleep(Duration::from_millis(20));
    }
    assert!(locked);
    let mut second = start();
    assert!(!second.0.wait().unwrap().success());
    assert!(
      Command::new("kill")
        .args([signal, &first.0.id().to_string()])
        .status()
        .unwrap()
        .success()
    );
    let status = first.0.wait().unwrap();
    assert_eq!(status.success(), signal == "-TERM");
  }
  fs::remove_dir_all(root).unwrap();
}
