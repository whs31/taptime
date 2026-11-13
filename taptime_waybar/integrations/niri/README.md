# TapTime on Niri / DankMaterialShell

Niri is a compositor; the bar is provided by your shell. This integration supports **DMS (DankMaterialShell)** using its native plugin API, without launching another bar.

## Install

Run from the TapTime repository root:

```sh
bash taptime_waybar/integrations/niri/deploy.sh
```

The deployment builds/install the Rust binaries and user systemd services, preserves existing `~/.config/taptime-waybar/config.toml`, and copies the DMS plugin to `~/.config/dms/plugins/taptime/` when `dms` is installed. Existing binaries, units and DMS plugin files are backed up in the printed backup directory. The script never removes TapTime login state. It does **not** modify Niri config or DMS bar layout.

## Enable widget

1. Open DMS Settings → Plugins → Scan for Plugins.
2. Enable **TapTime**.
3. Add **TapTime** to DankBar (Bar Layout / Add Widget).
4. If needed, run `dms restart`.

Bar: left click opens TapTime dashboard; right/middle click opens DMS native popup. Popup closes on external click or Escape. Colors, sizing and fonts follow DMS `Theme`.

## Diagnose a disconnected daemon

A service may briefly report `active` even if it fails immediately afterwards. The CLI error `No such file or directory` usually means the Unix socket expected by the client is missing or unreachable; it does not prove a credential error.

```sh
printf 'XDG_RUNTIME_DIR=%s\n' "${XDG_RUNTIME_DIR:-<unset>}"
systemctl --user status taptime.service --no-pager
journalctl --user -u taptime.service -b -n 80 --no-pager
ls -la "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/taptime" 2>&1
~/.local/bin/taptime status --json
```

If status JSON reads `"error":"unauthenticated"`, sign in interactively using `~/.local/bin/taptime login` (never put credentials in command-line arguments). If `status.sock` is absent, inspect systemd errors *before* logging in.

## Manual plugin-only install

```sh
mkdir -p ~/.config/dms/plugins/taptime
cp -a taptime_waybar/integrations/niri/dms/taptime/. \
  ~/.config/dms/plugins/taptime/
cp taptime_waybar/integrations/niri/TapTimeStatus.qml \
  ~/.config/dms/plugins/taptime/TapTimeStatus.qml
dms restart
```

For other Quickshell panels, `TapTimeStatus.qml` remains the reusable, UI-neutral data source.
