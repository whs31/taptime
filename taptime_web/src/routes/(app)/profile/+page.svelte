<script lang="ts">
  import { goto } from "$app/navigation";
  import * as Alert from "$lib/components/ui/alert/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { AuthService } from "$lib/services";
  import { userStore } from "$lib/stores";
  import { Uid } from "@taptime/proto/taptime/uid_pb.js";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import UserIcon from "@lucide/svelte/icons/user";

  let profileLoadedKey = $state("");
  let name = $state("");
  let email = $state("");
  let organization = $state("");
  let uidInput = $state("");
  let savingProfile = $state(false);
  let savingUid = $state(false);
  let deletingData = $state(false);
  let deletingProfile = $state(false);
  let deleteDataOpen = $state(false);
  let deleteProfileOpen = $state(false);
  let deleteDataConfirm = $state("");
  let deleteProfileConfirm = $state("");
  let deleteProfilePassword = $state("");
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  const user = $derived(userStore.user);
  const existingUidLabel = $derived(formatUid(user?.rfidUid?.value));
  const parsedUid = $derived(parseUid(uidInput));
  const uidError = $derived(uidInput.trim().length > 0 && !parsedUid);
  const emailValid = $derived(/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email));
  const profileDirty = $derived(
    Boolean(
      user &&
        (name.trim() !== user.name ||
          email.trim() !== user.email ||
          (organization.trim() || undefined) !== user.organization),
    ),
  );
  const profileSaveDisabled = $derived(
    savingProfile ||
      !profileDirty ||
      name.trim().length === 0 ||
      !emailValid,
  );
  const uidDirty = $derived((parsedUid?.label ?? "") !== existingUidLabel);
  const uidSaveDisabled = $derived(savingUid || !parsedUid || !uidDirty);
  const clearUidDisabled = $derived(savingUid || !existingUidLabel);
  const deleteDataDisabled = $derived(
    deletingData || deleteDataConfirm !== "DELETE DATA",
  );
  const deleteProfileDisabled = $derived(
    deletingProfile ||
      deleteProfileConfirm !== "DELETE PROFILE" ||
      deleteProfilePassword.length === 0,
  );

  $effect(() => {
    if (!user) return;
    const key = [
      user.name,
      user.email,
      user.organization ?? "",
      existingUidLabel,
    ].join(":");
    if (key !== profileLoadedKey) {
      profileLoadedKey = key;
      name = user.name;
      email = user.email;
      organization = user.organization ?? "";
      uidInput = existingUidLabel;
    }
  });

  function formatUid(bytes?: Uint8Array) {
    if (!bytes || bytes.length === 0) return "";
    return [...bytes].map((byte) => byte.toString(16).padStart(2, "0").toUpperCase()).join(" ");
  }

  function parseUid(value: string): { bytes: Uint8Array; label: string } | null {
    const cleaned = value.replace(/[\s:;.,-]/g, "").toUpperCase();
    if (cleaned.length === 0) return null;
    if (![8, 14, 20].includes(cleaned.length)) return null;
    if (!/^[0-9A-F]+$/.test(cleaned)) return null;

    const bytes = new Uint8Array(cleaned.length / 2);
    for (let i = 0; i < bytes.length; i += 1) {
      bytes[i] = Number.parseInt(cleaned.slice(i * 2, i * 2 + 2), 16);
    }
    return { bytes, label: formatUid(bytes) };
  }

  function timestampLabel(value?: { seconds: bigint; nanos: number }) {
    if (!value) return "Never";
    const ms = Number(value.seconds) * 1000 + Math.floor(value.nanos / 1_000_000);
    return new Intl.DateTimeFormat("en-US", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(ms));
  }

  async function saveProfile() {
    if (profileSaveDisabled) return;
    savingProfile = true;
    error = null;
    success = null;
    try {
      const updated = await AuthService.updateProfile(name, email, organization);
      userStore.set(updated);
      success = "Profile saved.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      savingProfile = false;
    }
  }

  async function saveUid() {
    if (uidSaveDisabled || !parsedUid) return;
    savingUid = true;
    error = null;
    success = null;
    try {
      const updated = await AuthService.updateRfidUid(
        new Uid({ value: parsedUid.bytes }),
      );
      userStore.set(updated);
      uidInput = parsedUid.label;
      success = "RFID UID saved.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      savingUid = false;
    }
  }

  async function clearUid() {
    if (clearUidDisabled) return;
    savingUid = true;
    error = null;
    success = null;
    try {
      const updated = await AuthService.updateRfidUid();
      userStore.set(updated);
      uidInput = "";
      success = "RFID UID cleared.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      savingUid = false;
    }
  }

  async function deleteTimeData() {
    if (deleteDataDisabled) return;
    deletingData = true;
    error = null;
    success = null;
    try {
      await AuthService.deleteTimeData();
      deleteDataOpen = false;
      deleteDataConfirm = "";
      success = "Time data deleted.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      deletingData = false;
    }
  }

  async function deleteProfile() {
    if (deleteProfileDisabled) return;
    deletingProfile = true;
    error = null;
    success = null;
    try {
      await AuthService.deleteAccount(deleteProfilePassword);
      userStore.clear();
      await goto("/login");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      deletingProfile = false;
    }
  }
</script>

<div class="flex flex-col gap-6 py-4">
  <div class="flex flex-col gap-1">
    <h2 class="text-2xl font-semibold">Profile</h2>
    <p class="text-muted-foreground text-sm">
      Manage account identity, MCU interop, and profile lifecycle.
    </p>
  </div>

  {#if error}
    <Alert.Root variant="destructive">
      <CircleAlertIcon />
      <Alert.Title>Could not save changes</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  {#if success}
    <div class="border-primary/30 bg-primary/10 text-primary rounded-md border px-3 py-2 text-sm">
      {success}
    </div>
  {/if}

  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
    <div class="flex flex-col gap-4">
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <UserIcon class="size-4" />
            Account Identity
          </Card.Title>
          <Card.Description>Name, email, and optional organization.</Card.Description>
        </Card.Header>
        <Card.Content class="grid gap-4">
          <div class="grid gap-2">
            <Label for="profile-name">Name</Label>
            <Input id="profile-name" bind:value={name} autocomplete="name" />
          </div>

          <div class="grid gap-2">
            <Label for="profile-email">Email</Label>
            <Input
              id="profile-email"
              type="email"
              bind:value={email}
              autocomplete="email"
              aria-invalid={email.length > 0 && !emailValid}
            />
            {#if email.length > 0 && !emailValid}
              <span class="text-destructive text-xs">Enter a valid email address.</span>
            {/if}
          </div>

          <div class="grid gap-2">
            <Label for="profile-organization">Organization</Label>
            <Input
              id="profile-organization"
              bind:value={organization}
              placeholder="Optional"
            />
          </div>
        </Card.Content>
        <Card.Footer class="justify-end">
          <Button onclick={saveProfile} disabled={profileSaveDisabled}>
            {savingProfile ? "Saving..." : "Save Profile"}
          </Button>
        </Card.Footer>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <CreditCardIcon class="size-4" />
            RFID UID
          </Card.Title>
          <Card.Description>Assign a 4, 7, or 10 byte UID for MCU taps.</Card.Description>
        </Card.Header>
        <Card.Content class="grid gap-4">
          <div class="grid gap-2">
            <Label for="profile-rfid">UID</Label>
            <Input
              id="profile-rfid"
              bind:value={uidInput}
              placeholder="A1 B2 C3 D4"
              autocapitalize="characters"
              spellcheck="false"
              aria-invalid={uidError}
            />
            <span class="text-muted-foreground text-xs">
              Separators are optional. Supported lengths are 4, 7, or 10 bytes.
            </span>
            {#if uidError}
              <span class="text-destructive text-xs">
                Enter 8, 14, or 20 hex characters.
              </span>
            {/if}
          </div>

          <div class="rounded-md border bg-muted/30 px-3 py-2 text-sm">
            <div class="text-muted-foreground text-xs uppercase">Current UID</div>
            <div class="mt-1 font-mono tabular-nums">
              {existingUidLabel || "Not assigned"}
            </div>
          </div>
        </Card.Content>
        <Card.Footer class="justify-end gap-2">
          <Button variant="secondary" onclick={clearUid} disabled={clearUidDisabled}>
            Clear UID
          </Button>
          <Button onclick={saveUid} disabled={uidSaveDisabled}>
            {savingUid ? "Saving..." : "Save UID"}
          </Button>
        </Card.Footer>
      </Card.Root>
    </div>

    <div class="flex flex-col gap-4">
      <Card.Root>
        <Card.Header>
          <Card.Title>Account Metadata</Card.Title>
          <Card.Description>Read-only account details.</Card.Description>
        </Card.Header>
        <Card.Content class="grid gap-3 text-sm">
          <div>
            <div class="text-muted-foreground text-xs uppercase">Created</div>
            <div>{timestampLabel(user?.createdAt)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Last Seen</div>
            <div>{timestampLabel(user?.lastSeen)}</div>
          </div>
          <div>
            <div class="text-muted-foreground text-xs uppercase">Time Zone</div>
            <div>{user?.timeZone?.timeZone ?? "Unknown"}</div>
          </div>
        </Card.Content>
      </Card.Root>

      <Card.Root class="border-destructive/30">
        <Card.Header>
          <Card.Title class="flex items-center gap-2 text-destructive">
            <Trash2Icon class="size-4" />
            Danger Zone
          </Card.Title>
          <Card.Description>Destructive actions require confirmation.</Card.Description>
        </Card.Header>
        <Card.Content class="flex flex-col gap-4">
          <div class="rounded-md border p-3">
            <div class="font-medium">Delete all time data</div>
            <p class="text-muted-foreground mt-1 text-sm">
              Removes check-in/out events and day flags. Profile, settings, UID, and login stay.
            </p>
            <Dialog.Root bind:open={deleteDataOpen}>
              <Dialog.Trigger>
                {#snippet child({ props })}
                  <Button {...props} variant="destructive" class="mt-3">
                    Delete Time Data
                  </Button>
                {/snippet}
              </Dialog.Trigger>
              <Dialog.Content>
                <Dialog.Header>
                  <Dialog.Title>Delete all time data?</Dialog.Title>
                  <Dialog.Description>
                    This removes events and day flags. Type DELETE DATA to continue.
                  </Dialog.Description>
                </Dialog.Header>
                <div class="grid gap-2">
                  <Label for="delete-data-confirm">Confirmation</Label>
                  <Input
                    id="delete-data-confirm"
                    bind:value={deleteDataConfirm}
                    placeholder="DELETE DATA"
                  />
                </div>
                <Dialog.Footer>
                  <Button variant="ghost" onclick={() => (deleteDataOpen = false)}>Cancel</Button>
                  <Button variant="destructive" onclick={deleteTimeData} disabled={deleteDataDisabled}>
                    {deletingData ? "Deleting..." : "Delete Data"}
                  </Button>
                </Dialog.Footer>
              </Dialog.Content>
            </Dialog.Root>
          </div>

          <Separator />

          <div class="rounded-md border border-destructive/30 p-3">
            <div class="font-medium">Delete profile</div>
            <p class="text-muted-foreground mt-1 text-sm">
              Permanently removes the account, credentials, profile, and all user-owned data.
            </p>
            <Dialog.Root bind:open={deleteProfileOpen}>
              <Dialog.Trigger>
                {#snippet child({ props })}
                  <Button {...props} variant="destructive" class="mt-3">
                    Delete Profile
                  </Button>
                {/snippet}
              </Dialog.Trigger>
              <Dialog.Content>
                <Dialog.Header>
                  <Dialog.Title>Delete profile?</Dialog.Title>
                  <Dialog.Description>
                    Enter your password and type DELETE PROFILE to permanently delete the account.
                  </Dialog.Description>
                </Dialog.Header>
                <div class="grid gap-3">
                  <div class="grid gap-2">
                    <Label for="delete-profile-password">Password</Label>
                    <Input
                      id="delete-profile-password"
                      type="password"
                      bind:value={deleteProfilePassword}
                      autocomplete="current-password"
                    />
                  </div>
                  <div class="grid gap-2">
                    <Label for="delete-profile-confirm">Confirmation</Label>
                    <Input
                      id="delete-profile-confirm"
                      bind:value={deleteProfileConfirm}
                      placeholder="DELETE PROFILE"
                    />
                  </div>
                </div>
                <Dialog.Footer>
                  <Button variant="ghost" onclick={() => (deleteProfileOpen = false)}>Cancel</Button>
                  <Button
                    variant="destructive"
                    onclick={deleteProfile}
                    disabled={deleteProfileDisabled}
                  >
                    {deletingProfile ? "Deleting..." : "Delete Profile"}
                  </Button>
                </Dialog.Footer>
              </Dialog.Content>
            </Dialog.Root>
          </div>
        </Card.Content>
      </Card.Root>
    </div>
  </div>
</div>
