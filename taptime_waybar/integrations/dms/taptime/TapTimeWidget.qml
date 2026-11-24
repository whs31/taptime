import QtQuick
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets
import qs.DCommon.Widgets
import qs.Modules.Plugins
import "TapTime.js" as TT

// Bar pill, popout and Control Center tile. Data comes from TapTimeDaemon.
PluginComponent {
    id: root

    property var popoutService: null

    readonly property var taptime: PluginService.globalVars["taptime"]?.status ?? ({
            loading: true,
            seconds: {}
        })

    readonly property TapTimeLook look: TapTimeLook {
        status: root.taptime
    }

    // Settings (TapTimeSettings.qml).
    readonly property string pillText: pluginData.pillText || "worked"
    readonly property bool showBalance: pluginData.showBalance ?? true
    readonly property bool showRing: pluginData.showRing ?? true
    readonly property bool swapClicks: pluginData.clickAction === "dashboard"

    function openDashboard() {
        Quickshell.execDetached([Quickshell.env("HOME") + "/.local/bin/taptime-waybar", "open"]);
    }

    // Left click opens the popout and right click the dashboard, unless the
    // user swapped them. `forcePopout` lets right click reach the popout
    // while pillClickAction is set.
    property bool forcePopout: false
    pillClickAction: swapClicks && !forcePopout ? () => root.openDashboard() : null
    pillRightClickAction: () => {
        if (!root.swapClicks) {
            root.openDashboard();
            return;
        }
        root.forcePopout = true;
        root.triggerPopout();
        root.forcePopout = false;
    }

    // Main text of the horizontal pill.
    readonly property string primaryText: {
        if (root.look.problem)
            return root.look.statusText;
        switch (pillText) {
        case "remaining":
            return root.look.phase === "working" ? I18n.trFor("taptime", "%1 left").arg(root.look.remainingText) : taptime.worked || "--:--";
        case "leaveAt":
            return root.look.phase === "working" ? I18n.trFor("taptime", "until %1").arg(root.look.targetText) : taptime.worked || "--:--";
        }
        return taptime.worked || "--:--";
    }
    // While checked in, "time left" and "leave at" already say what the
    // (negative) balance would.
    readonly property bool showsBalance: showBalance && pillText !== "none" && !root.look.problem && root.look.phase !== "loading" && !(root.look.phase === "working" && (pillText === "remaining" || pillText === "leaveAt"))
    readonly property color mutedText: Theme.withAlpha(Theme.widgetTextColor, 0.7)

    ccWidgetIcon: "schedule"
    ccWidgetPrimaryText: "TapTime"
    ccWidgetSecondaryText: {
        if (root.look.problem || root.look.phase === "loading")
            return root.look.statusText;
        if (root.look.phase === "working")
            return (taptime.worked || "--:--") + " · " + I18n.trFor("taptime", "until %1").arg(root.look.targetText);
        return (taptime.worked || "--:--") + " · " + (taptime.balance || "--:--");
    }
    ccWidgetIsActive: root.look.checkedIn
    ccWidgetIsToggle: false
    onCcWidgetToggled: openDashboard()

    ccDetailContent: Component {
        Rectangle {
            implicitHeight: detail.implicitHeight + Theme.spacingL * 2
            radius: Theme.cornerRadius
            color: Theme.foregroundColor(Theme.cardSurface)

            TapTimePopout {
                id: detail
                x: Theme.spacingL
                y: Theme.spacingL
                width: parent.width - Theme.spacingL * 2
                status: root.taptime
                onDashboardRequested: root.openDashboard()
            }
        }
    }

    popoutWidth: 380
    popoutContent: Component {
        TapTimePopout {
            status: root.taptime
            live: Window.window?.visible ?? true
            onDashboardRequested: {
                if (closePopout)
                    closePopout();
                root.openDashboard();
            }
        }
    }

    horizontalBarPill: Component {
        Item {
            implicitWidth: row.implicitWidth
            implicitHeight: Math.max(row.implicitHeight, root.iconSize)
            opacity: root.look.stale ? 0.65 : 1

            Row {
                id: row
                anchors.centerIn: parent
                spacing: Theme.spacingS

                TapTimeRing {
                    anchors.verticalCenter: parent.verticalCenter
                    look: root.look
                    size: root.iconSize
                    showRing: root.showRing
                }

                NumericText {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.pillText !== "none" || root.look.problem
                    text: root.primaryText
                    font.pixelSize: root.textSize
                    color: root.look.problem ? root.look.accent : Theme.widgetTextColor
                }

                NumericText {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.showsBalance
                    text: root.taptime.balance || "--:--"
                    font.pixelSize: root.textSize
                    color: root.look.balanceColor(root.look.seconds.balance, root.mutedText)
                }
            }
        }
    }

    verticalBarPill: Component {
        Item {
            readonly property var parts: (root.taptime.worked || "--:--").split(":")

            implicitWidth: Math.max(column.implicitWidth, root.iconSize)
            implicitHeight: column.implicitHeight
            opacity: root.look.stale ? 0.65 : 1

            Column {
                id: column
                anchors.centerIn: parent
                spacing: Theme.spacingXXS

                TapTimeRing {
                    anchors.horizontalCenter: parent.horizontalCenter
                    look: root.look
                    size: root.iconSize
                    showRing: root.showRing
                }

                Repeater {
                    model: root.look.problem || root.pillText === "none" ? [] : parent.parent.parts

                    NumericText {
                        required property string modelData
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: modelData
                        font.pixelSize: root.textSize
                        color: Theme.widgetTextColor
                    }
                }
            }
        }
    }
}
