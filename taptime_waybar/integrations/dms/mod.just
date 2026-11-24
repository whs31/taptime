set shell := ["bash", "-euo", "pipefail", "-c"]

plugin_dir := config_directory() / "dms/plugins/taptime"

[private]
default:
    @just --justfile "{{ source_file() }}" --list

# Install the backend (`just waybar install`) and the DankBar widget
install: && plugin
    just --justfile ../../Justfile install

# Replace the widget files with the repo's, e.g. after editing QML
plugin:
    rm -rf "{{ plugin_dir }}"
    mkdir -p "{{ plugin_dir }}"
    cp -a taptime/. "{{ plugin_dir }}/"
    install -m644 ../quickshell/TapTimeStatus.qml "{{ plugin_dir }}/"
    @echo "Installed {{ plugin_dir }}"
    @echo "First install: DMS Settings > Plugins > Scan, enable TapTime, add it to DankBar."
    @echo "Updates: run 'just waybar dms restart' to reload the shell."

# Restart DankMaterialShell to reload plugins
restart:
    dms restart

# Remove the widget files
uninstall:
    rm -rf "{{ plugin_dir }}"
