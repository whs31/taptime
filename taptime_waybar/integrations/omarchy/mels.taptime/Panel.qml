import QtQuick
import Quickshell
import qs.Commons
import qs.Commons as Commons
import qs.Ui

Panel {
  id: root
  moduleName: "mels.taptime"
  manageIpc: false

  property var anchorItem: null
  property var hostWidget: null

  readonly property var current: hostWidget ? hostWidget.current : ({})
  readonly property string worked: current.worked || "--:--"
  readonly property string balance: current.balance || "--:--"
  readonly property string monthBalance: current.month_balance || "--:--"

  function shortDate(value) {
    var parts = String(value || "").match(/^\d{4}-(\d{2})-(\d{2})$/)
    return parts ? parts[2] + "." + parts[1] : String(value || "")
  }

  function open() { root.controller.show() }
  function close() { root.controller.hide() }

  function switchPanel(direction) {
    if (root.bar && typeof root.bar.switchPanelFrom === "function")
      return root.bar.switchPanelFrom(root.hostWidget || root, direction)
    return false
  }

  KeyboardPanel {
    id: popup
    anchorItem: root.anchorItem
    owner: root.hostWidget || root
    bar: root.bar
    open: root.opened
    focusTarget: keyCatcher
    contentWidth: popup.fittedContentWidth(Style.space(250))
    contentHeight: popup.fittedContentHeight(content.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }

      Column {
        id: content
        width: parent.width
        spacing: Style.space(10)

        Row {
          width: parent.width
          spacing: Style.space(8)

          Text {
            id: titleText
            text: "TapTime"
            textFormat: Text.PlainText
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
            font.bold: true
            color: Commons.Color.popups.text
          }

          Item {
            width: Math.max(0, parent.width - titleText.implicitWidth
              - dateText.implicitWidth - parent.spacing)
            height: 1
          }

          Text {
            id: dateText
            text: root.shortDate(root.current.date)
            textFormat: Text.PlainText
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
            color: Commons.Color.muted
          }
        }

        Text {
          width: parent.width
          text: (root.current.start_time || "—") + "  —  "
            + (root.current.active === true
              ? "В работе" : (root.current.end_time || "Нет выхода"))
          textFormat: Text.PlainText
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          color: root.current.active === true
            ? Commons.Color.accent : Commons.Color.popups.text
          elide: Text.ElideRight
        }

        Column {
          width: parent.width
          spacing: Style.space(6)

          Row {
            width: parent.width
            Text {
              id: workedLabel
              text: "Отработано"
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: Commons.Color.muted
            }
            Item {
              width: Math.max(0, parent.width - workedLabel.implicitWidth
                - workedValue.implicitWidth)
              height: 1
            }
            Text {
              id: workedValue
              text: root.worked
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: Commons.Color.popups.text
            }
          }

          Row {
            width: parent.width
            Text {
              id: dayLabel
              text: "Баланс за день"
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: Commons.Color.muted
            }
            Item {
              width: Math.max(0, parent.width - dayLabel.implicitWidth
                - dayValue.implicitWidth)
              height: 1
            }
            Text {
              id: dayValue
              text: root.balance
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: root.current.balance_positive === true
                ? Commons.Color.accent : Commons.Color.popups.text
            }
          }

          Row {
            width: parent.width
            Text {
              id: monthLabel
              text: "Баланс за месяц"
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: Commons.Color.muted
            }
            Item {
              width: Math.max(0, parent.width - monthLabel.implicitWidth
                - monthValue.implicitWidth)
              height: 1
            }
            Text {
              id: monthValue
              text: root.monthBalance
              textFormat: Text.PlainText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              color: root.current.month_balance_positive === true
                ? Commons.Color.accent : Commons.Color.popups.text
            }
          }
        }
      }
    }
  }
}
