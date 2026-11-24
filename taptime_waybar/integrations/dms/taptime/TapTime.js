.pragma library

// Pure helpers shared by the bar pill, popout and dash card. `s` is the
// status object published by TapTimeDaemon (see TapTimeStatus.qml).

function num(value) {
    return typeof value === "number" && isFinite(value) ? value : null;
}

// 19186 -> "05:19"; null -> "--:--". Matches taptime-waybar's default format.
function clock(seconds) {
    const n = num(seconds);
    if (n === null)
        return "--:--";
    const minutes = Math.floor(Math.abs(n) / 60);
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return (h < 10 ? "0" + h : h) + ":" + (m < 10 ? "0" + m : m);
}

function ratio(part, whole) {
    const p = num(part), w = num(whole);
    return p === null || w === null || w <= 0 ? 0 : Math.max(0, p / w);
}

// Worked time against the work target (the hero ring).
function workProgress(s) {
    return ratio(s.seconds.worked, s.seconds.work_target);
}

// Presence against the required day including lunch (the pill ring, timeline).
function dayProgress(s) {
    return ratio(s.seconds.presence, s.seconds.presence_target);
}

// One of: loading, broken, disconnected, signedOut, idle, working, overtime, away.
function phase(s) {
    if (!s || s.loading)
        return "loading";
    if (s.broken)
        return "broken";
    if (s.disconnected)
        return "disconnected";
    if (s.error === "unauthenticated")
        return "signedOut";
    if (s.active)
        return num(s.seconds.remaining) === 0 ? "overtime" : "working";
    return s.start_time ? "away" : "idle";
}

// Time in the office that didn't count as work.
function breakSeconds(s) {
    const presence = num(s.seconds.presence), worked = num(s.seconds.worked);
    return presence === null || worked === null ? null : Math.max(0, presence - worked);
}

function isProblem(phase) {
    return phase === "broken" || phase === "disconnected" || phase === "signedOut";
}

function isStale(s) {
    return s.alt !== "ready";
}

// When presence reaches the daily target, as a Date. Past for overtime.
function targetTime(s, now) {
    const remaining = num(s.seconds.remaining);
    const presence = num(s.seconds.presence);
    const target = num(s.seconds.presence_target);
    if (!s.active || remaining === null)
        return null;
    if (remaining > 0)
        return new Date(now.getTime() + remaining * 1000);
    if (presence === null || target === null)
        return null;
    return new Date(now.getTime() - (presence - target) * 1000);
}

function hhmm(date) {
    if (!date)
        return "--:--";
    const h = date.getHours(), m = date.getMinutes();
    return (h < 10 ? "0" + h : h) + ":" + (m < 10 ? "0" + m : m);
}

// "YYYY-MM-DD" -> local Date, or null.
function parseDate(text) {
    const parts = /^(\d{4})-(\d{2})-(\d{2})$/.exec(text || "");
    return parts ? new Date(+parts[1], +parts[2] - 1, +parts[3]) : null;
}

// Seconds since an RFC 3339 timestamp, or null.
function ageSeconds(timestamp, now) {
    const t = Date.parse(timestamp || "");
    return isNaN(t) ? null : Math.max(0, Math.round((now.getTime() - t) / 1000));
}

function signOf(seconds) {
    const n = num(seconds);
    return n === null ? 0 : n > 0 ? 1 : n < 0 ? -1 : 0;
}
