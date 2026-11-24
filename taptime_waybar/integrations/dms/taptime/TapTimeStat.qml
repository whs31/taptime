import QtQuick
import qs.Common
import qs.Widgets
import qs.DCommon.Widgets

// A labelled value tile for the popout's stats row.
Rectangle {
    id: root

    property string icon: ""
    property string label: ""
    property string value: "--:--"
    property color valueColor: Theme.surfaceText

    implicitHeight: content.implicitHeight + Theme.spacingM * 2
    radius: Theme.cornerRadius
    color: Theme.foregroundColor(Theme.cardSurface)

    Column {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.spacingM
        spacing: Theme.spacingXS

        Row {
            width: parent.width
            spacing: Theme.spacingXS

            DIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: root.icon
                size: Theme.iconSizeSmall
                color: Theme.surfaceVariantText
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - Theme.iconSizeSmall - parent.spacing
                text: root.label
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceVariantText
                elide: Text.ElideRight
            }
        }

        NumericText {
            text: root.value
            font.pixelSize: Theme.fontSizeLarge
            font.weight: Theme.fontWeightMedium
            color: root.valueColor
            horizontalAlignment: Text.AlignLeft
        }
    }
}
