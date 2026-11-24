import QtQuick
import qs.Commons as Commons
import qs.Ui

BarWidget {
  id: root
  moduleName: "mels.taptime"

  readonly property var tapService: bar && bar.shell
    ? bar.shell.serviceFor(moduleName) : null
  readonly property var current: tapService
    ? tapService.status : ({ text: "󰥔 --:--", class: ["stale"], alt: "stale" })

  readonly property bool positive: current.balance_positive === true
  readonly property bool stale: current.alt !== "ready"

  readonly property bool opened: panelLoader.item
    ? panelLoader.item.opened === true : false
  readonly property bool popoutSwitchClosing: panelLoader.item
    ? panelLoader.item.popoutSwitchClosing === true : false

  function open() {
    if (panelLoader.item) panelLoader.item.open()
  }

  function close() {
    if (panelLoader.item) panelLoader.item.close()
  }

  function toggle() {
    if (panelLoader.item) panelLoader.item.toggle()
  }

  function closeForPopoutSwitch() {
    if (panelLoader.item) panelLoader.item.closeForPopoutSwitch()
  }

  function injectPanel() {
    if (!panelLoader.item) return
    panelLoader.item.bar = root.bar
    panelLoader.item.anchorItem = button
    panelLoader.item.hostWidget = root
  }

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight
  onBarChanged: injectPanel()

  Loader {
    id: panelLoader
    active: true
    source: Qt.resolvedUrl("Panel.qml")
    visible: false
    onLoaded: {
      root.injectPanel()
      Qt.callLater(root.injectPanel)
    }
  }

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.bar && root.bar.vertical ? "󰥔" : root.current.text
    tooltipText: ""

    foreground: root.stale ? Commons.Color.muted
      : root.positive ? Commons.Color.accent
      : root.bar ? root.bar.barForeground : Commons.Color.foreground

    onPressed: function(mouseButton) {
      if (mouseButton === Qt.LeftButton) {
        root.close()
        if (root.tapService)
          root.tapService.openDashboard()
      } else if (mouseButton === Qt.RightButton ||
                 mouseButton === Qt.MiddleButton) {
        root.toggle()
      }
    }
  }
}
