#!/usr/bin/env bash
# Install TapTime Rust binaries + user services for a Niri desktop.
# This does not modify the running Quickshell panel.
set -Eeuo pipefail

usage() {
  cat <<'HELP'
Usage: taptime_waybar/integrations/niri/deploy.sh

Build and install:
  - taptime, taptime-daemon, taptime-waybar, taptime-notifier
  - taptime.service, taptime-notifier.service (systemd --user)
  - sample config.toml (only if no user config already exists)

Preserves TapTime authentication, user settings and notification history.
Backs up existing binaries, services and configuration before replacing them.
Does NOT modify Niri config.kdl. If DMS is installed, installs its
TapTime plugin (you enable/add it through DMS Settings).
HELP
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi
if (($# != 0)); then
  usage >&2
  exit 2
fi

for command in cargo install systemctl mktemp cp mv; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$command" >&2
    exit 1
  fi
done

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd -P)
cd "$repo_root"

for source in \
  taptime_client/Cargo.toml \
  taptime_waybar/Cargo.toml \
  taptime_client/systemd/taptime.service \
  taptime_waybar/systemd/taptime-notifier.service \
  taptime_waybar/examples/config.toml; do
  if [[ ! -f "$source" ]]; then
    printf 'Missing source file: %s\n' "$source" >&2
    exit 1
  fi
done

umask 077
bin_dir="$HOME/.local/bin"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
state_home="${XDG_STATE_HOME:-$HOME/.local/state}"
units_dir="$HOME/.config/systemd/user"
config_dir="$config_home/taptime-waybar"
config_file="$config_dir/config.toml"
backups_dir="$state_home/taptime-deploy-backups"

printf '\n==> Build Rust binaries\n'
cargo build --release --locked --manifest-path taptime_client/Cargo.toml
cargo build --release --locked --manifest-path taptime_waybar/Cargo.toml

for binary in \
  taptime_client/target/release/taptime \
  taptime_client/target/release/taptime-daemon \
  taptime_waybar/target/release/taptime-waybar \
  taptime_waybar/target/release/taptime-notifier; do
  if [[ ! -x "$binary" ]]; then
    printf 'Expected built binary not found: %s\n' "$binary" >&2
    exit 1
  fi
done

printf '\n==> Back up current user installation\n'
mkdir -p "$backups_dir" "$bin_dir" "$units_dir" "$config_dir"
backup_dir=$(mktemp -d "$backups_dir/niri-$(date +%Y%m%d-%H%M%S)-XXXXXX")
mkdir -p "$backup_dir/bin" "$backup_dir/units" "$backup_dir/config"

for name in taptime taptime-daemon taptime-waybar taptime-notifier; do
  if [[ -f "$bin_dir/$name" ]]; then
    cp -a -- "$bin_dir/$name" "$backup_dir/bin/"
  fi
done
for name in taptime.service taptime-notifier.service; do
  if [[ -f "$units_dir/$name" ]]; then
    cp -a -- "$units_dir/$name" "$backup_dir/units/"
  fi
done
if [[ -f "$config_file" ]]; then
  cp -a -- "$config_file" "$backup_dir/config/config.toml"
fi
printf 'Backup: %s\n' "$backup_dir"

printf '\n==> Stop previous services\n'
systemctl --user stop taptime-notifier.service taptime.service 2>/dev/null || true

# Use a temporary file in the destination directory and an atomic rename.
# This avoids replacing the inode of a binary still used by Quickshell.
deploy_binary() {
  local source=$1 destination=$2 tmp
  tmp=$(mktemp "${destination}.new.XXXXXX")
  if ! install -m755 -- "$source" "$tmp"; then
    rm -f -- "$tmp"
    return 1
  fi
  mv -f -- "$tmp" "$destination"
}

printf '\n==> Install binaries and systemd units\n'
deploy_binary taptime_client/target/release/taptime "$bin_dir/taptime"
deploy_binary taptime_client/target/release/taptime-daemon "$bin_dir/taptime-daemon"
deploy_binary taptime_waybar/target/release/taptime-waybar "$bin_dir/taptime-waybar"
deploy_binary taptime_waybar/target/release/taptime-notifier "$bin_dir/taptime-notifier"
install -m644 taptime_client/systemd/taptime.service "$units_dir/taptime.service"
install -m644 taptime_waybar/systemd/taptime-notifier.service "$units_dir/taptime-notifier.service"

if [[ ! -f "$config_file" ]]; then
  install -m600 taptime_waybar/examples/config.toml "$config_file"
  printf 'Created: %s\n' "$config_file"
else
  printf 'Preserved existing config: %s\n' "$config_file"
fi

# DMS uses its own plugin format; do not reuse Omarchy QML.
if command -v dms >/dev/null 2>&1; then
  printf '\n==> Install DankMaterialShell plugin\n'
  dms_plugin_source="taptime_waybar/integrations/niri/dms/taptime"
  dms_plugin_dir="$config_home/DankMaterialShell/plugins/taptime"
  if [[ -f "$dms_plugin_source/plugin.json" ]]; then
    if [[ -d "$dms_plugin_dir" ]]; then
      cp -a -- "$dms_plugin_dir" "$backup_dir/dms-taptime"
    fi
    mkdir -p -- "$dms_plugin_dir"
    cp -a -- "$dms_plugin_source/." "$dms_plugin_dir/"
    install -m644 -- "taptime_waybar/integrations/niri/TapTimeStatus.qml" \
      "$dms_plugin_dir/TapTimeStatus.qml"
    printf 'DMS plugin: %s\n' "$dms_plugin_dir"
    printf 'Enable it in DMS Settings > Plugins and add TapTime to DankBar.\n'
  else
    printf 'DMS plugin source missing; skipping UI.\n' >&2
  fi
fi

printf '\n==> Validate user configuration\n'
"$bin_dir/taptime-waybar" config >/dev/null
"$bin_dir/taptime-notifier" config >/dev/null

printf '\n==> Start services\n'
systemctl --user daemon-reload
systemctl --user enable taptime.service taptime-notifier.service
systemctl --user restart taptime.service taptime-notifier.service
systemctl --user is-active --quiet taptime.service
systemctl --user is-active --quiet taptime-notifier.service

printf '\n==> Status\n'
systemctl --user is-active taptime.service taptime-notifier.service
"$bin_dir/taptime" status --json || true
"$bin_dir/taptime-waybar" once || true

cat <<'NOTE'

Rust backend and notifications installed.
If DMS was detected, the TapTime widget plugin has been copied to
~/.config/DankMaterialShell/plugins/taptime. In DMS Settings > Plugins,
scan and enable it, then add TapTime to DankBar. Run `dms restart` if
it does not appear. Niri compositor settings were not changed.
NOTE
