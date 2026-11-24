import QtQuick

// Omarchy service entry point; the data source lives in TapTimeStatus.qml.
Item {
  id: root

  property var shell: null
  readonly property var status: source.status

  function openDashboard() {
    source.openDashboard()
  }

  TapTimeStatus {
    id: source
  }
}
