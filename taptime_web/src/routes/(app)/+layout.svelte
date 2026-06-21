<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import * as Alert from "$lib/components/ui/alert/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarBlock from "$lib/blocks/SidebarBlock.svelte";
  import SiteHeaderBlock from "$lib/blocks/SiteHeaderBlock.svelte";
  import { userStore } from "$lib/stores/index.js";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";

  let { children } = $props();

  onMount(async () => {
    if (!localStorage.getItem("jwt")) {
      goto("/login");
      return;
    }
    await userStore.fetch();
    if (!userStore.user && !userStore.banNotice) {
      goto("/login");
    }
  });

  function signOut() {
    userStore.clear();
    void goto("/login");
  }
</script>

{#if userStore.banNotice}
  <main class="bg-background flex min-h-svh items-center justify-center p-6">
    <Alert.Root variant="destructive" class="max-w-lg">
      <CircleAlertIcon />
      <Alert.Title>Account Access Restricted</Alert.Title>
      <Alert.Description>{userStore.banNotice}</Alert.Description>
      <div class="mt-4">
        <Button variant="outline" onclick={signOut}>Sign out</Button>
      </div>
    </Alert.Root>
  </main>
{:else}
  <Sidebar.Provider
    style="--sidebar-width: calc(var(--spacing) * 72); --header-height: calc(var(--spacing) * 12);"
  >
    <SidebarBlock variant="inset" />
    <Sidebar.Inset>
      <SiteHeaderBlock />
      <div class="flex flex-1 flex-col gap-4 p-4 pt-0 lg:p-6 lg:pt-0">
        {@render children()}
      </div>
    </Sidebar.Inset>
  </Sidebar.Provider>
{/if}
