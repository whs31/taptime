// UI-neutral TapTime data source shared by the Quickshell frontends
// (DankMaterialShell, Omarchy Shell). Streams `taptime-waybar watch` and
// exposes the latest line as `status`; theme and layout belong to the host.
// The install recipes copy this file next to each plugin's QML.
import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root
  visible: false
  implicitWidth: 0
  implicitHeight: 0

  readonly property string executable: Quickshell.env("HOME") + "/.local/bin/taptime-waybar"
  property var status: placeholder("TapTime: ожидание данных")
  property bool stopping: false
  // Whether the current `watch` process has produced a line yet.
  property bool received: false

  function placeholder(tooltip) {
    return {
      text: "󰥔 --:--",
      tooltip: tooltip,
      class: ["unavailable"],
      alt: "stale",
      active: false,
      date: "",
      start_time: null,
      end_time: null,
      worked: "--:--",
      balance: "--:--",
      balance_positive: false,
      month_balance: "--:--",
      month_balance_positive: false,
      seconds: {},
      error: null,
      disconnected: false,
      last_sync: null,
      // Frontend-only flags: no line received yet / the stream keeps failing.
      loading: true,
      broken: false
    }
  }

  function apply(line) {
    if (line.length > 8192) return
    let next
    try {
      next = JSON.parse(line)
    } catch (error) {
      root.status = placeholder("TapTime: некорректный ответ")
      return
    }
    if (typeof next !== "object" || next === null ||
        typeof next.text !== "string" || typeof next.tooltip !== "string" ||
        !Array.isArray(next.class)) return
    root.status = {
      text: next.text,
      tooltip: next.tooltip,
      class: next.class,
      alt: next.alt === "ready" ? "ready" : "stale",
      active: next.active === true,
      date: next.date || "",
      start_time: next.start_time || null,
      end_time: next.end_time || null,
      worked: next.worked || "--:--",
      balance: next.balance || "--:--",
      balance_positive: next.balance_positive === true,
      month_balance: next.month_balance || "--:--",
      month_balance_positive: next.month_balance_positive === true,
      seconds: typeof next.seconds === "object" && next.seconds !== null ? next.seconds : {},
      error: typeof next.error === "string" ? next.error : null,
      disconnected: next.disconnected === true,
      last_sync: next.last_sync || null,
      loading: false,
      broken: false
    }
    root.received = true
  }

  // Opens display.dashboard_url from config.toml.
  function openDashboard() {
    if (!opener.running) opener.running = true
  }

  Process {
    id: stream
    command: [root.executable, "watch"]
    running: true
    stdout: SplitParser {
      onRead: line => root.apply(line)
    }
    onStarted: root.received = false
    onExited: {
      if (root.stopping) return
      // Exiting before the first line means the binary is missing or the
      // config is invalid, not just a disconnected daemon.
      if (!root.received) {
        const failed = placeholder("TapTime: taptime-waybar failed to start")
        failed.loading = false
        failed.broken = true
        root.status = failed
      }
      retry.start()
    }
  }

  Timer {
    id: retry
    interval: 5000
    onTriggered: if (!stream.running) stream.running = true
  }

  Process {
    id: opener
    command: [root.executable, "open"]
  }

  Component.onDestruction: stopping = true
}
