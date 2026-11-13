# TapTime Waybar and desktop notifications

A Rust Waybar custom module and independent notification process backed by the local `taptime_client` daemon. Neither process contacts the TapTime API or reads its authentication token.

## Build and verify

From the repository root:

```sh
cargo build --release --manifest-path taptime_client/Cargo.toml
cargo build --release --manifest-path taptime_waybar/Cargo.toml
cargo test --manifest-path taptime_waybar/Cargo.toml
cargo clippy --manifest-path taptime_waybar/Cargo.toml --all-targets -- -D warnings
```

Start `taptime-daemon` or its `taptime.service` first.

## Single configuration file

Both `taptime-waybar` and `taptime-notifier` read **only**:

`$XDG_CONFIG_HOME/taptime-waybar/config.toml` (by default `~/.config/taptime-waybar/config.toml`).

The file has `[display]` and `[notifications]` sections, including notification thresholds. Missing sections use defaults. A sample is provided in [`examples/config.toml`](examples/config.toml).

```sh
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/taptime-waybar"
cp -n taptime_waybar/examples/config.toml "${XDG_CONFIG_HOME:-$HOME/.config}/taptime-waybar/config.toml"
taptime_waybar/target/release/taptime-waybar config
taptime_waybar/target/release/taptime-notifier config
```

If upgrading from the old split configuration, manually transfer your settings to this file before updating the binaries; the old `taptime/waybar.toml` and `taptime/config.toml` paths are no longer read. Configuration files are never installed over an existing user file by the examples above.

## Manual installation (Linux)

```sh
install -Dm755 taptime_waybar/target/release/taptime-waybar "$HOME/.local/bin/taptime-waybar"
install -Dm755 taptime_waybar/target/release/taptime-notifier "$HOME/.local/bin/taptime-notifier"
install -Dm644 taptime_waybar/systemd/taptime-notifier.service "$HOME/.config/systemd/user/taptime-notifier.service"
systemctl --user daemon-reload
systemctl --user enable --now taptime-notifier.service
```

For updates, stop `taptime-notifier.service` before replacing its executable, then restart it. Use `systemctl --user disable --now taptime-notifier.service` to disable notifications. Neither systemd service is a Waybar instance; the bar starts its own `taptime-waybar watch` process.

## Waybar integration

See [`examples/waybar.jsonc`](examples/waybar.jsonc) for `custom/taptime` and [`examples/style.css`](examples/style.css) for optional CSS classes. Add the module to the bar's module list, then restart Waybar. `taptime-waybar once` prints one JSON object; `watch` streams one object per update; `open` launches the configured dashboard URL.

## Omarchy Shell / Quickshell

Omarchy Shell is based on Quickshell, not Waybar. The source plugin is in [`integrations/omarchy/mels.taptime`](integrations/omarchy/mels.taptime). It is a separate frontend for the same `taptime-waybar watch` JSON stream and retains left-click dashboard access and right/middle-click details popup.

To copy the plugin manually (after backing up any existing user customization):

```sh
mkdir -p "$HOME/.config/omarchy/plugins/mels.taptime"
cp -a taptime_waybar/integrations/omarchy/mels.taptime/. "$HOME/.config/omarchy/plugins/mels.taptime/"
omarchy plugin validate "$HOME/.config/omarchy/plugins/mels.taptime"
omarchy restart shell
```

The plugin must also be enabled in your Omarchy Shell configuration; copying files alone does not automatically add it to the bar. The plugin expects `~/.local/bin/taptime-waybar` from the manual installation above. The QML uses current Omarchy theme colors, not an embedded palette.

## Notification behavior

`taptime-notifier` runs separately from Waybar and reads status using the same Unix IPC. Configure thresholds, text, urgency, quiet hours, weekday filters and notification options in `[notifications]`. Delivered threshold state lives under `$XDG_STATE_HOME/taptime/`.

`persistent = true` requests a non-expiring desktop notification; the actual notification server may ignore it. In particular, test this with the current Omarchy Shell before relying on persistence. No Check In or Check Out API actions are performed by these tools.
