# TapTime bar module and notifications

Desktop frontends for the local `taptime-daemon` (see [`taptime_client`](../taptime_client)):

- `taptime-waybar` prints the current status as Waybar JSON (`once`, `watch`), opens the dashboard (`open`) and validates the config (`config`).
- `taptime-notifier` is a user service that shows desktop notifications as the daily target gets close.

Neither binary talks to the TapTime API or reads the login session; both only read the daemon's Unix socket.

## Install

Everything is driven by `just` from the repository root:

```sh
just waybar install          # daemon + bar module + notifier
just client login            # once, to sign the daemon in
```

`just waybar install` builds release binaries, installs `taptime`, `taptime-daemon`, `taptime-waybar` and `taptime-notifier` to `~/.local/bin`, installs and (re)starts `taptime.service` and `taptime-notifier.service`, and creates `~/.config/taptime-waybar/config.toml` from [`examples/config.toml`](examples/config.toml) if it doesn't exist. Running `taptime-waybar watch` processes are restarted so bars pick up the new binary. Re-run it to update.

Then hook it into your bar:

| Bar | Command |
| --- | --- |
| [DankMaterialShell](integrations/dms) (e.g. on niri) | `just waybar dms install` |
| [Omarchy Shell](integrations/omarchy) | `just waybar omarchy install` |
| Waybar | manual, see below |

The shell recipes run `just waybar install` first, then copy the plugin. Use `just waybar dms plugin` / `just waybar omarchy plugin` to copy only the QML after editing it, and `just waybar dms restart` / `just waybar omarchy restart` to reload the shell.

Run `just waybar` to list every recipe. The other useful ones are `status`, `logs`, `config`, `edit` and `uninstall`.

### DankMaterialShell

After the first install, open DMS Settings → Plugins → Scan for Plugins, enable **TapTime**, and add it to DankBar. The plugin also provides a Control Center tile (Control Center edit mode → Add) and a dash overview card (dash edit mode → Add widget). One background stream feeds all of them, whatever the number of monitors.

- **Bar:** a ring filling up towards the daily target (the dot inside is filled while you're checked in), worked time, and the day balance (highlighted once it's positive). Its icon changes when the daemon is down, you're signed out, or `taptime-waybar` is missing.
- **Details popout:** when you can leave, worked time against the target, the day from check-in to the target, day/month balances and breaks, and sync freshness. When something is broken it says what happened and offers the fix: restart the daemon, or copy the login or install command.
- **Settings** (DMS Settings → Plugins → TapTime): bar text (worked time, time left, leave-at time or ring only), balance and ring visibility, and whether left click opens the details or the dashboard (right click does the other one).
- **IPC:** `dms ipc call widget toggle taptime` toggles the details, e.g. from a niri keybind. `dms ipc call taptime dashboard` opens the website, and `dms ipc call taptime text` / `json` print the current status.

Texts follow the DMS language. Russian is included in `integrations/dms/taptime/translations/`. After `just waybar dms plugin`, run `just waybar dms restart`: DMS caches the plugin's QML files until the shell restarts.

### Omarchy Shell

Enable `mels.taptime` in your Omarchy Shell config and restart the shell. Copying the files doesn't add the widget to the bar.

### Waybar

Add [`integrations/waybar/taptime.jsonc`](integrations/waybar/taptime.jsonc) to your config's `include` list (or paste its contents), put `custom/taptime` in a module list, and optionally copy the rules from [`integrations/waybar/style.css`](integrations/waybar/style.css). Waybar restarts the module 5 seconds after it exits.

## Configuration

Both binaries read only `$XDG_CONFIG_HOME/taptime-waybar/config.toml` (default `~/.config/taptime-waybar/config.toml`). Every key is optional and unknown keys are rejected. The [example](examples/config.toml) documents all settings.

- `[display]` covers the module text and tooltip templates, duration format, CSS classes and the dashboard URL. `taptime-waybar` reads it once at startup, so restart the bar (or run `pkill -x taptime-waybar`) after editing.
- `[notifications]` covers thresholds, texts, quiet hours, weekdays and urgency. The notifier reloads it every 5 seconds. While it is invalid, notifications pause and the error goes to `just waybar logs`.

`just waybar config` validates both sections and prints the effective settings.

Delivered notifications are tracked in `$XDG_STATE_HOME/taptime/notifications.json` (one week of history). `persistent = true` asks for a non-expiring notification, but the notification server may ignore it.

## Output

Each line from `taptime-waybar` is a JSON object with the Waybar fields `text`, `tooltip`, `class` and `alt` (`ready` or `stale`), plus `worked`, `balance`, `balance_positive`, `month_balance`, `month_balance_positive`, `start_time`, `end_time`, `active`, `date`, raw counters in `seconds` (`worked`, `presence`, `work_target`, `presence_target`, `balance`, `month_balance`, `remaining`), the daemon's sync `error`, `disconnected` and `last_sync` for the Quickshell frontends. [`integrations/quickshell/TapTimeStatus.qml`](integrations/quickshell/TapTimeStatus.qml) is a UI-neutral data source for this stream. Other Quickshell bars can reuse it.

## Development

```sh
just waybar test
just waybar lint
just waybar fmt
just waybar run          # cargo run -- watch, against the running daemon
```

## Troubleshooting

When the module reports *Daemon unavailable*, check the daemon first:

```sh
just client status
just client logs
```

`No such file or directory` from `taptime status` means the socket at `$XDG_RUNTIME_DIR/taptime/status.sock` is missing, so the service isn't running. It is not a credentials problem. `"error":"unauthenticated"` in the status JSON means you need to run `just client login`.
