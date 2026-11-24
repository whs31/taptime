import QtQuick
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets
import qs.DCommon.Widgets
import "TapTime.js" as TT

// Details view used by the bar popout and the Control Center detail page.
Column {
    id: root

    required property var status
    // Animate the hero only while somebody can see it.
    property bool live: true
    // Injected by PluginPopout.
    property var closePopout: null

    signal dashboardRequested

    readonly property TapTimeLook look: TapTimeLook {
        status: root.status
    }
    readonly property var seconds: look.seconds

    function copy(text) {
        Quickshell.execDetached(["dms", "cl", "copy", text]);
        ToastService.showInfo(I18n.trFor("taptime", "Copied %1").arg(text));
    }

    function restartDaemon() {
        Quickshell.execDetached(["systemctl", "--user", "restart", "taptime.service"]);
        ToastService.showInfo(I18n.trFor("taptime", "Restarting taptime.service"));
    }

    readonly property string dateText: {
        const date = TT.parseDate(status.date);
        return date ? date.toLocaleDateString(Qt.locale(), "dddd, d MMMM") : "";
    }

    readonly property string syncText: {
        const age = TT.ageSeconds(status.last_sync, look.now);
        if (age === null)
            return I18n.trFor("taptime", "Never synced");
        if (age < 60)
            return I18n.trFor("taptime", "Synced just now");
        if (age < 3600)
            return I18n.trFor("taptime", "Synced %1 min ago").arg(Math.floor(age / 60));
        return I18n.trFor("taptime", "Synced at %1").arg(TT.hhmm(new Date(Date.parse(status.last_sync))));
    }

    width: parent ? parent.width : 360
    spacing: Theme.spacingL

    // Header: title, date and a status chip.
    Item {
        width: parent.width
        height: Math.max(titleColumn.implicitHeight, chip.height)

        Column {
            id: titleColumn
            anchors.left: parent.left
            anchors.right: chip.left
            anchors.rightMargin: Theme.spacingS
            anchors.verticalCenter: parent.verticalCenter

            StyledText {
                text: "TapTime"
                font.pixelSize: Theme.fontSizeXLarge
                font.weight: Theme.fontWeightMedium
                color: Theme.surfaceText
            }

            StyledText {
                width: parent.width
                visible: text.length > 0
                text: root.dateText
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceVariantText
                elide: Text.ElideRight
            }
        }

        Rectangle {
            id: chip
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            width: chipRow.implicitWidth + Theme.spacingM * 2
            height: chipRow.implicitHeight + Theme.spacingXS * 2
            radius: Theme.fullRadius(width, height)
            color: Theme.withAlpha(root.look.accent, 0.16)

            Row {
                id: chipRow
                anchors.centerIn: parent
                spacing: Theme.spacingXS

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 7
                    height: 7
                    radius: 3.5
                    color: root.look.accent
                }

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.look.statusText
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Theme.fontWeightMedium
                    color: root.look.accent
                }
            }
        }
    }

    // Problems replace the whole body with an explanation and a fix.
    Column {
        width: parent.width
        visible: root.look.problem || root.look.phase === "loading"
        spacing: Theme.spacingM
        topPadding: Theme.spacingS
        bottomPadding: Theme.spacingS

        DCircularWavyProgress {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.look.phase === "loading"
            width: 40
            height: 40
            indeterminate: root.live && visible
        }

        DIcon {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.look.problem
            name: root.look.icon
            size: 40
            color: root.look.accent
        }

        StyledText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            font.pixelSize: Theme.fontSizeLarge
            font.weight: Theme.fontWeightMedium
            color: Theme.surfaceText
            text: {
                switch (root.look.phase) {
                case "broken":
                    return I18n.trFor("taptime", "taptime-waybar is not running");
                case "disconnected":
                    return I18n.trFor("taptime", "Can't reach the TapTime daemon");
                case "signedOut":
                    return I18n.trFor("taptime", "Signed out of TapTime");
                }
                return I18n.trFor("taptime", "Waiting for data");
            }
        }

        StyledText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            font.pixelSize: Theme.fontSizeMedium
            color: Theme.surfaceVariantText
            text: {
                switch (root.look.phase) {
                case "broken":
                    return I18n.trFor("taptime", "It is missing or the config is invalid. Reinstall from the repository:");
                case "disconnected":
                    return I18n.trFor("taptime", "taptime.service is stopped or crashed.");
                case "signedOut":
                    return I18n.trFor("taptime", "Sign in once from a terminal; the daemon picks it up automatically.");
                }
                return "";
            }
            visible: text.length > 0
        }

        // Command to run, click to copy.
        Rectangle {
            id: commandChip
            readonly property string command: root.look.phase === "broken" ? "just waybar dms install" : "taptime login"

            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.look.phase === "broken" || root.look.phase === "signedOut"
            width: commandRow.implicitWidth + Theme.spacingM * 2
            height: commandRow.implicitHeight + Theme.spacingS * 2
            radius: Theme.cornerRadius
            color: Theme.foregroundColor(Theme.cardSurface)

            Row {
                id: commandRow
                anchors.centerIn: parent
                spacing: Theme.spacingS

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: commandChip.command
                    isMonospace: true
                    font.pixelSize: Theme.fontSizeMedium
                    color: Theme.surfaceText
                }

                DIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "content_copy"
                    size: Theme.iconSizeSmall
                    color: Theme.surfaceVariantText
                }
            }

            StateLayer {
                anchors.fill: parent
                tooltipText: I18n.trFor("taptime", "Copy")
                onClicked: root.copy(commandChip.command)
            }
        }

        DButton {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.look.phase === "disconnected"
            variant: "tonal"
            iconName: "restart_alt"
            text: I18n.trFor("taptime", "Restart daemon")
            onClicked: root.restartDaemon()
        }
    }

    // Hero: worked-time ring and when the day's target is reached.
    Row {
        width: parent.width
        visible: !root.look.problem && root.look.phase !== "loading"
        spacing: Theme.spacingL

        Item {
            id: hero
            readonly property real size: 116

            width: size
            height: size

            DCircularWavyProgress {
                anchors.fill: parent
                progress: Math.min(1, root.look.workProgress)
                color: root.look.workProgress >= 1 ? Theme.tertiary : Theme.primary
                trackColor: Theme.withAlpha(color, 0.2)
                strokeWidth: 6
                trackStrokeWidth: 6
                wavelength: 26
                maxWaveAmplitude: root.look.checkedIn ? 1.8 : 0
                // A calm drift while checked in; flat otherwise.
                waveSpeed: root.live && root.look.checkedIn ? wavelength / 2 : 0
            }

            Column {
                anchors.centerIn: parent

                NumericText {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.status.worked || "--:--"
                    font.pixelSize: Theme.fontSizeXXLarge - 4
                    font.weight: Theme.fontWeightMedium
                    color: Theme.surfaceText
                }

                StyledText {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: I18n.trFor("taptime", "of %1").arg(TT.clock(root.seconds.work_target))
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.surfaceVariantText
                }
            }
        }

        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - hero.width - parent.spacing
            spacing: Theme.spacingXXS

            StyledText {
                width: parent.width
                wrapMode: Text.WordWrap
                font.pixelSize: Theme.fontSizeMedium
                color: Theme.surfaceVariantText
                text: {
                    switch (root.look.phase) {
                    case "working":
                        return I18n.trFor("taptime", "You can leave at");
                    case "overtime":
                        return I18n.trFor("taptime", "Target reached at");
                    case "away":
                        return I18n.trFor("taptime", "Checked out at");
                    }
                    return I18n.trFor("taptime", "Not checked in today");
                }
            }

            NumericText {
                visible: root.look.phase !== "idle"
                text: root.look.phase === "away" ? (root.status.end_time || "--:--") : root.look.targetText
                font.pixelSize: Theme.fontSizeDisplay
                font.weight: Theme.fontWeightMedium
                color: root.look.phase === "away" ? Theme.surfaceText : root.look.accent
                horizontalAlignment: Text.AlignLeft
            }

            StyledText {
                width: parent.width
                wrapMode: Text.WordWrap
                font.pixelSize: Theme.fontSizeMedium
                color: root.look.phase === "overtime" ? Theme.tertiary : Theme.surfaceVariantText
                text: {
                    switch (root.look.phase) {
                    case "working":
                        return I18n.trFor("taptime", "in %1").arg(root.look.remainingText);
                    case "overtime":
                        return I18n.trFor("taptime", "%1 overtime").arg(root.status.balance);
                    case "away":
                        return I18n.trFor("taptime", "Day balance %1").arg(root.status.balance);
                    }
                    return I18n.trFor("taptime", "Month balance %1").arg(root.status.month_balance);
                }
            }
        }
    }

    // Day timeline: check-in to the target, filled up to now.
    Column {
        width: parent.width
        visible: !root.look.problem && !!root.status.start_time
        spacing: Theme.spacingXS

        Rectangle {
            id: track
            width: parent.width
            height: 10
            radius: height / 2
            color: Theme.withAlpha(root.look.accent, 0.18)

            Rectangle {
                width: Math.max(parent.height, parent.width * Math.min(1, root.look.dayProgress))
                height: parent.height
                radius: parent.radius
                color: root.look.phase === "away" ? Theme.surfaceVariantText : root.look.accent

                Behavior on width {
                    NumberAnimation {
                        duration: Theme.mediumDuration
                        easing.type: Theme.standardEasing
                    }
                }
            }
        }

        Item {
            width: parent.width
            height: startLabel.implicitHeight

            StyledText {
                id: startLabel
                anchors.left: parent.left
                text: root.status.start_time || "--:--"
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceVariantText
            }

            StyledText {
                anchors.horizontalCenter: parent.horizontalCenter
                text: I18n.trFor("taptime", "In office %1 / %2").arg(TT.clock(root.seconds.presence)).arg(TT.clock(root.seconds.presence_target))
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceVariantText
            }

            StyledText {
                anchors.right: parent.right
                text: root.look.phase === "away" ? (root.status.end_time || "--:--") : root.look.targetText
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceVariantText
            }
        }
    }

    Row {
        id: stats
        readonly property real tileWidth: (width - spacing * 2) / 3

        width: parent.width
        visible: !root.look.problem && root.look.phase !== "loading"
        spacing: Theme.spacingS

        TapTimeStat {
            width: stats.tileWidth
            icon: "today"
            label: I18n.trFor("taptime", "Today")
            value: root.status.balance || "--:--"
            valueColor: root.look.balanceColor(root.seconds.balance, Theme.surfaceText)
        }

        TapTimeStat {
            width: stats.tileWidth
            icon: "calendar_month"
            label: I18n.trFor("taptime", "Month")
            value: root.status.month_balance || "--:--"
            valueColor: root.look.balanceColor(root.seconds.month_balance, Theme.surfaceText)
        }

        TapTimeStat {
            width: stats.tileWidth
            icon: "coffee"
            label: I18n.trFor("taptime", "Breaks")
            value: TT.clock(TT.breakSeconds(root.status))
        }
    }

    // Sync problems while the daemon itself is reachable.
    Rectangle {
        width: parent.width
        visible: root.look.stale
        height: staleRow.implicitHeight + Theme.spacingS * 2
        radius: Theme.cornerRadius
        color: Theme.withAlpha(Theme.warning, 0.14)

        Row {
            id: staleRow
            anchors.verticalCenter: parent.verticalCenter
            x: Theme.spacingM
            width: parent.width - Theme.spacingM * 2
            spacing: Theme.spacingS

            DIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: "cloud_off"
                size: Theme.iconSizeSmall
                color: Theme.warning
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - Theme.iconSizeSmall - parent.spacing
                wrapMode: Text.WordWrap
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceText
                text: root.status.error === "api_unavailable" ? I18n.trFor("taptime", "TapTime server is unreachable, times are estimated") : I18n.trFor("taptime", "Data may be outdated, times are estimated")
            }
        }
    }

    // Footer: freshness and the website (both need a working taptime-waybar).
    Item {
        width: parent.width
        height: dashboardButton.height
        visible: root.look.phase !== "broken" && root.look.phase !== "loading"

        StyledText {
            anchors.left: parent.left
            anchors.right: dashboardButton.left
            anchors.rightMargin: Theme.spacingS
            anchors.verticalCenter: parent.verticalCenter
            text: root.syncText
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.surfaceVariantText
            elide: Text.ElideRight
        }

        DButton {
            id: dashboardButton
            anchors.right: parent.right
            variant: "tonal"
            iconName: "open_in_new"
            text: I18n.trFor("taptime", "Dashboard")
            onClicked: root.dashboardRequested()
        }
    }
}
