use std::{
  process::{Command, Stdio},
  time::Duration,
};

#[test]
fn ipc_singleton_restart_and_sigterm() {
  let dir = std::env::temp_dir().join(format!("taptime-test-{}", std::process::id()));
  std::fs::create_dir_all(&dir).unwrap();
  let start = || {
    Command::new(env!("CARGO_BIN_EXE_taptime-daemon"))
      .env("XDG_RUNTIME_DIR", &dir)
      .env("XDG_DATA_HOME", &dir)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .unwrap()
  };
  for _ in 0..2 {
    let mut daemon = start();
    for _ in 0..100 {
      if dir.join("taptime/status.sock").exists() {
        break;
      }
      std::thread::sleep(Duration::from_millis(20));
    }
    let result = Command::new(env!("CARGO_BIN_EXE_taptime"))
      .args(["status", "--json"])
      .env("XDG_RUNTIME_DIR", &dir)
      .output()
      .unwrap();
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["stale"], true);
    let mut second = start();
    assert!(!second.wait().unwrap().success());
    assert!(
      Command::new("kill")
        .args(["-TERM", &daemon.id().to_string()])
        .status()
        .unwrap()
        .success()
    );
    assert!(daemon.wait().unwrap().success());
    assert!(!dir.join("taptime/status.sock").exists());
  }
  std::fs::remove_dir_all(dir).unwrap();
}
