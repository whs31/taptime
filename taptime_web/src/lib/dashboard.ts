import type { Duration } from "@bufbuild/protobuf";
import type { Balance } from "@taptime/proto/taptime/balance_pb.js";
import { Date as ProtoDate } from "@taptime/proto/taptime/date_pb.js";
import { DayFlag, type Day } from "@taptime/proto/taptime/day_pb.js";
import type { Event } from "@taptime/proto/taptime/event_pb.js";
import { LocalTime } from "@taptime/proto/taptime/local_time_pb.js";
import type { DaySummary } from "@taptime/proto/taptime/services/store_pb.js";

export const MS_PER_DAY = 86_400_000;
export const SECONDS_PER_DAY = 86_400;

export type ManualEventType = "checkIn" | "checkOut";

export type Session = {
  checkIn: number | null;
  checkOut: number | null;
  duration: number | null;
};

export type ChartPoint = {
  day: number;
  seconds: number;
  kind: "checkIn" | "checkOut";
  label: string;
};

export type MonthRhythmModel = {
  monthStart: number;
  monthEnd: number;
  daysInMonth: number;
  firstLine: ChartPoint[];
  lastLine: ChartPoint[];
  points: ChartPoint[];
};

export type RangeTotals = {
  totalClocked: number;
  overtime: number;
  undertime: number;
  workedDays: number;
  remoteDays: number;
  dayOffs: number;
  vacationDays: number;
  skippedDays: number;
  fullWeekendWorkDays: number;
  fullVacationWorkDays: number;
};

export type MonthlyBucket = RangeTotals & {
  key: string;
  monthStart: number;
  label: string;
};

export type ExceptionRow = {
  key: number;
  date: number;
  label: string;
  kind: string;
  clocked: number;
  required: number;
  balance: number;
};

export function protoDate(daysSinceEpoch: number) {
  return new ProtoDate({ daysSinceEpoch });
}

export function todayDate(timeZone: string) {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(new Date());
  const y = parseInt(parts.find((p) => p.type === "year")?.value ?? "1970");
  const mo = parseInt(parts.find((p) => p.type === "month")?.value ?? "1");
  const d = parseInt(parts.find((p) => p.type === "day")?.value ?? "1");
  return protoDate(Math.floor(Date.UTC(y, mo - 1, d) / MS_PER_DAY));
}

export function currentTimeParts(timeZone: string) {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  }).formatToParts(new Date());
  return {
    h: parseInt(parts.find((p) => p.type === "hour")?.value ?? "0") % 24,
    m: parseInt(parts.find((p) => p.type === "minute")?.value ?? "0"),
    s: parseInt(parts.find((p) => p.type === "second")?.value ?? "0"),
  };
}

export function currentTimeValue(timeZone: string) {
  const { h, m } = currentTimeParts(timeZone);
  return `${pad(h)}:${pad(m)}`;
}

export function parseManualTime(value: string): LocalTime | null {
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour < 0 || hour > 23 || minute < 0 || minute > 59) return null;
  return new LocalTime({ hour, minute, second: 0 });
}

export function nowSeconds(timeZone: string) {
  const { h, m, s } = currentTimeParts(timeZone);
  return h * 3600 + m * 60 + s;
}

export function dayKey(value?: Day): number | null {
  return value?.date?.daysSinceEpoch ?? null;
}

export function addDays(daysSinceEpoch: number, amount: number) {
  return daysSinceEpoch + amount;
}

export function monthStartDay(daysSinceEpoch: number) {
  const date = new Date(daysSinceEpoch * MS_PER_DAY);
  return Math.floor(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / MS_PER_DAY,
  );
}

export function monthEndDay(daysSinceEpoch: number) {
  const date = new Date(daysSinceEpoch * MS_PER_DAY);
  return Math.floor(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0) / MS_PER_DAY,
  );
}

export function yearStartDay(daysSinceEpoch: number) {
  const date = new Date(daysSinceEpoch * MS_PER_DAY);
  return Math.floor(Date.UTC(date.getUTCFullYear(), 0, 1) / MS_PER_DAY);
}

export function previousMonthStartDay(daysSinceEpoch: number) {
  const date = new Date(daysSinceEpoch * MS_PER_DAY);
  return Math.floor(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth() - 1, 1) / MS_PER_DAY,
  );
}

export function quarterStartDay(daysSinceEpoch: number) {
  const date = new Date(daysSinceEpoch * MS_PER_DAY);
  const month = Math.floor(date.getUTCMonth() / 3) * 3;
  return Math.floor(Date.UTC(date.getUTCFullYear(), month, 1) / MS_PER_DAY);
}

export function mondayFirstDayOfWeek(daysSinceEpoch: number) {
  return (new Date(daysSinceEpoch * MS_PER_DAY).getUTCDay() + 6) % 7;
}

export function dateLabel(daysSinceEpoch: number, compact = false) {
  return new Intl.DateTimeFormat("en-US", {
    timeZone: "UTC",
    weekday: compact ? undefined : "short",
    month: "short",
    day: "numeric",
  }).format(new Date(daysSinceEpoch * MS_PER_DAY));
}

export function fullDateLabel(daysSinceEpoch: number) {
  return new Intl.DateTimeFormat("en-US", {
    timeZone: "UTC",
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(daysSinceEpoch * MS_PER_DAY));
}

export function monthLabel(daysSinceEpoch: number) {
  return new Intl.DateTimeFormat("en-US", {
    timeZone: "UTC",
    month: "long",
    year: "numeric",
  }).format(new Date(daysSinceEpoch * MS_PER_DAY));
}

export function monthShortLabel(daysSinceEpoch: number) {
  return new Intl.DateTimeFormat("en-US", {
    timeZone: "UTC",
    month: "short",
  }).format(new Date(daysSinceEpoch * MS_PER_DAY));
}

export function rangeLabel(start: number, end: number) {
  return `${dateLabel(start, true)} - ${dateLabel(end, true)}`;
}

export function buildSummaryMap(items: DaySummary[]) {
  const map = new Map<number, DaySummary>();
  for (const summary of items) {
    const key = dayKey(summary.day);
    if (key !== null) map.set(key, summary);
  }
  return map;
}

export function durationSeconds(duration?: Duration): number {
  return Number(duration?.seconds ?? 0n);
}

export function ltToSeconds(lt: { hour: number; minute: number; second: number }) {
  return lt.hour * 3600 + lt.minute * 60 + lt.second;
}

export function eventSeconds(event: Event): number | null {
  if (
    event.eventType.case !== "checkIn" &&
    event.eventType.case !== "checkOut"
  ) {
    return null;
  }
  return ltToSeconds(event.eventType.value);
}

export function pad(n: number) {
  return String(n).padStart(2, "0");
}

export function formatSeconds(secs: number) {
  const sign = secs < 0 ? "-" : "";
  const s = Math.max(0, Math.floor(Math.abs(secs)));
  return `${sign}${pad(Math.floor(s / 3600))}:${pad(Math.floor((s % 3600) / 60))}:${pad(s % 60)}`;
}

export function formatHours(secs: number) {
  const sign = secs < 0 ? "-" : "";
  const abs = Math.abs(secs);
  const hours = Math.floor(abs / 3600);
  const minutes = Math.floor((abs % 3600) / 60);
  if (hours === 0) return `${sign}${minutes}m`;
  if (minutes === 0) return `${sign}${hours}h`;
  return `${sign}${hours}h ${minutes}m`;
}

export function formatTime(seconds: number | null) {
  if (seconds === null) return "--:--";
  return `${pad(Math.floor(seconds / 3600))}:${pad(Math.floor((seconds % 3600) / 60))}`;
}

export function computeWorkSeconds(d: Day | null | undefined, currentSeconds?: number) {
  if (!d) return 0;

  let total = 0;
  let openCheckInSecs: number | null = null;

  for (const event of d.events) {
    if (event.eventType.case === "checkIn") {
      openCheckInSecs = ltToSeconds(event.eventType.value);
      continue;
    }

    if (event.eventType.case === "checkOut" && openCheckInSecs !== null) {
      total += Math.max(0, ltToSeconds(event.eventType.value) - openCheckInSecs);
      openCheckInSecs = null;
    }
  }

  if (openCheckInSecs !== null && currentSeconds !== undefined) {
    total += Math.max(0, currentSeconds - openCheckInSecs);
  }

  return total;
}

export function buildSessions(
  d: Day | null | undefined,
  currentSeconds?: number,
): Session[] {
  if (!d) return [];

  const sessions: Session[] = [];
  let openCheckIn: number | null = null;

  for (const event of d.events) {
    const seconds = eventSeconds(event);
    if (seconds === null) continue;
    if (event.eventType.case === "checkIn") {
      openCheckIn = seconds;
    } else if (openCheckIn !== null) {
      sessions.push({
        checkIn: openCheckIn,
        checkOut: seconds,
        duration: Math.max(0, seconds - openCheckIn),
      });
      openCheckIn = null;
    } else {
      sessions.push({ checkIn: null, checkOut: seconds, duration: null });
    }
  }

  if (openCheckIn !== null) {
    sessions.push({
      checkIn: openCheckIn,
      checkOut: null,
      duration:
        currentSeconds === undefined ? null : Math.max(0, currentSeconds - openCheckIn),
    });
  }

  return sessions;
}

export function nextManualEventType(events: Event[]): ManualEventType {
  return events.length > 0 &&
    events[events.length - 1].eventType.case === "checkIn"
    ? "checkOut"
    : "checkIn";
}

export function hasFlag(summary: DaySummary | null | undefined, flag: DayFlag) {
  return Boolean(summary?.day && (summary.day.flags & flag) === flag);
}

export function hasDayFlag(day: Day | null | undefined, flag: DayFlag) {
  return Boolean(day && (day.flags & flag) === flag);
}

export function isRegularRequiredDay(day: Day | null | undefined) {
  if (!day) return false;
  const nonRegularFlags =
    DayFlag.WEEKEND | DayFlag.DAY_OFF | DayFlag.VACATION | DayFlag.REMOTE;
  return (day.flags & nonRegularFlags) === 0;
}

export function serverBalanceSeconds(balance?: Balance): number {
  switch (balance?.balanceType.case) {
    case "overtime":
      return durationSeconds(balance.balanceType.value);
    case "underTime":
      return -durationSeconds(balance.balanceType.value);
    default:
      return 0;
  }
}

export function summaryClockedSeconds(
  summary: DaySummary | null | undefined,
  liveDay?: number,
  liveSeconds?: number,
) {
  if (!summary) return 0;
  return liveDay !== undefined && dayKey(summary.day) === liveDay
    ? (liveSeconds ?? 0)
    : durationSeconds(summary.clockedWork);
}

export function liveBalanceSeconds(
  summary: DaySummary | null | undefined,
  liveDay?: number,
  liveSeconds?: number,
) {
  if (!summary?.day || !isRegularRequiredDay(summary.day)) {
    return serverBalanceSeconds(summary?.balance);
  }
  return (
    summaryClockedSeconds(summary, liveDay, liveSeconds) -
    durationSeconds(summary.day.requiredWorkHours)
  );
}

export function balanceLabel(
  summary: DaySummary | null | undefined,
  liveDay?: number,
  liveSeconds?: number,
) {
  if (!summary) return "No data";
  if (summary.skipped) return "Skipped";
  const delta = liveBalanceSeconds(summary, liveDay, liveSeconds);
  if (delta > 0) return `Overtime ${formatHours(delta)}`;
  if (delta < 0) return `Undertime ${formatHours(delta)}`;
  return "Exact";
}

export function dayKindLabel(summary: DaySummary | null | undefined) {
  if (!summary?.day) return "No day";
  const labels = [];
  if (hasFlag(summary, DayFlag.WEEKEND)) labels.push("Weekend");
  if (hasFlag(summary, DayFlag.REMOTE)) labels.push("Remote");
  if (hasFlag(summary, DayFlag.DAY_OFF)) labels.push("Day off");
  if (hasFlag(summary, DayFlag.VACATION)) labels.push("Vacation");
  return labels.length > 0 ? labels.join(", ") : "Regular day";
}

export function flagLabels(summary: DaySummary | null | undefined) {
  if (!summary?.day) return [];
  const labels: string[] = [];
  if (hasFlag(summary, DayFlag.WEEKEND)) labels.push("Weekend");
  if (hasFlag(summary, DayFlag.REMOTE)) labels.push("Remote");
  if (hasFlag(summary, DayFlag.DAY_OFF)) labels.push("Day off");
  if (hasFlag(summary, DayFlag.VACATION)) labels.push("Vacation");
  return labels;
}

export function firstCheckIn(summary: DaySummary | null | undefined) {
  const event = summary?.day?.events.find(
    (event) => event.eventType.case === "checkIn",
  );
  return event ? eventSeconds(event) : null;
}

export function lastCheckOut(summary: DaySummary | null | undefined) {
  const event = [...(summary?.day?.events ?? [])]
    .reverse()
    .find((event) => event.eventType.case === "checkOut");
  return event ? eventSeconds(event) : null;
}

export function balanceContribution(clocked: number, required: number) {
  const delta = clocked - required;
  return {
    overtime: Math.max(0, delta),
    undertime: Math.max(0, -delta),
  };
}

export function summarizeRange(items: DaySummary[]): RangeTotals {
  const totals: RangeTotals = {
    totalClocked: 0,
    overtime: 0,
    undertime: 0,
    workedDays: 0,
    remoteDays: 0,
    dayOffs: 0,
    vacationDays: 0,
    skippedDays: 0,
    fullWeekendWorkDays: 0,
    fullVacationWorkDays: 0,
  };

  for (const summary of items) {
    const clocked = durationSeconds(summary.clockedWork);
    totals.totalClocked += clocked;
    if (clocked > 0 || (summary.day?.events.length ?? 0) > 0) totals.workedDays += 1;
    if (hasFlag(summary, DayFlag.REMOTE)) totals.remoteDays += 1;
    if (hasFlag(summary, DayFlag.DAY_OFF)) totals.dayOffs += 1;
    if (hasFlag(summary, DayFlag.VACATION)) totals.vacationDays += 1;
    if (summary.skipped) totals.skippedDays += 1;
    if (hasFlag(summary, DayFlag.WEEKEND) && summary.fullDayWorked) {
      totals.fullWeekendWorkDays += 1;
    }
    if (hasFlag(summary, DayFlag.VACATION) && summary.fullDayWorked) {
      totals.fullVacationWorkDays += 1;
    }
    if (isRegularRequiredDay(summary.day)) {
      const balance = serverBalanceSeconds(summary.balance);
      if (balance > 0) totals.overtime += balance;
      if (balance < 0) totals.undertime += Math.abs(balance);
    }
  }

  return totals;
}

export function monthlyBuckets(items: DaySummary[]): MonthlyBucket[] {
  const buckets = new Map<number, DaySummary[]>();
  for (const summary of items) {
    const key = dayKey(summary.day);
    if (key === null) continue;
    const monthStart = monthStartDay(key);
    const list = buckets.get(monthStart) ?? [];
    list.push(summary);
    buckets.set(monthStart, list);
  }

  return [...buckets.entries()]
    .sort(([a], [b]) => a - b)
    .map(([monthStart, summaries]) => ({
      key: String(monthStart),
      monthStart,
      label: monthShortLabel(monthStart),
      ...summarizeRange(summaries),
    }));
}

export function exceptionRows(items: DaySummary[]): ExceptionRow[] {
  return items
    .flatMap((summary) => {
      const key = dayKey(summary.day);
      if (key === null || !summary.day) return [];
      const clocked = durationSeconds(summary.clockedWork);
      const required = durationSeconds(summary.day.requiredWorkHours);
      const balance = serverBalanceSeconds(summary.balance);
      const rows: ExceptionRow[] = [];

      if (summary.skipped) {
        rows.push({
          key,
          date: key,
          label: "Skipped day",
          kind: "Skipped",
          clocked,
          required,
          balance,
        });
      }
      if (isRegularRequiredDay(summary.day) && Math.abs(balance) >= 60 * 60) {
        rows.push({
          key,
          date: key,
          label: balance > 0 ? "Large overtime" : "Large undertime",
          kind: balance > 0 ? "Overtime" : "Undertime",
          clocked,
          required,
          balance,
        });
      }
      if (hasFlag(summary, DayFlag.WEEKEND) && summary.fullDayWorked) {
        rows.push({
          key,
          date: key,
          label: "Weekend work",
          kind: "Weekend",
          clocked,
          required,
          balance,
        });
      }
      if (hasFlag(summary, DayFlag.VACATION) && summary.fullDayWorked) {
        rows.push({
          key,
          date: key,
          label: "Vacation work",
          kind: "Vacation",
          clocked,
          required,
          balance,
        });
      }

      return rows;
    })
    .sort((a, b) => b.date - a.date);
}

export function buildMonthRhythmModel(
  items: DaySummary[],
  currentDay: number,
): MonthRhythmModel {
  const monthStart = monthStartDay(currentDay);
  const monthEnd = monthEndDay(currentDay);
  const daysInMonth = monthEnd - monthStart + 1;
  const firstLine: ChartPoint[] = [];
  const lastLine: ChartPoint[] = [];
  const points: ChartPoint[] = [];

  for (const summary of items) {
    const key = dayKey(summary.day);
    if (key === null || !summary.day) continue;
    const dayNumber = key - monthStart + 1;
    const dayEvents = summary.day.events
      .map((event) => {
        const seconds = eventSeconds(event);
        if (seconds === null) return null;
        return {
          day: dayNumber,
          seconds,
          kind: event.eventType.case as "checkIn" | "checkOut",
          label: `${dateLabel(key, true)} ${event.eventType.case === "checkIn" ? "in" : "out"} ${formatTime(seconds)}`,
        };
      })
      .filter((event): event is ChartPoint => event !== null);

    const firstIn = dayEvents.find((event) => event.kind === "checkIn");
    const lastOut = [...dayEvents]
      .reverse()
      .find((event) => event.kind === "checkOut");
    if (firstIn) firstLine.push(firstIn);
    if (lastOut) lastLine.push(lastOut);
    points.push(...dayEvents);
  }

  return { monthStart, monthEnd, daysInMonth, firstLine, lastLine, points };
}

export function chartX(dayNumber: number, daysInMonth: number) {
  const left = 46;
  const width = 626;
  if (daysInMonth <= 1) return left;
  return left + ((dayNumber - 1) / (daysInMonth - 1)) * width;
}

export function chartY(seconds: number) {
  const top = 16;
  const height = 184;
  return top + (1 - seconds / SECONDS_PER_DAY) * height;
}

export function linePath(points: ChartPoint[], daysInMonth: number) {
  return points
    .map(
      (point, index) =>
        `${index === 0 ? "M" : "L"} ${chartX(point.day, daysInMonth).toFixed(2)} ${chartY(point.seconds).toFixed(2)}`,
    )
    .join(" ");
}
