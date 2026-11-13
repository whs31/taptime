import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root

  property var shell: null
  property var status: ({ text: "󰥔 --:--", tooltip: "TapTime: ожидание данных", class: ["unavailable"] })
  property bool stopping: false
  function apply(line) {
    if (line.length > 2048) return
    try {
      var next = JSON.parse(line)
      if (typeof next.text !== "string" || typeof next.tooltip !== "string" || !Array.isArray(next.class)) return
      if (next.text.length > 80 || next.tooltip.length > 1024) return
      status = {
        text: next.text,
        tooltip: next.tooltip,
        class: next.class,
        active: next.active === true,
        date: next.date || "",
        start_time: next.start_time || null,
        end_time: next.end_time || null,
        worked: next.worked || "--:--",
        balance: next.balance || "--:--",
        balance_positive: next.balance_positive === true,
        month_balance_positive: next.month_balance_positive === true,
        month_balance: next.month_balance || "--:--"
      }
    } catch (error) {
      status = { text: "󰥔 --:--", tooltip: "Некорректный ответ TapTime", class: ["unavailable"] }
    }
  }

  Process {
    id: stream
    command: [Quickshell.env("HOME") + "/.local/bin/taptime-waybar", "watch"]
    running: true
    stdout: SplitParser { onRead: function(line) { root.apply(line) } }
    onExited: if (!root.stopping) restart.start()
  }

  Timer {
    id: restart
    interval: 5000
    onTriggered: if (!stream.running) stream.running = true
  }

  Component.onDestruction: stopping = true
}
