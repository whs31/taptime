import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Widgets
import qs.Modules.Plugins

PluginComponent {
    id: root
    property var popoutService: null

    readonly property var data: dataSource.status
    readonly property bool ready: data.alt === "ready"

    // DMS already owns the bar pill and the popout. We only supply content.
    // Left click is intercepted inside the pill to open the website;
    // right click is handled by DMS, and middle click by our pill MouseArea.
    pillRightClickAction: () => root.triggerPopout()
    popoutWidth: 320

    TapTimeStatus {
        id: dataSource
    }

    Process {
        id: openWebsite
        command: [Quickshell.env("HOME") + "/.local/bin/taptime-waybar", "open"]
        running: false
    }

    horizontalBarPill: Component {
        Item {
            implicitWidth: contents.implicitWidth + Theme.spacingS * 2
            implicitHeight: Math.max(contents.implicitHeight + Theme.spacingXS, 22)

            Row {
                id: contents
                anchors.centerIn: parent
                spacing: Theme.spacingXS

                DankIcon {
                    name: "schedule"
                    color: root.ready ? Theme.widgetTextColor : Theme.surfaceVariantText
                    font.pixelSize: root.iconSize
                    anchors.verticalCenter: parent.verticalCenter
                }

                StyledText {
                    text: root.data.worked + "  " + root.data.balance
                    color: root.ready ? Theme.widgetTextColor : Theme.surfaceVariantText
                    font.pixelSize: root.textSize
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            MouseArea {
                anchors.fill: parent
                z: 10
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                cursorShape: Qt.PointingHandCursor
                onPressed: mouse => {
                    if (mouse.button === Qt.LeftButton) {
                        if (!openWebsite.running)
                            openWebsite.running = true;
                    } else if (mouse.button === Qt.MiddleButton) {
                        root.triggerPopout();
                    }
                }
            }
        }
    }

    verticalBarPill: Component {
        Item {
            implicitWidth: Math.max(icon.implicitWidth + Theme.spacingXS, 22)
            implicitHeight: Math.max(icon.implicitHeight + Theme.spacingXS, 22)

            DankIcon {
                id: icon
                anchors.centerIn: parent
                name: "schedule"
                color: root.ready ? Theme.widgetTextColor : Theme.surfaceVariantText
                font.pixelSize: root.iconSize
            }

            MouseArea {
                anchors.fill: parent
                z: 10
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                cursorShape: Qt.PointingHandCursor
                onPressed: mouse => {
                    if (mouse.button === Qt.LeftButton) {
                        if (!openWebsite.running)
                            openWebsite.running = true;
                    } else if (mouse.button === Qt.MiddleButton) {
                        root.triggerPopout();
                    }
                }
            }
        }
    }

    popoutContent: Component {
        PopoutComponent {
            width: parent.width
            headerText: "TapTime"
            detailsText: root.data.date || "Нет данных"

            Column {
                width: parent.width
                spacing: Theme.spacingM

                TapTimeDetailRow {
                    label: "Начало — конец"
                    value: (root.data.start_time || "—") + " — "
                         + (root.data.active === true ? "В работе" : (root.data.end_time || "—"))
                    highlight: root.data.active === true
                }

                TapTimeDetailRow {
                    label: "Отработано"
                    value: root.data.worked || "--:--"
                }

                TapTimeDetailRow {
                    label: "Баланс за день"
                    value: root.data.balance || "--:--"
                    highlight: root.data.balance_positive === true
                }

                TapTimeDetailRow {
                    label: "Баланс за месяц"
                    value: root.data.month_balance || "--:--"
                    highlight: root.data.month_balance_positive === true
                }

                StyledText {
                    width: parent.width
                    visible: !root.ready
                    text: "Нет связи с TapTime daemon"
                    color: Theme.surfaceVariantText
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
