# TapTime Client

Rust client library and a per-user background daemon for the official TapTime gRPC-Web API. Uses the existing `taptime_core` and `taptime_schema` crates. The daemon is the sole owner of network synchronization and exposes a read-only JSON snapshot over a Unix socket.

Run commands from the repository root.

## Build and verify

```sh
just client build
just client test
just client lint
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

## Install (Linux)

```sh
just client install    # build, copy to ~/.local/bin, enable taptime.service
just client login
just client status
```

`just client uninstall` disables the service and removes the binaries and unit; the session and cached state are kept. `just waybar install` also runs this, see [`taptime_waybar`](../taptime_waybar).

## IPC contract

The daemon listens on `$XDG_RUNTIME_DIR/taptime/status.sock` (private directory and socket). Each connection receives one JSON line. Field definitions and protocol version are in [`status.schema.json`](status.schema.json). Cached status lives under `$XDG_STATE_HOME/taptime/` (or `~/.local/state/taptime/`), and credentials are kept in the configured user session store. Frontends should not use the TapTime API directly.

The client currently targets Linux/Unix for the local socket, terminal password input and systemd integration. The daemon calculates an estimated open session locally and marks stale data when synchronization is no longer fresh.
