<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { WorkHoursInput } from "$lib/blocks/components";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import * as ScrollArea from "$lib/components/ui/scroll-area/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import * as Switch from "$lib/components/ui/switch/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import {
    addDays,
    balanceLabel,
    buildSessions,
    buildSummaryMap,
    currentTimeValue,
    dateLabel,
    dayKey,
    durationSeconds,
    firstCheckIn,
    flagLabels,
    formatHours,
    formatSeconds,
    formatTime,
    fullDateLabel,
    hasFlag,
    lastCheckOut,
    mondayFirstDayOfWeek,
    monthEndDay,
    monthStartDay,
    nextManualEventType,
    parseManualTime,
    protoDate,
    rangeLabel,
    requiredDaySeconds,
    summaryClockedSeconds,
    todayDate,
    workTargetSeconds,
    type ManualEventType,
  } from "$lib/dashboard";
  import { StoreService } from "$lib/services";
  import { userStore } from "$lib/stores";
  import { Duration } from "@bufbuild/protobuf";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import { DayFlag } from "@taptime/proto/taptime/day_pb.js";
  import type { DashboardResponse, DaySummary } from "@taptime/proto/taptime/services/store_pb.js";

  type HistoryPreset = "week" | "month" | "last30" | "last90";
  type StatusFilter =
    | "all"
    | "regular"
    | "remote"
    | "dayOff"
    | "vacation"
    | "weekend"
    | "skipped"
    | "exceptions";

  const flagControls = [
    { flag: DayFlag.WEEKEND, label: "Weekend" },
    { flag: DayFlag.REMOTE, label: "Remote" },
    { flag: DayFlag.DAY_OFF, label: "Day off" },
    { flag: DayFlag.VACATION, label: "Vacation" },
  ];

  const tz = $derived(
    userStore.user?.timeZone?.timeZone ??
      Intl.DateTimeFormat().resolvedOptions().timeZone,
  );

  let dashboard = $state<DashboardResponse | null>(null);
  let loadError = $state<string | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let preset = $state<HistoryPreset>("last30");
  let statusFilter = $state<StatusFilter>("all");
  let query = $state("");
  let selectedDayKey = $state<number | null>(null);
  let loadedKey = $state("");
  let flagUpdating = $state(false);
  let overrideSaving = $state(false);
  let manualOpen = $state(false);
  let manualSubmitting = $state(false);
  let manualEventType = $state<ManualEventType>("checkIn");
  let manualTime = $state("");
  let overrideHours = $state(0);
  let overrideMinutes = $state(0);
  let overrideLoadedKey = $state("");

  const todayDays = $derived(StoreService.currentDate(tz).daysSinceEpoch);
  const range = $derived(historyRange(preset, todayDays));
  const summaries = $derived(dashboard?.days ?? []);
  const summaryByDay = $derived(buildSummaryMap(summaries));
  const selectedSummary = $derived(
    selectedDayKey === null ? null : summaryByDay.get(selectedDayKey) ?? null,
  );
  const rows = $derived([...summaries].sort((a, b) => (dayKey(b.day) ?? 0) - (dayKey(a.day) ?? 0)));
  const filteredRows = $derived(rows.filter((summary) => matchesFilters(summary)));
  const selectedSessions = $derived(buildSessions(selectedSummary?.day));
  const validManualEventType = $derived(nextManualEventType(selectedSummary?.day?.events ?? []));
  const parsedManualTime = $derived(parseManualTime(manualTime));
  const manualEventTypeLabel = $derived(manualEventType === "checkIn" ? "Check In" : "Check Out");
  const manualSubmitDisabled = $derived(
    manualSubmitting ||
      loading ||
      refreshing ||
      !selectedSummary?.day?.date ||
      !parsedManualTime ||
      manualEventType !== validManualEventType,
  );
  const selectedWorkTargetSeconds = $derived(workTargetSeconds(selectedSummary));
  const selectedLunchSeconds = $derived(
    durationSeconds(selectedSummary?.day?.lunchBreakDuration),
  );
  const overrideSeconds = $derived(overrideHours * 3600 + overrideMinutes * 60);
  const overrideDirty = $derived(
    Boolean(selectedSummary?.day && overrideSeconds !== selectedWorkTargetSeconds),
  );
  const overrideSaveDisabled = $derived(
    overrideSaving ||
      flagUpdating ||
      manualSubmitting ||
      !selectedSummary?.day?.date ||
      overrideSeconds <= 0 ||
      !overrideDirty,
  );
  const overrideClearDisabled = $derived(
    overrideSaving ||
      flagUpdating ||
      manualSubmitting ||
      !selectedSummary?.day?.date ||
      !selectedSummary.requiredWorkHoursOverridden,
  );

  $effect(() => {
    const key = `${tz}:${preset}:${range.start}:${range.end}`;
    if (key !== loadedKey) {
      loadedKey = key;
      void loadHistory();
    }
  });

  $effect(() => {
    if (manualEventType !== validManualEventType) {
      manualEventType = validManualEventType;
    }
  });

  $effect(() => {
    const key = [
      selectedSummary?.day?.date?.daysSinceEpoch ?? "none",
      selectedWorkTargetSeconds,
      selectedSummary?.requiredWorkHoursOverridden ? "override" : "default",
    ].join(":");
    if (key !== overrideLoadedKey) {
      overrideLoadedKey = key;
      overrideHours = Math.floor(selectedWorkTargetSeconds / 3600);
      overrideMinutes = Math.floor((selectedWorkTargetSeconds % 3600) / 60);
    }
  });

  function historyRange(value: HistoryPreset, today: number) {
    switch (value) {
      case "week":
        return { start: addDays(today, -mondayFirstDayOfWeek(today)), end: today };
      case "month":
        return { start: monthStartDay(today), end: today };
      case "last90":
        return { start: addDays(today, -89), end: today };
      case "last30":
      default:
        return { start: addDays(today, -29), end: today };
    }
  }

  async function loadHistory(background = dashboard !== null) {
    if (background) {
      refreshing = true;
    } else {
      loading = true;
    }
    loadError = null;
    try {
      const today = todayDate(tz);
      const next = await StoreService.getDashboardRange({
        rangeStart: protoDate(range.start),
        rangeEnd: protoDate(range.end),
        monthStart: protoDate(monthStartDay(range.end)),
        monthEnd: protoDate(monthEndDay(range.end)),
        today,
      });
      const nextMap = buildSummaryMap(next.days);
      dashboard = next;
      if (selectedDayKey === null || !nextMap.has(selectedDayKey)) {
        selectedDayKey = nextMap.has(today.daysSinceEpoch)
          ? today.daysSinceEpoch
          : range.end;
      }
    } catch (e) {
      if (!background) dashboard = null;
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      if (background) {
        refreshing = false;
      } else {
        loading = false;
      }
    }
  }

  function matchesFilters(summary: DaySummary) {
    const text = query.trim().toLowerCase();
    if (text.length > 0) {
      const key = dayKey(summary.day) ?? 0;
      const haystack = [
        fullDateLabel(key),
        dateLabel(key),
        flagLabels(summary).join(" "),
        balanceLabel(summary),
      ]
        .join(" ")
        .toLowerCase();
      if (!haystack.includes(text)) return false;
    }

    switch (statusFilter) {
      case "regular":
        return flagLabels(summary).length === 0;
      case "remote":
        return hasFlag(summary, DayFlag.REMOTE);
      case "dayOff":
        return hasFlag(summary, DayFlag.DAY_OFF);
      case "vacation":
        return hasFlag(summary, DayFlag.VACATION);
      case "weekend":
        return hasFlag(summary, DayFlag.WEEKEND);
      case "skipped":
        return summary.skipped;
      case "exceptions":
        return summary.skipped || balanceLabel(summary).startsWith("Overtime") || balanceLabel(summary).startsWith("Undertime");
      case "all":
      default:
        return true;
    }
  }

  function openManualEvent() {
    manualEventType = validManualEventType;
    manualTime = currentTimeValue(tz);
    manualOpen = true;
  }

  function manualEventTypeAllowed(type: ManualEventType) {
    return type === validManualEventType;
  }

  async function submitManualEvent(event: SubmitEvent) {
    event.preventDefault();
    const date = selectedSummary?.day?.date;
    if (manualSubmitDisabled || !date || !parsedManualTime) return;
    manualSubmitting = true;
    try {
      if (manualEventType === "checkIn") {
        await StoreService.addCheckIn(date, parsedManualTime);
      } else {
        await StoreService.addCheckOut(date, parsedManualTime);
      }
      selectedDayKey = date.daysSinceEpoch;
      manualOpen = false;
      await loadHistory(true);
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      manualSubmitting = false;
    }
  }

  async function toggleSelectedFlag(flag: DayFlag) {
    const date = selectedSummary?.day?.date;
    if (!date) return;
    flagUpdating = true;
    try {
      await StoreService.setFlag(date, flag);
      selectedDayKey = date.daysSinceEpoch;
      await loadHistory(true);
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      flagUpdating = false;
    }
  }

  async function saveRequiredOverride() {
    const date = selectedSummary?.day?.date;
    if (overrideSaveDisabled || !date) return;
    overrideSaving = true;
    try {
      await StoreService.setRequiredWorkHoursOverride(
        date,
        new Duration({ seconds: BigInt(overrideSeconds) }),
      );
      selectedDayKey = date.daysSinceEpoch;
      await loadHistory(true);
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      overrideSaving = false;
    }
  }

  async function clearRequiredOverride() {
    const date = selectedSummary?.day?.date;
    if (overrideClearDisabled || !date) return;
    overrideSaving = true;
    try {
      await StoreService.setRequiredWorkHoursOverride(date, null);
      selectedDayKey = date.daysSinceEpoch;
      await loadHistory(true);
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      overrideSaving = false;
    }
  }

  function selectedHasFlag(flag: DayFlag) {
    return hasFlag(selectedSummary, flag);
  }

  function rowStatus(summary: DaySummary) {
    const labels = flagLabels(summary);
    if (summary.skipped) labels.push("Skipped");
    if (summary.fullDayWorked && (hasFlag(summary, DayFlag.WEEKEND) || hasFlag(summary, DayFlag.VACATION))) {
      labels.push("Full work");
    }
    return labels.length > 0 ? labels : ["Regular"];
  }
</script>

<div class="flex flex-col gap-6 py-4" aria-busy={loading || refreshing}>
  <div class="flex flex-col gap-1">
    <h2 class="text-2xl font-semibold">History</h2>
    <p class="text-muted-foreground text-sm">
      Audit days, correct missing taps, and review sessions.
    </p>
  </div>

  {#if loadError}
    <div class="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-sm">
      {loadError}
    </div>
  {/if}

  <Card.Root>
    <Card.Content class="flex flex-col gap-3 p-4 md:flex-row md:items-end">
      <div class="grid gap-2 md:w-48">
        <Label for="history-preset">Range</Label>
        <Select.Root
          type="single"
          name="history-preset"
          value={preset}
          onValueChange={(value) => {
            if (value === "week" || value === "month" || value === "last30" || value === "last90") preset = value;
          }}
        >
          <Select.Trigger id="history-preset" class="w-full">{rangeLabel(range.start, range.end)}</Select.Trigger>
          <Select.Content>
            <Select.Item value="week" label="This week">This week</Select.Item>
            <Select.Item value="month" label="This month">This month</Select.Item>
            <Select.Item value="last30" label="Last 30 days">Last 30 days</Select.Item>
            <Select.Item value="last90" label="Last 90 days">Last 90 days</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>

      <div class="grid gap-2 md:w-44">
        <Label for="history-status">Status</Label>
        <Select.Root
          type="single"
          name="history-status"
          value={statusFilter}
          onValueChange={(value) => {
            if (
              value === "all" ||
              value === "regular" ||
              value === "remote" ||
              value === "dayOff" ||
              value === "vacation" ||
              value === "weekend" ||
              value === "skipped" ||
              value === "exceptions"
            ) statusFilter = value;
          }}
        >
          <Select.Trigger id="history-status" class="w-full">Filter</Select.Trigger>
          <Select.Content>
            <Select.Item value="all" label="All days">All days</Select.Item>
            <Select.Item value="regular" label="Regular">Regular</Select.Item>
            <Select.Item value="remote" label="Remote">Remote</Select.Item>
            <Select.Item value="dayOff" label="Day off">Day off</Select.Item>
            <Select.Item value="vacation" label="Vacation">Vacation</Select.Item>
            <Select.Item value="weekend" label="Weekend">Weekend</Select.Item>
            <Select.Item value="skipped" label="Skipped">Skipped</Select.Item>
            <Select.Item value="exceptions" label="Exceptions">Exceptions</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>

      <div class="grid flex-1 gap-2">
        <Label for="history-search">Search</Label>
        <Input
          id="history-search"
          bind:value={query}
          placeholder="Date, flag, or balance"
        />
      </div>

      <Button
        variant="secondary"
        onclick={() => loadHistory(true)}
        disabled={loading || refreshing}
        class="md:w-28"
      >
        <RefreshCwIcon />
        Refresh
      </Button>
    </Card.Content>
  </Card.Root>

  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
    <Card.Root>
      <Card.Header>
        <Card.Title>Day Ledger</Card.Title>
        <Card.Description>{filteredRows.length} of {rows.length} days</Card.Description>
      </Card.Header>
      <Card.Content>
        <ScrollArea.Root class="h-[560px]">
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.Head>Date</Table.Head>
                <Table.Head>Status</Table.Head>
                <Table.Head>First in</Table.Head>
                <Table.Head>Last out</Table.Head>
                <Table.Head>Clocked</Table.Head>
                <Table.Head>Required</Table.Head>
                <Table.Head>Balance</Table.Head>
                <Table.Head class="text-right">Sessions</Table.Head>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {#if loading}
                <Table.Row>
                  <Table.Cell colspan={8} class="text-muted-foreground py-8 text-center">
                    Loading...
                  </Table.Cell>
                </Table.Row>
              {:else if filteredRows.length === 0}
                <Table.Row>
                  <Table.Cell colspan={8} class="text-muted-foreground py-8 text-center">
                    No matching days.
                  </Table.Cell>
                </Table.Row>
              {:else}
                {#each filteredRows as summary}
                  <Table.Row
                    class="cursor-pointer hover:bg-muted/40 {selectedDayKey === dayKey(summary.day) ? 'bg-muted/60' : ''}"
                    onclick={() => (selectedDayKey = dayKey(summary.day))}
                  >
                    <Table.Cell class="font-medium">{dateLabel(dayKey(summary.day) ?? 0)}</Table.Cell>
                    <Table.Cell>
                      <div class="flex flex-wrap gap-1">
                        {#each rowStatus(summary) as label}
                          <span class="rounded-sm bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                            {label}
                          </span>
                        {/each}
                      </div>
                    </Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatTime(firstCheckIn(summary))}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatTime(lastCheckOut(summary))}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatHours(summaryClockedSeconds(summary))}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatHours(requiredDaySeconds(summary.day))}</Table.Cell>
                    <Table.Cell>{balanceLabel(summary)}</Table.Cell>
                    <Table.Cell class="text-right font-mono tabular-nums">{buildSessions(summary.day).length}</Table.Cell>
                  </Table.Row>
                {/each}
              {/if}
            </Table.Body>
          </Table.Root>
        </ScrollArea.Root>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Selected Day</Card.Title>
        <Card.Description>
          {selectedSummary?.day?.date
            ? fullDateLabel(selectedSummary.day.date.daysSinceEpoch)
            : "Pick a day"}
        </Card.Description>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4">
        {#if selectedSummary?.day}
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <div class="text-muted-foreground text-xs uppercase">Clocked</div>
              <div class="font-mono">{formatSeconds(summaryClockedSeconds(selectedSummary))}</div>
            </div>
            <div>
              <div class="text-muted-foreground text-xs uppercase">Balance</div>
              <div class="font-medium">{balanceLabel(selectedSummary)}</div>
            </div>
            <div>
              <div class="text-muted-foreground text-xs uppercase">First in</div>
              <div class="font-mono">{formatTime(firstCheckIn(selectedSummary))}</div>
            </div>
            <div>
              <div class="text-muted-foreground text-xs uppercase">Last out</div>
              <div class="font-mono">{formatTime(lastCheckOut(selectedSummary))}</div>
            </div>
          </div>

          <Separator />

          <div class="flex flex-col gap-3">
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-medium">Required Work Override</div>
                <div class="text-muted-foreground text-xs">
                  {formatHours(overrideSeconds)} work + {formatHours(selectedLunchSeconds)} lunch = {formatHours(overrideSeconds + selectedLunchSeconds)} target
                </div>
              </div>
              {#if selectedSummary.requiredWorkHoursOverridden}
                <span class="rounded-sm bg-primary/10 px-1.5 py-0.5 text-xs text-primary">
                  Override
                </span>
              {/if}
            </div>
            <WorkHoursInput bind:hours={overrideHours} bind:minutes={overrideMinutes} />
            <div class="flex justify-end gap-2">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onclick={clearRequiredOverride}
                disabled={overrideClearDisabled}
              >
                Clear
              </Button>
              <Button
                type="button"
                size="sm"
                onclick={saveRequiredOverride}
                disabled={overrideSaveDisabled}
              >
                {overrideSaving ? "Saving..." : "Save Target"}
              </Button>
            </div>
          </div>

          <Separator />

          <div class="flex flex-col gap-3">
            {#each flagControls as control}
              <div class="flex items-center justify-between gap-3">
                <span class="text-sm">{control.label}</span>
                <Switch.Root
                  checked={selectedHasFlag(control.flag)}
                  disabled={flagUpdating || manualSubmitting}
                  onclick={() => toggleSelectedFlag(control.flag)}
                />
              </div>
            {/each}
          </div>

          <Separator />

          <div class="flex items-center justify-between gap-3">
            <div>
              <div class="text-sm font-medium">Sessions</div>
              <div class="text-muted-foreground text-xs">{selectedSessions.length} sessions</div>
            </div>
            <Popover.Root bind:open={manualOpen}>
              <Popover.Trigger>
                {#snippet child({ props })}
                  <Button
                    {...props}
                    variant="secondary"
                    size="sm"
                    onclick={openManualEvent}
                    disabled={manualSubmitting || flagUpdating}
                  >
                    <PlusIcon />
                    Add Event
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content align="end" sideOffset={8} class="w-80">
                <form class="flex flex-col gap-4" onsubmit={submitManualEvent}>
                  <div class="flex flex-col gap-1">
                    <div class="font-medium">Add Event</div>
                    <div class="text-muted-foreground text-xs">
                      {dateLabel(selectedSummary.day.date?.daysSinceEpoch ?? 0)}
                    </div>
                  </div>

                  <div class="grid gap-2">
                    <Label for="history-manual-type">Event</Label>
                    <Select.Root
                      type="single"
                      name="history-manual-type"
                      value={manualEventType}
                      onValueChange={(value) => {
                        if (value === "checkIn" || value === "checkOut") manualEventType = value;
                      }}
                    >
                      <Select.Trigger id="history-manual-type" class="w-full">{manualEventTypeLabel}</Select.Trigger>
                      <Select.Content>
                        <Select.Item value="checkIn" label="Check In" disabled={!manualEventTypeAllowed("checkIn")}>Check In</Select.Item>
                        <Select.Item value="checkOut" label="Check Out" disabled={!manualEventTypeAllowed("checkOut")}>Check Out</Select.Item>
                      </Select.Content>
                    </Select.Root>
                  </div>

                  <div class="grid gap-2">
                    <Label for="history-manual-time">Time</Label>
                    <Input
                      id="history-manual-time"
                      type="time"
                      step="60"
                      bind:value={manualTime}
                      aria-invalid={!parsedManualTime}
                    />
                  </div>

                  <div class="flex justify-end gap-2">
                    <Button type="button" variant="ghost" onclick={() => (manualOpen = false)}>Cancel</Button>
                    <Button type="submit" disabled={manualSubmitDisabled}>
                      {manualSubmitting ? "Adding..." : `Add ${manualEventTypeLabel}`}
                    </Button>
                  </div>
                </form>
              </Popover.Content>
            </Popover.Root>
          </div>

          <ScrollArea.Root class="h-56 pr-3">
            {#if selectedSessions.length === 0}
              <div class="text-muted-foreground text-sm">No sessions.</div>
            {:else}
              <div class="flex flex-col gap-2">
                {#each selectedSessions as session, index}
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
          </ScrollArea.Root>
        {:else}
          <div class="text-muted-foreground flex min-h-48 items-center justify-center text-sm">
            Select a day from the ledger.
          </div>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>
</div>
