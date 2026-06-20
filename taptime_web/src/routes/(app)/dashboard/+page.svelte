<script lang="ts">
  import { onMount } from "svelte";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Progress } from "$lib/components/ui/progress/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import { userStore } from "$lib/stores";
  import { StoreService } from "$lib/services";
  import CalendarXIcon from "@lucide/svelte/icons/calendar-x";
  import LogInIcon from "@lucide/svelte/icons/log-in";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import { DayFlag, type Day } from "@taptime/proto/taptime/day_pb.js";
  import type { Balance } from "@taptime/proto/taptime/balance_pb.js";
  import type { Event } from "@taptime/proto/taptime/event_pb.js";
  import { Date as ProtoDate } from "@taptime/proto/taptime/date_pb.js";
  import type {
    DashboardResponse,
    DaySummary,
    MonthlyStats,
  } from "@taptime/proto/taptime/services/store_pb.js";
  import type { Duration } from "@bufbuild/protobuf";

  const MS_PER_DAY = 86_400_000;
  const SECONDS_PER_DAY = 86_400;

  type CalendarCell = DaySummary | null;
  type Session = {
    checkIn: number | null;
    checkOut: number | null;
    duration: number | null;
  };
  type ChartPoint = {
    day: number;
    seconds: number;
    kind: "checkIn" | "checkOut";
    label: string;
  };

  const tz = $derived(
    userStore.user?.timeZone?.timeZone ??
      Intl.DateTimeFormat().resolvedOptions().timeZone,
  );

  let dashboard = $state<DashboardResponse | null>(null);
  let loadError = $state<string | null>(null);
  let loading = $state(true);
  let submitting = $state(false);
  let flagUpdating = $state(false);
  let loadedWindowKey = $state("");
  let selectedDayKey = $state<number | null>(null);

  let currentTimeDisplay = $state("--:--:--");
  let workSeconds = $state(0);

  const summaries = $derived(dashboard?.days ?? []);
  const summaryByDay = $derived(buildSummaryMap(summaries));
  const todayDays = $derived(StoreService.currentDate(tz).daysSinceEpoch);
  const todaySummary = $derived(summaryByDay.get(todayDays) ?? null);
  const day = $derived(todaySummary?.day ?? null);
  const events = $derived(day?.events ?? []);
  const isCheckedIn = $derived(
    events.length > 0 && events[events.length - 1].eventType.case === "checkIn",
  );
  const hasCheckInToday = $derived(
    events.some((event) => event.eventType.case === "checkIn"),
  );
  const todayIsDayOff = $derived(hasFlag(todaySummary, DayFlag.DAY_OFF));
  const takeDayOffDisabled = $derived(
    loading || submitting || flagUpdating || hasCheckInToday || todayIsDayOff,
  );
  const takeDayOffLabel = $derived(todayIsDayOff ? "Day Off Set" : "Take Day Off");
  const requiredSeconds = $derived(durationSeconds(day?.requiredWorkHours));
  const progressPercent = $derived(
    requiredSeconds > 0
      ? Math.min(100, (workSeconds / requiredSeconds) * 100)
      : 0,
  );
  const monthStats = $derived(dashboard?.monthStats ?? null);
  const calendarCells = $derived(buildCalendarCells(summaries));
  const selectedSummary = $derived(
    selectedDayKey === null ? null : summaryByDay.get(selectedDayKey) ?? null,
  );
  const todaySessions = $derived(buildSessions(day, workSeconds));
  const monthSummaries = $derived(
    summaries.filter((summary) => {
      const key = dayKey(summary.day);
      return (
        key !== null &&
        key >= monthStartDay(todayDays) &&
        key <= monthEndDay(todayDays)
      );
    }),
  );
  const chartModel = $derived(buildChartModel(monthSummaries, todayDays));

  const flagControls = [
    { flag: DayFlag.WEEKEND, label: "Weekend" },
    { flag: DayFlag.REMOTE, label: "Remote" },
    { flag: DayFlag.DAY_OFF, label: "Day off" },
    { flag: DayFlag.VACATION, label: "Vacation" },
  ];

  const statItems = $derived(buildStatItems(monthStats, todaySummary));

  $effect(() => {
    const window = dashboardWindow(tz);
    const key = [
      window.rangeStart.daysSinceEpoch,
      window.rangeEnd.daysSinceEpoch,
      window.monthStart.daysSinceEpoch,
      window.monthEnd.daysSinceEpoch,
    ].join(":");
    if (key !== loadedWindowKey) {
      loadedWindowKey = key;
      void loadDashboard();
    }
  });

  function dashboardWindow(timeZone: string) {
    const today = StoreService.currentDate(timeZone);
    const rangeEnd = today;
    const rangeStart = new ProtoDate({
      daysSinceEpoch: today.daysSinceEpoch - 364,
    });
    const monthStart = new ProtoDate({
      daysSinceEpoch: monthStartDay(today.daysSinceEpoch),
    });
    const monthEnd = new ProtoDate({
      daysSinceEpoch: monthEndDay(today.daysSinceEpoch),
    });
    return { rangeStart, rangeEnd, monthStart, monthEnd, today };
  }

  function buildSummaryMap(items: DaySummary[]) {
    const map = new Map<number, DaySummary>();
    for (const summary of items) {
      const key = dayKey(summary.day);
      if (key !== null) map.set(key, summary);
    }
    return map;
  }

  function dayKey(value?: Day): number | null {
    return value?.date?.daysSinceEpoch ?? null;
  }

  function monthStartDay(daysSinceEpoch: number) {
    const date = new Date(daysSinceEpoch * MS_PER_DAY);
    return Math.floor(
      Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / MS_PER_DAY,
    );
  }

  function monthEndDay(daysSinceEpoch: number) {
    const date = new Date(daysSinceEpoch * MS_PER_DAY);
    return Math.floor(
      Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0) / MS_PER_DAY,
    );
  }

  function utcDayOfWeek(daysSinceEpoch: number) {
    return new Date(daysSinceEpoch * MS_PER_DAY).getUTCDay();
  }

  function dateLabel(daysSinceEpoch: number, compact = false) {
    return new Intl.DateTimeFormat("en-US", {
      timeZone: "UTC",
      weekday: compact ? undefined : "short",
      month: "short",
      day: "numeric",
    }).format(new Date(daysSinceEpoch * MS_PER_DAY));
  }

  function monthLabel(daysSinceEpoch: number) {
    return new Intl.DateTimeFormat("en-US", {
      timeZone: "UTC",
      month: "long",
      year: "numeric",
    }).format(new Date(daysSinceEpoch * MS_PER_DAY));
  }

  function getTz(): string {
    return (
      userStore.user?.timeZone?.timeZone ??
      Intl.DateTimeFormat().resolvedOptions().timeZone
    );
  }

  function tzTimeParts(timeZone: string): { h: number; m: number; s: number } {
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

  function ltToSeconds(lt: { hour: number; minute: number; second: number }) {
    return lt.hour * 3600 + lt.minute * 60 + lt.second;
  }

  function eventSeconds(event: Event): number | null {
    if (
      event.eventType.case !== "checkIn" &&
      event.eventType.case !== "checkOut"
    ) {
      return null;
    }
    return ltToSeconds(event.eventType.value);
  }

  function pad(n: number) {
    return String(n).padStart(2, "0");
  }

  function durationSeconds(duration?: Duration): number {
    return Number(duration?.seconds ?? 0n);
  }

  function summaryClockedSeconds(summary: DaySummary | null | undefined) {
    if (!summary) return 0;
    return dayKey(summary.day) === todayDays
      ? workSeconds
      : durationSeconds(summary.clockedWork);
  }

  function formatSeconds(secs: number) {
    const sign = secs < 0 ? "-" : "";
    const s = Math.max(0, Math.floor(Math.abs(secs)));
    return `${sign}${pad(Math.floor(s / 3600))}:${pad(Math.floor((s % 3600) / 60))}:${pad(s % 60)}`;
  }

  function formatHours(secs: number) {
    const sign = secs < 0 ? "-" : "";
    const abs = Math.abs(secs);
    const hours = Math.floor(abs / 3600);
    const minutes = Math.floor((abs % 3600) / 60);
    if (hours === 0) return `${sign}${minutes}m`;
    if (minutes === 0) return `${sign}${hours}h`;
    return `${sign}${hours}h ${minutes}m`;
  }

  function formatTime(seconds: number | null) {
    if (seconds === null) return "--:--";
    return `${pad(Math.floor(seconds / 3600))}:${pad(Math.floor((seconds % 3600) / 60))}`;
  }

  function computeWorkSeconds(d: Day | null): number {
    if (!d) return 0;

    let total = 0;
    let openCheckInSecs: number | null = null;

    for (const event of d.events) {
      if (event.eventType.case === "checkIn") {
        openCheckInSecs = ltToSeconds(event.eventType.value);
        continue;
      }

      if (event.eventType.case === "checkOut" && openCheckInSecs !== null) {
        total += Math.max(
          0,
          ltToSeconds(event.eventType.value) - openCheckInSecs,
        );
        openCheckInSecs = null;
      }
    }

    if (openCheckInSecs !== null) {
      const { h, m, s } = tzTimeParts(getTz());
      total += Math.max(0, h * 3600 + m * 60 + s - openCheckInSecs);
    }

    return total;
  }

  function buildSessions(d: Day | null, _liveTick: number): Session[] {
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
      const { h, m, s } = tzTimeParts(getTz());
      const now = h * 3600 + m * 60 + s;
      sessions.push({
        checkIn: openCheckIn,
        checkOut: null,
        duration: Math.max(0, now - openCheckIn),
      });
    }

    return sessions;
  }

  function tick() {
    const { h, m, s } = tzTimeParts(getTz());
    currentTimeDisplay = `${pad(h)}:${pad(m)}:${pad(s)}`;
    workSeconds = computeWorkSeconds(day);
  }

  async function loadDashboard() {
    loading = true;
    loadError = null;
    try {
      const window = dashboardWindow(tz);
      dashboard = await StoreService.getDashboard(
        window.rangeStart,
        window.rangeEnd,
        window.monthStart,
        window.monthEnd,
        window.today,
      );
      if (selectedDayKey === null || !summaryByDay.has(selectedDayKey)) {
        selectedDayKey = window.today.daysSinceEpoch;
      }
      tick();
    } catch (e) {
      dashboard = null;
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleCheckInOut() {
    submitting = true;
    try {
      const date = StoreService.currentDate(tz);
      const time = StoreService.currentTime(tz);
      if (isCheckedIn) {
        await StoreService.addCheckOut(date, time);
      } else {
        await StoreService.addCheckIn(date, time);
      }
      await loadDashboard();
      tick();
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function handleTakeDayOff() {
    if (takeDayOffDisabled) return;
    const date = day?.date ?? StoreService.currentDate(tz);
    flagUpdating = true;
    try {
      await StoreService.setFlag(date, DayFlag.DAY_OFF);
      selectedDayKey = date.daysSinceEpoch;
      await loadDashboard();
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      flagUpdating = false;
    }
  }

  async function toggleSelectedFlag(flag: DayFlag) {
    const selected = selectedSummary?.day;
    if (!selected?.date) return;
    flagUpdating = true;
    try {
      await StoreService.setFlag(selected.date, flag);
      await loadDashboard();
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      flagUpdating = false;
    }
  }

  function hasFlag(summary: DaySummary | null | undefined, flag: DayFlag) {
    return Boolean(summary?.day && (summary.day.flags & flag) === flag);
  }

  function serverBalanceSeconds(balance?: Balance): number {
    switch (balance?.balanceType.case) {
      case "overtime":
        return durationSeconds(balance.balanceType.value);
      case "underTime":
        return -durationSeconds(balance.balanceType.value);
      default:
        return 0;
    }
  }

  function isRegularRequiredDayValue(d: Day | null | undefined) {
    if (!d) return false;
    const nonRegularFlags =
      DayFlag.WEEKEND | DayFlag.DAY_OFF | DayFlag.VACATION | DayFlag.REMOTE;
    return (d.flags & nonRegularFlags) === 0;
  }

  function liveBalanceSeconds(summary: DaySummary | null | undefined) {
    if (!summary?.day || !isRegularRequiredDayValue(summary.day)) {
      return serverBalanceSeconds(summary?.balance);
    }
    return (
      summaryClockedSeconds(summary) -
      durationSeconds(summary.day.requiredWorkHours)
    );
  }

  function balanceLabel(summary: DaySummary | null | undefined) {
    if (!summary) return "No data";
    if (summary.skipped) return "Skipped";
    const delta = liveBalanceSeconds(summary);
    if (delta > 0) return `Overtime ${formatHours(delta)}`;
    if (delta < 0) return `Undertime ${formatHours(delta)}`;
    return "Exact";
  }

  function dayKindLabel(summary: DaySummary | null | undefined) {
    if (!summary?.day) return "No day";
    const labels = [];
    if (hasFlag(summary, DayFlag.WEEKEND)) labels.push("Weekend");
    if (hasFlag(summary, DayFlag.REMOTE)) labels.push("Remote");
    if (hasFlag(summary, DayFlag.DAY_OFF)) labels.push("Day off");
    if (hasFlag(summary, DayFlag.VACATION)) labels.push("Vacation");
    return labels.length > 0 ? labels.join(", ") : "Regular day";
  }

  function buildCalendarCells(items: DaySummary[]): CalendarCell[] {
    if (items.length === 0) return [];
    const cells: CalendarCell[] = [];
    const first = dayKey(items[0].day);
    if (first !== null) {
      for (let i = 0; i < utcDayOfWeek(first); i += 1) cells.push(null);
    }
    cells.push(...items);
    return cells;
  }

  function themeMix(token: string, amount: number, base = "var(--card)") {
    return `color-mix(in oklch, var(${token}) ${amount.toFixed(1)}%, ${base})`;
  }

  function activityCell(color: string) {
    return [
      `--activity-cell-bg: ${color}`,
      "background-color: var(--activity-cell-bg)",
      "border-color: color-mix(in oklch, var(--activity-cell-bg) 45%, var(--border))",
    ].join("; ");
  }

  function calendarCellStyle(summary: DaySummary | null) {
    if (!summary?.day) return "background-color: transparent;";
    if (hasFlag(summary, DayFlag.DAY_OFF)) {
      return activityCell(themeMix("--muted-foreground", 28));
    }
    if (hasFlag(summary, DayFlag.VACATION)) {
      return activityCell(themeMix("--primary", 48));
    }
    if (hasFlag(summary, DayFlag.REMOTE)) {
      return activityCell(themeMix("--chart-3", 58));
    }
    if (hasFlag(summary, DayFlag.WEEKEND)) {
      return summary.fullDayWorked
        ? activityCell(themeMix("--chart-5", 68))
        : activityCell(themeMix("--muted", 78));
    }
    if (summary.skipped) {
      return activityCell(themeMix("--destructive", 66));
    }

    const delta = liveBalanceSeconds(summary);
    const required = Math.max(1, durationSeconds(summary.day.requiredWorkHours));
    if (delta < 0) {
      const amount = Math.min(1, Math.abs(delta) / required);
      return activityCell(themeMix("--destructive", 30 + amount * 36));
    }

    const amount = Math.min(1, delta / required);
    return activityCell(themeMix("--chart-1", 26 + amount * 40));
  }

  function calendarDots(summary: DaySummary | null) {
    if (!summary?.day) return [];
    const dots: string[] = [];
    const clocked = summaryClockedSeconds(summary);
    if (hasFlag(summary, DayFlag.VACATION) && summary.fullDayWorked) {
      dots.push("var(--chart-5)");
    } else if (
      (hasFlag(summary, DayFlag.REMOTE) || hasFlag(summary, DayFlag.DAY_OFF)) &&
      clocked > 0
    ) {
      dots.push("var(--chart-1)");
    }
    return dots;
  }

  function balanceContribution(clocked: number, required: number) {
    const delta = clocked - required;
    return {
      overtime: Math.max(0, delta),
      undertime: Math.max(0, -delta),
    };
  }

  function buildStatItems(
    stats: MonthlyStats | null,
    today: DaySummary | null,
  ) {
    const closedToday = durationSeconds(today?.clockedWork);
    const liveToday = summaryClockedSeconds(today);
    const liveDelta = Math.max(0, liveToday - closedToday);
    let totalClocked = durationSeconds(stats?.totalClockedWork) + liveDelta;
    let overtime = durationSeconds(stats?.overtime);
    let undertime = durationSeconds(stats?.undertime);

    if (today?.day && isRegularRequiredDayValue(today.day)) {
      const required = durationSeconds(today.day.requiredWorkHours);
      const closed = balanceContribution(closedToday, required);
      const live = balanceContribution(liveToday, required);
      overtime += live.overtime - closed.overtime;
      undertime += live.undertime - closed.undertime;
    }

    return [
      {
        label: "Clocked",
        value: formatHours(totalClocked),
      },
      { label: "Overtime", value: formatHours(overtime) },
      {
        label: "Undertime",
        value: formatHours(undertime),
      },
      { label: "Worked days", value: String(stats?.workedDays ?? 0) },
      { label: "Remote", value: String(stats?.remoteDays ?? 0) },
      { label: "Day off", value: String(stats?.dayOffs ?? 0) },
      { label: "Vacation", value: String(stats?.vacationDays ?? 0) },
      { label: "Skipped", value: String(stats?.skippedDays ?? 0) },
      {
        label: "Weekend work",
        value: String(stats?.fullWeekendWorkDays ?? 0),
      },
      {
        label: "Vacation work",
        value: String(stats?.fullVacationWorkDays ?? 0),
      },
    ];
  }

  function buildChartModel(items: DaySummary[], currentDay: number) {
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

  function chartX(dayNumber: number, daysInMonth: number) {
    const left = 46;
    const width = 626;
    if (daysInMonth <= 1) return left;
    return left + ((dayNumber - 1) / (daysInMonth - 1)) * width;
  }

  function chartY(seconds: number) {
    const top = 16;
    const height = 184;
    return top + (1 - seconds / SECONDS_PER_DAY) * height;
  }

  function linePath(points: ChartPoint[], daysInMonth: number) {
    return points
      .map(
        (point, index) =>
          `${index === 0 ? "M" : "L"} ${chartX(point.day, daysInMonth).toFixed(2)} ${chartY(point.seconds).toFixed(2)}`,
      )
      .join(" ");
  }

  function selectedHasFlag(flag: DayFlag) {
    return hasFlag(selectedSummary, flag);
  }

  function selectedFlagDisabled(flag: DayFlag) {
    if (flag !== DayFlag.DAY_OFF || !selectedSummary?.day) return false;
    return (
      dayKey(selectedSummary.day) === todayDays &&
      selectedSummary.day.events.some(
        (event) => event.eventType.case === "checkIn",
      )
    );
  }

  onMount(() => {
    tick();
    const interval = setInterval(tick, 1000);
    return () => clearInterval(interval);
  });
</script>

<div class="py-4 flex flex-col gap-6">
  <div class="flex flex-col gap-1">
    <h2 class="text-2xl font-semibold">Dashboard</h2>
    <p class="text-muted-foreground text-sm">
      Today, rolling activity, and month-to-date work.
    </p>
  </div>

  {#if loadError}
    <div class="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-sm">
      {loadError}
    </div>
  {/if}

  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
    <Card.Root>
      <Card.Header>
        <Card.Title>Today</Card.Title>
        <Card.Description>{dateLabel(todayDays)}</Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-5 md:grid-cols-[minmax(0,1fr)_260px]">
        <div class="flex flex-col gap-5">
          <div class="grid gap-4 sm:grid-cols-2">
            <div class="flex flex-col gap-1">
              <span class="text-muted-foreground text-xs uppercase">Current time</span>
              <span class="font-mono text-4xl font-semibold tabular-nums">
                {currentTimeDisplay}
              </span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-foreground text-xs uppercase">Work time</span>
              <span
                class="font-mono text-4xl font-semibold tabular-nums transition-colors {isCheckedIn
                  ? 'text-primary'
                  : ''}"
              >
                {formatSeconds(workSeconds)}
              </span>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <Progress value={progressPercent} max={100} class="h-1.5" />
            <div class="flex justify-between text-xs tabular-nums text-muted-foreground">
              <span>{formatSeconds(workSeconds)}</span>
              <span>{formatSeconds(requiredSeconds)}</span>
            </div>
          </div>

          <div class="flex flex-col gap-2 sm:flex-row">
            <Button
              onclick={handleCheckInOut}
              disabled={submitting || loading}
              variant={isCheckedIn ? "outline" : "default"}
              class="w-full sm:w-40"
            >
              {#if isCheckedIn}
                <LogOutIcon />
              {:else}
                <LogInIcon />
              {/if}
              {#if submitting}
                ...
              {:else if isCheckedIn}
                Check Out
              {:else}
                Check In
              {/if}
            </Button>
            <Button
              onclick={handleTakeDayOff}
              disabled={takeDayOffDisabled}
              variant="secondary"
              class="w-full sm:w-40"
              title={hasCheckInToday
                ? "Unavailable after check-in"
                : takeDayOffLabel}
            >
              <CalendarXIcon />
              {takeDayOffLabel}
            </Button>
          </div>
        </div>

        <div class="flex min-h-40 flex-col gap-3">
          <div class="flex items-center justify-between gap-3">
            <span class="text-sm font-medium">Today history</span>
            <span class="text-muted-foreground text-xs">{todaySessions.length} sessions</span>
          </div>
          <Separator />
          {#if loading}
            <div class="text-muted-foreground text-sm">Loading...</div>
          {:else if todaySessions.length === 0}
            <div class="text-muted-foreground text-sm">No check-ins yet.</div>
          {:else}
            <div class="flex flex-col gap-2">
              {#each todaySessions as session, index}
                <div class="grid grid-cols-[24px_1fr_auto] items-center gap-3 text-sm">
                  <span class="text-muted-foreground tabular-nums">{index + 1}</span>
                  <span class="font-mono tabular-nums">
                    {formatTime(session.checkIn)} - {formatTime(session.checkOut)}
                  </span>
                  <span class="text-muted-foreground font-mono tabular-nums">
                    {session.duration === null ? "--:--" : formatSeconds(session.duration)}
                  </span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Selected Day</Card.Title>
        <Card.Description>
          {selectedSummary?.day?.date
            ? dateLabel(selectedSummary.day.date.daysSinceEpoch)
            : "Pick a day"}
        </Card.Description>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4">
        <div class="grid grid-cols-2 gap-3 text-sm">
          <div>
            <div class="text-muted-foreground text-xs uppercase">Status</div>
            <div class="font-medium">{dayKindLabel(selectedSummary)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Balance</div>
            <div class="font-medium">{balanceLabel(selectedSummary)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Clocked</div>
            <div class="font-mono">{formatSeconds(summaryClockedSeconds(selectedSummary))}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Required</div>
            <div class="font-mono">
              {formatSeconds(durationSeconds(selectedSummary?.day?.requiredWorkHours))}
            </div>
          </div>
        </div>

        <Separator />

        <div class="flex flex-col gap-3">
          {#each flagControls as control}
            <div class="flex items-center justify-between gap-3">
              <span class="text-sm">{control.label}</span>
              <Switch.Root
                checked={selectedHasFlag(control.flag)}
                disabled={
                  flagUpdating || !selectedSummary || selectedFlagDisabled(control.flag)
                }
                onclick={() => toggleSelectedFlag(control.flag)}
              />
            </div>
          {/each}
        </div>
      </Card.Content>
    </Card.Root>
  </div>

  <Card.Root>
    <Card.Header>
      <Card.Title>Activity</Card.Title>
      <Card.Description>Rolling year ending today</Card.Description>
    </Card.Header>
    <Card.Content class="overflow-x-auto">
      {#if loading}
        <div class="text-muted-foreground text-sm">Loading...</div>
      {:else}
        <div class="grid grid-cols-[32px_1fr] gap-2">
          <div class="grid grid-rows-7 gap-[3px] text-[10px] text-muted-foreground">
            <span></span>
            <span>Mon</span>
            <span></span>
            <span>Wed</span>
            <span></span>
            <span>Fri</span>
            <span></span>
          </div>
          <div
            class="grid grid-flow-col grid-rows-7 auto-cols-[13px] gap-[3px]"
            aria-label="Rolling year activity calendar"
          >
            {#each calendarCells as summary}
              {#if summary?.day}
                <button
                  type="button"
                  class="relative size-[13px] rounded-[3px] border border-border/40 outline-none transition-transform hover:scale-125 focus-visible:ring-2 focus-visible:ring-ring {selectedDayKey ===
                  summary.day.date?.daysSinceEpoch
                    ? 'ring-2 ring-ring'
                    : ''}"
                  style={calendarCellStyle(summary)}
                  title={`${dateLabel(summary.day.date?.daysSinceEpoch ?? 0)} - ${dayKindLabel(summary)} - ${balanceLabel(summary)}`}
                  onclick={() => (selectedDayKey = summary.day?.date?.daysSinceEpoch ?? null)}
                >
                  <span class="sr-only">
                    {dateLabel(summary.day.date?.daysSinceEpoch ?? 0)}
                  </span>
                  {#each calendarDots(summary) as color}
                    <span
                      class="absolute bottom-[1px] right-[1px] size-[4px] rounded-full border border-background"
                      style={`background-color: ${color};`}
                    ></span>
                  {/each}
                </button>
              {:else}
                <span class="size-[13px]"></span>
              {/if}
            {/each}
          </div>
        </div>
      {/if}
    </Card.Content>
  </Card.Root>

  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(420px,1fr)]">
    <Card.Root>
      <Card.Header>
        <Card.Title>{monthLabel(todayDays)}</Card.Title>
        <Card.Description>Month-to-date stats</Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
          {#each statItems as item}
            <div class="rounded-md border bg-muted/30 px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase">{item.label}</div>
              <div class="mt-1 font-mono text-lg font-semibold tabular-nums">
                {item.value}
              </div>
            </div>
          {/each}
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Month Rhythm</Card.Title>
        <Card.Description>First check-in, last checkout, and intermediate taps</Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="w-full overflow-x-auto">
          <svg
            viewBox="0 0 720 240"
            class="h-[260px] min-w-[620px] w-full text-xs"
            role="img"
            aria-label="Month check-in and checkout chart"
          >
            <g class="text-muted-foreground">
              {#each [0, 6, 12, 18, 24] as hour}
                <line
                  x1="46"
                  x2="672"
                  y1={chartY(hour * 3600)}
                  y2={chartY(hour * 3600)}
                  stroke="currentColor"
                  stroke-opacity="0.16"
                />
                <text x="8" y={chartY(hour * 3600) + 4} fill="currentColor">
                  {pad(hour)}:00
                </text>
              {/each}
              {#each [1, 8, 15, 22, chartModel.daysInMonth] as dayNumber}
                <line
                  x1={chartX(dayNumber, chartModel.daysInMonth)}
                  x2={chartX(dayNumber, chartModel.daysInMonth)}
                  y1="16"
                  y2="200"
                  stroke="currentColor"
                  stroke-opacity="0.1"
                />
                <text
                  x={chartX(dayNumber, chartModel.daysInMonth)}
                  y="224"
                  text-anchor="middle"
                  fill="currentColor"
                >
                  {dayNumber}
                </text>
              {/each}
            </g>

            <path
              d={linePath(chartModel.firstLine, chartModel.daysInMonth)}
              fill="none"
              stroke="var(--chart-1)"
              stroke-width="2"
            />
            <path
              d={linePath(chartModel.lastLine, chartModel.daysInMonth)}
              fill="none"
              stroke="var(--destructive)"
              stroke-width="2"
            />

            {#each chartModel.points as point}
              <circle
                cx={chartX(point.day, chartModel.daysInMonth)}
                cy={chartY(point.seconds)}
                r={point.kind === "checkIn" ? 3 : 3.5}
                fill={point.kind === "checkIn"
                  ? "var(--chart-1)"
                  : "var(--destructive)"}
              >
                <title>{point.label}</title>
              </circle>
            {/each}
          </svg>
        </div>
        <div class="mt-3 flex flex-wrap gap-4 text-xs text-muted-foreground">
          <span class="inline-flex items-center gap-2">
            <span
              class="size-2 rounded-full"
              style="background-color: var(--chart-1);"
            ></span>
            First check-in
          </span>
          <span class="inline-flex items-center gap-2">
            <span
              class="size-2 rounded-full"
              style="background-color: var(--destructive);"
            ></span>
            Last checkout and checkouts
          </span>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</div>
