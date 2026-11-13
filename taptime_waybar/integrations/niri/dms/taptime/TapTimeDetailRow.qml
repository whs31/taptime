import QtQuick
import qs.Common
import qs.Widgets

Item {
    id: root
    property string label: ""
    property string value: "—"
    property bool highlight: false

    width: parent ? parent.width : 280
    implicitHeight: Math.max(name.implicitHeight, number.implicitHeight) + Theme.spacingXS

    StyledText {
        id: name
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        text: root.label
        color: Theme.surfaceVariantText
        font.pixelSize: Theme.fontSizeMedium
    }

    StyledText {
        id: number
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        text: root.value
        color: root.highlight ? Theme.primary : Theme.surfaceText
        font.pixelSize: Theme.fontSizeMedium
        font.bold: true
    }
}
