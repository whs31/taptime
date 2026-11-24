import QtQuick
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets
import qs.DCommon.Widgets
import qs.Modules.Plugins
import qs.Modules.DDash
import "TapTime.js" as TT

// Dash overview card: worked time ring and the leave-at estimate.
DashCardComponent {
    id: root

    readonly property var taptime: PluginService.globalVars["taptime"]?.status ?? ({
            loading: true,
            seconds: {}
        })
    // Square-ish cards put the time inside a big ring; wide ones put a ring
    // next to the text; the smallest drops the ring.
    readonly property bool stacked: height >= DashMetrics.gridRowUnit * 2 && width < height * 1.5
    readonly property real ringSize: stacked ? Math.min(body.width, body.height - footer.height - Theme.spacingS) : Math.min(body.height, 96)
    readonly property bool showRing: !root.look.problem && ringSize >= 48

    readonly property string subtitle: {
        switch (root.look.phase) {
        case "working":
            return I18n.trFor("taptime", "until %1").arg(root.look.targetText);
        case "overtime":
            return I18n.trFor("taptime", "%1 overtime").arg(root.taptime.balance);
        }
        return I18n.trFor("taptime", "Day %1").arg(root.taptime.balance || "--:--");
    }

    clickable: true
    opensTab: false
    onClicked: Quickshell.execDetached([Quickshell.env("HOME") + "/.local/bin/taptime-waybar", "open"])

    readonly property TapTimeLook look: TapTimeLook {
        status: root.taptime
    }

    StyledText {
        id: label
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        text: "TapTime · " + root.look.statusText
        wrapMode: Text.NoWrap
        font.pixelSize: Theme.fontSizeSmall
        font.weight: Theme.fontWeightMedium
        color: root.mutedColor
        elide: Text.ElideRight
    }

    Item {
        id: body
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: label.bottom
        anchors.bottom: parent.bottom
        anchors.topMargin: Theme.spacingS

        DRingGauge {
            id: ring
            x: root.stacked ? (body.width - width) / 2 : 0
            y: root.stacked ? 0 : (body.height - height) / 2
            visible: root.showRing
            width: root.ringSize
            height: width
            value: Math.min(1, root.look.workProgress)
            strokeWidth: Math.max(4, Math.round(width / 14))
            ringColor: root.look.workProgress >= 1 ? Theme.tertiary : root.accentColor
            trackColor: root.chipColor

            Column {
                anchors.centerIn: parent
                visible: root.stacked

                NumericText {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.taptime.worked || "--:--"
                    font.pixelSize: Math.round(ring.width / 5.5)
                    font.weight: Theme.fontWeightMedium
                    color: root.contentColor
                }

                StyledText {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.subtitle
                    font.pixelSize: Theme.fontSizeSmall
                    color: root.mutedColor
                }
            }
        }

        Column {
            anchors.left: ring.visible ? ring.right : parent.left
            anchors.leftMargin: ring.visible ? Theme.spacingM : 0
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            visible: !root.stacked || !root.showRing
            spacing: Theme.spacingXXS

            NumericText {
                text: root.look.problem ? "--:--" : (root.taptime.worked || "--:--")
                font.pixelSize: Theme.fontSizeXXLarge
                font.weight: Theme.fontWeightMedium
                color: root.accentColor
                horizontalAlignment: Text.AlignLeft
            }

            StyledText {
                width: parent.width
                wrapMode: Text.NoWrap
                elide: Text.ElideRight
                font.pixelSize: Theme.fontSizeSmall
                color: root.mutedColor
                text: root.subtitle
            }
        }

        StyledText {
            id: footer
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: root.stacked ? parent.horizontalCenter : undefined
            anchors.left: root.stacked ? undefined : (ring.visible ? ring.right : parent.left)
            anchors.leftMargin: ring.visible && !root.stacked ? Theme.spacingM : 0
            visible: root.stacked || body.height >= 72
            wrapMode: Text.NoWrap
            font.pixelSize: Theme.fontSizeSmall
            color: root.mutedColor
            text: I18n.trFor("taptime", "Month %1").arg(root.taptime.month_balance || "--:--")
        }
    }
}
