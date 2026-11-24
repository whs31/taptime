import QtQuick
import Quickshell.Io
import qs.Services
import qs.Modules.Plugins

// Instantiated once per shell. Owns the single `taptime-waybar watch` stream
// and publishes it as the "status" global var for every bar, Control Center
// and dash surface, so extra monitors don't spawn extra processes.
PluginComponent {
    id: root

    function publish() {
        PluginService.setGlobalVar("taptime", "status", source.status);
    }

    Component.onCompleted: publish()

    TapTimeStatus {
        id: source
        onStatusChanged: root.publish()
    }

    // dms ipc call taptime dashboard
    // dms ipc call taptime text
    // (the popout itself: dms ipc call widget toggle taptime)
    IpcHandler {
        target: "taptime"

        function dashboard(): string {
            source.openDashboard();
            return "ok";
        }

        function text(): string {
            return source.status.text;
        }

        function json(): string {
            return JSON.stringify(source.status);
        }
    }
}
