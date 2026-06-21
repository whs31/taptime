<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as ScrollArea from "$lib/components/ui/scroll-area/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import {
    dateLabel,
    dayKey,
    exceptionRows,
    formatHours,
    formatSeconds,
    fullDateLabel,
    monthEndDay,
    monthLabel,
    monthlyBuckets,
    monthStartDay,
    previousMonthStartDay,
    protoDate,
    quarterStartDay,
    rangeLabel,
    summarizeRange,
    todayDate,
    yearStartDay,
  } from "$lib/dashboard";
  import { StoreService } from "$lib/services";
  import { userStore } from "$lib/stores";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import type { DashboardResponse, DaySummary } from "@taptime/proto/taptime/services/store_pb.js";

  type ReportPreset = "month" | "previousMonth" | "quarter" | "year";

  type WorkBucket = {
    key: string;
    label: string;
    value: number;
  };

  const tz = $derived(
    userStore.user?.timeZone?.timeZone ??
      Intl.DateTimeFormat().resolvedOptions().timeZone,
  );

  let dashboard = $state<DashboardResponse | null>(null);
  let loadError = $state<string | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let preset = $state<ReportPreset>("year");
  let loadedKey = $state("");

  const todayDays = $derived(StoreService.currentDate(tz).daysSinceEpoch);
  const range = $derived(reportRange(preset, todayDays));
  const summaries = $derived(dashboard?.days ?? []);
  const totals = $derived(summarizeRange(summaries));
  const months = $derived(monthlyBuckets(summaries));
  const exceptions = $derived(exceptionRows(summaries));
  const workBuckets = $derived(buildWorkBuckets(summaries, range.start, range.end));
  const maxMonthlyBalance = $derived(Math.max(3_600, ...months.map((month) => Math.abs(month.overtime - month.undertime))));
  const maxWorked = $derived(Math.max(3_600, ...workBuckets.map((bucket) => bucket.value)));
  const statusBars = $derived([
    { label: "Remote", value: totals.remoteDays, color: "var(--chart-3)" },
    { label: "Day off", value: totals.dayOffs, color: "var(--muted-foreground)" },
    { label: "Vacation", value: totals.vacationDays, color: "var(--primary)" },
    { label: "Skipped", value: totals.skippedDays, color: "var(--destructive)" },
    { label: "Weekend work", value: totals.fullWeekendWorkDays, color: "var(--chart-5)" },
    { label: "Vacation work", value: totals.fullVacationWorkDays, color: "var(--chart-4)" },
  ]);
  const maxStatus = $derived(Math.max(1, ...statusBars.map((bar) => bar.value)));
  const statItems = $derived([
    { label: "Clocked", value: formatHours(totals.totalClocked) },
    { label: "Overtime", value: formatHours(totals.overtime) },
    { label: "Undertime", value: formatHours(totals.undertime) },
    { label: "Worked days", value: String(totals.workedDays) },
    { label: "Remote", value: String(totals.remoteDays) },
    { label: "Day off", value: String(totals.dayOffs) },
    { label: "Vacation", value: String(totals.vacationDays) },
    { label: "Skipped", value: String(totals.skippedDays) },
    { label: "Weekend work", value: String(totals.fullWeekendWorkDays) },
    { label: "Vacation work", value: String(totals.fullVacationWorkDays) },
  ]);

  $effect(() => {
    const key = `${tz}:${preset}:${range.start}:${range.end}`;
    if (key !== loadedKey) {
      loadedKey = key;
      void loadReports();
    }
  });

  function reportRange(value: ReportPreset, today: number) {
    switch (value) {
      case "month":
        return { start: monthStartDay(today), end: today, label: "Current month" };
      case "previousMonth": {
        const start = previousMonthStartDay(today);
        return { start, end: monthEndDay(start), label: "Previous month" };
      }
      case "quarter":
        return { start: quarterStartDay(today), end: today, label: "Current quarter" };
      case "year":
      default:
        return { start: yearStartDay(today), end: today, label: "Current year" };
    }
  }

  async function loadReports(background = dashboard !== null) {
    if (background) {
      refreshing = true;
    } else {
      loading = true;
    }
    loadError = null;
    try {
      const today = todayDate(tz);
      dashboard = await StoreService.getDashboardRange({
        rangeStart: protoDate(range.start),
        rangeEnd: protoDate(range.end),
        monthStart: protoDate(range.start),
        monthEnd: protoDate(range.end),
        today,
      });
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

  function buildWorkBuckets(items: DaySummary[], start: number, end: number): WorkBucket[] {
    if (end - start <= 45) {
      return items
        .map((summary) => {
          const key = dayKey(summary.day);
          return key === null
            ? null
            : {
                key: String(key),
                label: dateLabel(key, true),
                value: Number(summary.clockedWork?.seconds ?? 0n),
              };
        })
        .filter((bucket): bucket is WorkBucket => bucket !== null);
    }

    const buckets: WorkBucket[] = [];
    for (let cursor = start; cursor <= end; cursor += 7) {
      const bucketEnd = Math.min(end, cursor + 6);
      const value = items.reduce((total, summary) => {
        const key = dayKey(summary.day);
        if (key === null || key < cursor || key > bucketEnd) return total;
        return total + Number(summary.clockedWork?.seconds ?? 0n);
      }, 0);
      buckets.push({
        key: String(cursor),
        label: dateLabel(cursor, true),
        value,
      });
    }
    return buckets;
  }

  function monthlyNet(month: { overtime: number; undertime: number }) {
    return month.overtime - month.undertime;
  }

  function trendX(index: number, count: number) {
    const left = 44;
    const width = 552;
    if (count <= 1) return left + width / 2;
    return left + (index / (count - 1)) * width;
  }

  function trendY(value: number) {
    const center = 96;
    const height = 70;
    return center - (value / maxMonthlyBalance) * height;
  }

  function trendPath() {
    return months
      .map((month, index) => {
        const command = index === 0 ? "M" : "L";
        return `${command} ${trendX(index, months.length).toFixed(2)} ${trendY(monthlyNet(month)).toFixed(2)}`;
      })
      .join(" ");
  }

  function barHeight(value: number, max: number, height = 140) {
    if (value <= 0) return 2;
    return Math.max(2, (value / max) * height);
  }

  function signedHours(value: number) {
    if (value > 0) return `+${formatHours(value)}`;
    if (value < 0) return `-${formatHours(Math.abs(value))}`;
    return "0m";
  }
</script>

<div class="flex flex-col gap-6 py-4" aria-busy={loading || refreshing}>
  <div class="flex flex-col gap-1">
    <h2 class="text-2xl font-semibold">Reports</h2>
    <p class="text-muted-foreground text-sm">
      Trends, totals, and exceptions for the selected period.
    </p>
  </div>

  {#if loadError}
    <div class="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-sm">
      {loadError}
    </div>
  {/if}

  <Card.Root>
    <Card.Content class="flex flex-col gap-3 p-4 sm:flex-row sm:items-end">
      <div class="grid gap-2 sm:w-56">
        <Label for="reports-preset">Range</Label>
        <Select.Root
          type="single"
          name="reports-preset"
          value={preset}
          onValueChange={(value) => {
            if (value === "month" || value === "previousMonth" || value === "quarter" || value === "year") preset = value;
          }}
        >
          <Select.Trigger id="reports-preset" class="w-full">{range.label}</Select.Trigger>
          <Select.Content>
            <Select.Item value="month" label="Current month">Current month</Select.Item>
            <Select.Item value="previousMonth" label="Previous month">Previous month</Select.Item>
            <Select.Item value="quarter" label="Current quarter">Current quarter</Select.Item>
            <Select.Item value="year" label="Current year">Current year</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>

      <div class="flex-1">
        <div class="text-muted-foreground text-xs uppercase">Period</div>
        <div class="font-medium">{rangeLabel(range.start, range.end)}</div>
      </div>

      <Button
        variant="secondary"
        onclick={() => loadReports(true)}
        disabled={loading || refreshing}
        class="sm:w-28"
      >
        <RefreshCwIcon />
        Refresh
      </Button>
    </Card.Content>
  </Card.Root>

  <div class="grid grid-cols-2 gap-3 md:grid-cols-5">
    {#each statItems as item}
      <Card.Root>
        <Card.Content class="p-3">
          <div class="text-muted-foreground text-xs uppercase">{item.label}</div>
          <div class="mt-1 font-mono text-lg font-semibold tabular-nums">{item.value}</div>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>

  <div class="grid gap-4 xl:grid-cols-2">
    <Card.Root>
      <Card.Header>
        <Card.Title>Monthly Balance</Card.Title>
        <Card.Description>Overtime above the line, undertime below.</Card.Description>
      </Card.Header>
      <Card.Content>
        {#if loading}
          <div class="text-muted-foreground text-sm">Loading...</div>
        {:else if months.length === 0}
          <div class="text-muted-foreground text-sm">No data.</div>
        {:else}
          <div class="w-full overflow-x-auto">
            <svg
              viewBox="0 0 640 220"
              class="h-[240px] min-w-[560px] w-full text-xs"
              role="img"
              aria-label="Monthly balance trend"
            >
              <g class="text-muted-foreground">
                <line x1="36" x2="610" y1="96" y2="96" stroke="currentColor" stroke-opacity="0.22" />
                <text x="8" y="100" fill="currentColor">0</text>
                {#each months as month, index}
                  <line
                    x1={trendX(index, months.length)}
                    x2={trendX(index, months.length)}
                    y1="28"
                    y2="164"
                    stroke="currentColor"
                    stroke-opacity="0.08"
                  />
                  <text
                    x={trendX(index, months.length)}
                    y="190"
                    text-anchor="middle"
                    fill="currentColor"
                  >
                    {month.label}
                  </text>
                {/each}
              </g>

              <path d={trendPath()} fill="none" stroke="var(--chart-1)" stroke-width="2" />
              {#each months as month, index}
                {@const net = monthlyNet(month)}
                <circle
                  cx={trendX(index, months.length)}
                  cy={trendY(net)}
                  r="4"
                  fill={net < 0 ? "var(--destructive)" : "var(--chart-1)"}
                >
                  <title>{monthLabel(month.monthStart)} {signedHours(net)}</title>
                </circle>
              {/each}
            </svg>
          </div>
        {/if}
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Worked Time</Card.Title>
        <Card.Description>{range.end - range.start <= 45 ? "Daily" : "Weekly"} clocked totals.</Card.Description>
      </Card.Header>
      <Card.Content>
        {#if loading}
          <div class="text-muted-foreground text-sm">Loading...</div>
        {:else if workBuckets.length === 0}
          <div class="text-muted-foreground text-sm">No data.</div>
        {:else}
          <ScrollArea.Root orientation="horizontal" class="w-full pb-3">
            <div
              class="grid h-48 min-w-[560px] items-end gap-1"
              style={`grid-template-columns: repeat(${workBuckets.length}, minmax(10px, 1fr));`}
            >
              {#each workBuckets as bucket}
                <div class="flex h-44 flex-col justify-end gap-2">
                  <div
                    class="rounded-t-sm bg-primary/80"
                    style={`height: ${barHeight(bucket.value, maxWorked)}px;`}
                    title={`${bucket.label}: ${formatHours(bucket.value)}`}
                  ></div>
                  <div class="text-muted-foreground truncate text-center text-[10px]">{bucket.label}</div>
                </div>
              {/each}
            </div>
          </ScrollArea.Root>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid gap-4 xl:grid-cols-[420px_minmax(0,1fr)]">
    <Card.Root>
      <Card.Header>
        <Card.Title>Flags And Exceptions</Card.Title>
        <Card.Description>Counts inside the selected range.</Card.Description>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4">
        {#each statusBars as bar}
          <div class="grid grid-cols-[110px_1fr_36px] items-center gap-3 text-sm">
            <span class="text-muted-foreground">{bar.label}</span>
            <div class="h-2 rounded-full bg-muted">
              <div
                class="h-2 rounded-full"
                style={`width: ${(bar.value / maxStatus) * 100}%; background-color: ${bar.color};`}
              ></div>
            </div>
            <span class="text-right font-mono tabular-nums">{bar.value}</span>
          </div>
        {/each}
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Exceptions</Card.Title>
        <Card.Description>Skipped days, large balance deltas, weekend work, and vacation work.</Card.Description>
      </Card.Header>
      <Card.Content>
        <ScrollArea.Root class="h-80">
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.Head>Date</Table.Head>
                <Table.Head>Type</Table.Head>
                <Table.Head>Clocked</Table.Head>
                <Table.Head>Required</Table.Head>
                <Table.Head>Balance</Table.Head>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {#if loading}
                <Table.Row>
                  <Table.Cell colspan={5} class="text-muted-foreground py-8 text-center">
                    Loading...
                  </Table.Cell>
                </Table.Row>
              {:else if exceptions.length === 0}
                <Table.Row>
                  <Table.Cell colspan={5} class="text-muted-foreground py-8 text-center">
                    No exceptions in this range.
                  </Table.Cell>
                </Table.Row>
              {:else}
                {#each exceptions as row}
                  <Table.Row>
                    <Table.Cell class="font-medium">{fullDateLabel(row.date)}</Table.Cell>
                    <Table.Cell>{row.label}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatHours(row.clocked)}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">{formatHours(row.required)}</Table.Cell>
                    <Table.Cell class="font-mono tabular-nums">
                      {row.balance === 0 ? "0m" : signedHours(row.balance)}
                    </Table.Cell>
                  </Table.Row>
                {/each}
              {/if}
            </Table.Body>
          </Table.Root>
        </ScrollArea.Root>
      </Card.Content>
    </Card.Root>
  </div>

  <Separator />

  <div class="text-muted-foreground text-xs">
    Reports are computed from clocked intervals and day flags returned by the dashboard range API.
  </div>
</div>
