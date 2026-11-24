import QtQuick
import qs.Common
import "TapTime.js" as TT

// Derived, theme-aware presentation of a status object: the phase, its accent
// color and icon, labels and the estimated time the daily target is reached.
QtObject {
    id: root

    property var status: ({})

    // Re-evaluated on every status line (once per display.interval_ms), so
    // time-relative values stay fresh without a separate timer.
    readonly property date now: {
        root.status;
        return new Date();
    }

    readonly property string phase: TT.phase(status)
    readonly property bool problem: TT.isProblem(phase)
    readonly property bool stale: !problem && phase !== "loading" && TT.isStale(status)
    readonly property bool checkedIn: phase === "working" || phase === "overtime"
    readonly property var seconds: status.seconds || ({})

    readonly property real workProgress: TT.workProgress(status)
    readonly property real dayProgress: TT.dayProgress(status)
    readonly property var target: TT.targetTime(status, now)
    readonly property string targetText: TT.hhmm(target)
    readonly property string remainingText: TT.clock(seconds.remaining)

    readonly property color accent: {
        switch (phase) {
        case "working":
            return Theme.primary;
        case "overtime":
            return Theme.tertiary;
        case "broken":
        case "disconnected":
        case "signedOut":
            return Theme.error;
        }
        return Theme.surfaceVariantText;
    }

    readonly property string icon: {
        switch (phase) {
        case "broken":
            return "error";
        case "disconnected":
            return "sync_problem";
        case "signedOut":
            return "lock";
        case "loading":
            return "hourglass_empty";
        case "away":
            return "logout";
        case "idle":
            return "schedule";
        }
        return "work_history";
    }

    readonly property string statusText: {
        switch (phase) {
        case "loading":
            return I18n.trFor("taptime", "Waiting for data");
        case "broken":
            return I18n.trFor("taptime", "Not running");
        case "disconnected":
            return I18n.trFor("taptime", "Daemon offline");
        case "signedOut":
            return I18n.trFor("taptime", "Signed out");
        case "idle":
            return I18n.trFor("taptime", "Not checked in");
        case "working":
            return I18n.trFor("taptime", "In since %1").arg(status.start_time || "--:--");
        case "overtime":
            return I18n.trFor("taptime", "Overtime");
        case "away":
            return I18n.trFor("taptime", "Out at %1").arg(status.end_time || "--:--");
        }
        return "";
    }

    // Color for a signed balance: accent when positive, muted otherwise
    // (a negative day balance is normal until the day is over).
    function balanceColor(seconds, base) {
        return TT.signOf(seconds) > 0 ? Theme.tertiary : base;
    }
}
