set shell := ["bash", "-euo", "pipefail", "-c"]

plugin_dir := config_directory() / "omarchy/plugins/mels.taptime"

[private]
default:
    @just --justfile "{{ source_file() }}" --list

# Install the backend (`just waybar install`) and the Omarchy Shell plugin
install: && plugin
    just --justfile ../../Justfile install

# Replace the plugin files with the repo's, e.g. after editing QML
plugin:
    rm -rf "{{ plugin_dir }}"
    mkdir -p "{{ plugin_dir }}"
    cp -a mels.taptime/. "{{ plugin_dir }}/"
    install -m644 ../quickshell/TapTimeStatus.qml "{{ plugin_dir }}/"
    if command -v omarchy >/dev/null; then omarchy plugin validate "{{ plugin_dir }}"; fi
    @echo "Installed {{ plugin_dir }}"
    @echo "Enable mels.taptime in your Omarchy Shell config, then: just waybar omarchy restart"

# Restart Omarchy Shell to reload plugins
restart:
    omarchy restart shell

# Remove the plugin files
uninstall:
    rm -rf "{{ plugin_dir }}"
