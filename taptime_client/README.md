# TapTime Client

Rust client library and a per-user background daemon for the official TapTime gRPC-Web API. Uses the existing `taptime_core` and `taptime_schema` crates. The daemon is the sole owner of network synchronization and exposes a read-only JSON snapshot over a Unix socket.

Run commands from the repository root.

## Build and verify

```sh
cargo build --release --manifest-path taptime_client/Cargo.toml
cargo test --manifest-path taptime_client/Cargo.toml
cargo clippy --manifest-path taptime_client/Cargo.toml --all-targets -- -D warnings
```

## Login and status

```sh
taptime_client/target/release/taptime login
taptime_client/target/release/taptime-daemon
```

In another terminal:

```sh
taptime_client/target/release/taptime status --json
```

`login` prompts for credentials on the local TTY. Do not put credentials in shell arguments or commit session files. To clear the stored login, run `taptime logout`. The CLI also supports `status` and `--help`.

## Install manually (Linux)

```sh
install -Dm755 taptime_client/target/release/taptime "$HOME/.local/bin/taptime"
install -Dm755 taptime_client/target/release/taptime-daemon "$HOME/.local/bin/taptime-daemon"
install -Dm644 taptime_client/systemd/taptime.service "$HOME/.config/systemd/user/taptime.service"
systemctl --user daemon-reload
systemctl --user enable --now taptime.service
```

Check status with `systemctl --user status taptime.service` and `taptime status --json`. Stop or disable with `systemctl --user disable --now taptime.service`. For an update, rebuild, stop the service, copy the new binaries, then start it again. Do not overwrite running executables during an update. Uninstall by disabling the service and removing only the files listed above; the session and cached state are not automatically deleted.

## IPC contract

The daemon listens on `$XDG_RUNTIME_DIR/taptime/status.sock` (private directory and socket). Each connection receives one JSON line. Field definitions and protocol version are in [`status.schema.json`](status.schema.json). Cached status lives under `$XDG_STATE_HOME/taptime/` (or `~/.local/state/taptime/`), and credentials are kept in the configured user session store. Frontends should not use the TapTime API directly.

The client currently targets Linux/Unix for the local socket, terminal password input and systemd integration. The daemon calculates an estimated open session locally and marks stale data when synchronization is no longer fresh.
