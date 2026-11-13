use std::{
  io::{self, BufRead, Write},
  os::fd::AsRawFd,
};

use taptime_client::{TapTimeClient, backend, clear_session, save_session};

struct EchoGuard {
  fd: i32,
  original: libc::termios,
}

impl Drop for EchoGuard {
  fn drop(&mut self) {
    unsafe {
      libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.original);
    }
  }
}

fn credentials() -> io::Result<(String, String)> {
  let tty = std::fs::OpenOptions::new()
    .read(true)
    .write(true)
    .open("/dev/tty")?;
  let mut output = tty.try_clone()?;
  let mut input = io::BufReader::new(tty);
  write!(output, "Email: ")?;
  output.flush()?;
  let mut email = String::new();
  input.read_line(&mut email)?;
  let fd = input.get_ref().as_raw_fd();
  let mut original = std::mem::MaybeUninit::<libc::termios>::uninit();
  if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
    return Err(io::Error::last_os_error());
  }
  let original = unsafe { original.assume_init() };
  let mut hidden = original;
  hidden.c_lflag &= !libc::ECHO;
  if unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &hidden) } != 0 {
    return Err(io::Error::last_os_error());
  }
  let guard = EchoGuard { fd, original };
  write!(output, "Password: ")?;
  output.flush()?;
  let mut password = String::new();
  input.read_line(&mut password)?;
  drop(guard);
  writeln!(output)?;
  Ok((
    email.trim().to_owned(),
    password.trim_end_matches(['\r', '\n']).to_owned(),
  ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<_> = std::env::args().skip(1).collect();
  match args
    .iter()
    .map(String::as_str)
    .collect::<Vec<_>>()
    .as_slice()
  {
    ["login"] => {
      let (email, password) = credentials()?;
      let result = TapTimeClient::production()?
        .login(&email, &password)
        .await
        .map_err(|_| io::Error::other("login failed"))?;
      save_session(&result.session)?;
      println!("Logged in");
    }
    ["logout"] => {
      clear_session()?;
      println!("Logged out");
    }
    ["daemon"] => backend::run_daemon().await?,
    ["status", "--json"] => println!("{}", backend::status().await?),
    ["status"] => {
      let s = backend::status().await?;
      println!(
        "Worked: {} s; presence: {} s; balance: {} s; active: {}; stale: {}; error: {}",
        s["worked_seconds"],
        s["presence_seconds"],
        s["balance_seconds"],
        s["active"],
        s["stale"],
        s["error"]
      );
    }
    ["--help"] | [] => println!("taptime login | logout | status [--json] | daemon"),
    _ => return Err(io::Error::other("invalid arguments; use --help").into()),
  }
  Ok(())
}
