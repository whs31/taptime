import QtQuick
import qs.Common
import qs.DCommon.Widgets

// Bar glyph: a progress ring around a dot that is filled while checked in.
// Problem phases swap the ring for an icon.
Item {
    id: root

    required property var look
    property real size: Theme.iconSize
    property bool showRing: true

    implicitWidth: size
    implicitHeight: size

    DRingGauge {
        id: ring
        anchors.fill: parent
        visible: root.showRing && !root.look.problem && root.look.phase !== "loading"
        value: Math.min(1, root.look.dayProgress)
        strokeWidth: Math.max(2, Math.round(root.size / 7))
        ringColor: root.look.accent
        trackColor: Theme.withAlpha(root.look.accent, 0.22)

        Rectangle {
            anchors.centerIn: parent
            width: Math.max(4, Math.round(ring.innerSize * 0.42))
            height: width
            radius: width / 2
            color: root.look.checkedIn ? root.look.accent : "transparent"
            border.width: root.look.checkedIn ? 0 : Math.max(1, Math.round(root.size / 14))
            border.color: root.look.accent

            Behavior on color {
                ColorAnimation {
                    duration: Theme.shortDuration
                }
            }
        }
    }

    DIcon {
        anchors.centerIn: parent
        visible: !ring.visible
        name: root.look.icon
        size: root.size
        color: root.look.problem ? root.look.accent : Theme.widgetTextColor
    }
}
