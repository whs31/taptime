<script lang="ts">
  import { Duration } from "@bufbuild/protobuf";
  import * as Alert from "$lib/components/ui/alert/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import {
    LunchBreakSelect,
    TimeZoneSelect,
    WeekdaySelect,
    WorkHoursInput,
  } from "$lib/blocks/components";
  import { weekdayKey, weekdayLabel } from "$lib/account";
  import { durationSeconds, formatHours } from "$lib/dashboard";
  import { AuthService } from "$lib/services";
  import { userStore } from "$lib/stores";
  import { Tz } from "@taptime/proto/taptime/tz_pb.js";
  import { Weekday } from "@taptime/proto/taptime/weekday_pb.js";
  import { UpdateSettingsRequest } from "@taptime/proto/taptime/services/auth_pb.js";
  import CalendarDaysIcon from "@lucide/svelte/icons/calendar-days";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import ClockIcon from "@lucide/svelte/icons/clock";

  const LOCAL_TZ = Intl.DateTimeFormat().resolvedOptions().timeZone;
  let loadedSettingsKey = $state("");
  let timezone = $state(LOCAL_TZ);
  let workHours = $state(8);
  let workMinutes = $state(0);
  let lunchMinutes = $state(30);
  let weekends = $state<Weekday[]>([Weekday.SATURDAY, Weekday.SUNDAY]);
  let remoteDays = $state<Weekday[]>([]);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  const user = $derived(userStore.user);
  const settings = $derived(user?.settings);
  const workSeconds = $derived(workHours * 3600 + workMinutes * 60);
  const lunchSeconds = $derived(lunchMinutes * 60);
  const settingsDirty = $derived(
    Boolean(
      settings &&
        (timezone !== (user?.timeZone?.timeZone ?? LOCAL_TZ) ||
          workSeconds !== durationSeconds(settings.requiredWorkHours) ||
          lunchSeconds !== durationSeconds(settings.lunchBreakDuration) ||
          weekdayKey(weekends) !== weekdayKey(settings.weekends) ||
          weekdayKey(remoteDays) !== weekdayKey(settings.remoteDays)),
    ),
  );
  const saveDisabled = $derived(
    saving || !settingsDirty || timezone.length === 0 || workSeconds <= 0,
  );

  $effect(() => {
    if (!user?.settings) return;
    const key = [
      user.timeZone?.timeZone ?? LOCAL_TZ,
      durationSeconds(user.settings.requiredWorkHours),
      durationSeconds(user.settings.lunchBreakDuration),
      weekdayKey(user.settings.weekends),
      weekdayKey(user.settings.remoteDays),
    ].join(":");
    if (key !== loadedSettingsKey) {
      loadedSettingsKey = key;
      timezone = user.timeZone?.timeZone ?? LOCAL_TZ;
      const required = durationSeconds(user.settings.requiredWorkHours);
      workHours = Math.floor(required / 3600);
      workMinutes = Math.floor((required % 3600) / 60);
      lunchMinutes = Math.floor(durationSeconds(user.settings.lunchBreakDuration) / 60);
      weekends = [...user.settings.weekends];
      remoteDays = [...user.settings.remoteDays];
    }
  });

  async function saveSettings() {
    if (saveDisabled) return;
    saving = true;
    error = null;
    success = null;
    try {
      const updated = await AuthService.updateSettings(
        new UpdateSettingsRequest({
          timeZone: new Tz({ timeZone: timezone }),
          requiredWorkHours: new Duration({ seconds: BigInt(workSeconds) }),
          lunchBreakDuration: new Duration({ seconds: BigInt(lunchSeconds) }),
          weekends,
          remoteDays,
        }),
      );
      userStore.set(updated);
      success = "Settings saved.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex flex-col gap-6 py-4">
  <div class="flex flex-col gap-1">
    <h2 class="text-2xl font-semibold">Settings</h2>
    <p class="text-muted-foreground text-sm">
      Configure schedule defaults used for new days and dashboard calculations.
    </p>
  </div>

  {#if error}
    <Alert.Root variant="destructive">
      <CircleAlertIcon />
      <Alert.Title>Could not save settings</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  {#if success}
    <div class="border-primary/30 bg-primary/10 text-primary rounded-md border px-3 py-2 text-sm">
      {success}
    </div>
  {/if}

  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <ClockIcon class="size-4" />
          Work Schedule
        </Card.Title>
        <Card.Description>Default work hours, lunch, and time zone.</Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-5">
        <div class="grid gap-2">
          <Label for="settings-timezone">Time zone</Label>
          <TimeZoneSelect bind:value={timezone} />
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="grid gap-2">
            <Label>Required work hours per day</Label>
            <WorkHoursInput bind:hours={workHours} bind:minutes={workMinutes} />
          </div>
          <div class="grid gap-2">
            <Label>Lunch break</Label>
            <LunchBreakSelect bind:value={lunchMinutes} />
          </div>
        </div>

        <div class="grid gap-2">
          <Label>Weekend days</Label>
          <WeekdaySelect bind:value={weekends} />
        </div>

        <div class="grid gap-2">
          <Label>Remote days</Label>
          <WeekdaySelect bind:value={remoteDays} />
        </div>
      </Card.Content>
      <Card.Footer class="justify-end">
        <Button onclick={saveSettings} disabled={saveDisabled}>
          {saving ? "Saving..." : "Save Settings"}
        </Button>
      </Card.Footer>
    </Card.Root>

    <div class="flex flex-col gap-4">
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <CalendarDaysIcon class="size-4" />
            Preview
          </Card.Title>
          <Card.Description>Effective defaults for future days.</Card.Description>
        </Card.Header>
        <Card.Content class="grid gap-3 text-sm">
          <div>
            <div class="text-muted-foreground text-xs uppercase">Time Zone</div>
            <div class="font-medium">{timezone}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Required Work</div>
            <div class="font-mono">{formatHours(workSeconds)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Lunch Break</div>
            <div class="font-mono">{formatHours(lunchSeconds)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Required Day Target</div>
            <div class="font-mono">{formatHours(workSeconds + lunchSeconds)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Weekends</div>
            <div>{weekdayLabel(weekends)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Remote Days</div>
            <div>{weekdayLabel(remoteDays)}</div>
          </div>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title>Calculation Notes</Card.Title>
        </Card.Header>
        <Card.Content class="text-muted-foreground text-sm">
          Existing explicit day flags stay as they are. These defaults are applied when the server builds days from your profile settings.
        </Card.Content>
      </Card.Root>
    </div>
  </div>
</div>
