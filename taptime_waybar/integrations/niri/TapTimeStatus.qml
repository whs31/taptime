// Reusable data source for any Quickshell bar running under Niri.
// UI, popup and theme are intentionally provided by the host shell.
import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root
  visible: false
  implicitWidth: 0
  implicitHeight: 0

  property var status: ({
    text: "󰥔 --:--",
    worked: "--:--",
    balance: "--:--",
    month_balance: "--:--",
    start_time: null,
    end_time: null,
    active: false,
    tooltip: "TapTime: ожидание данных",
    class: ["unavailable"]
  })
  property bool stopping: false

  function applyLine(line) {
    if (line.length > 8192) return
    try {
      const parsed = JSON.parse(line)
      if (typeof parsed !== "object" || parsed === null ||
          typeof parsed.text !== "string" ||
          !Array.isArray(parsed.class)) return
      root.status = parsed
    } catch (error) {
      root.status = {
        text: "󰥔 --:--", worked: "--:--", balance: "--:--",
        month_balance: "--:--", start_time: null, end_time: null,
        active: false, class: ["unavailable"],
        tooltip: "TapTime: некорректный ответ"
      }
    }
  }

  Process {
    id: stream
    command: [Quickshell.env("HOME") + "/.local/bin/taptime-waybar", "watch"]
    running: true
    stdout: SplitParser {
      onRead: function(line) { root.applyLine(line) }
    }
    onExited: if (!root.stopping) retry.start()
  }

  Timer {
    id: retry
    interval: 5000
    onTriggered: if (!stream.running) stream.running = true
  }

  Component.onDestruction: stopping = true
}
