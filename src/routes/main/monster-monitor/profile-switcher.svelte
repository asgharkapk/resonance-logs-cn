<script lang="ts">
  /**
   * @file Picker for which monster-monitor sub-profile the *active loadout*
   * references. See `/main/loadouts` for managing loadouts and
   * duplicating/removing profiles shared across them.
   */
  import { SETTINGS } from "$lib/settings-store";
  import {
    activeLoadout,
    removeMonsterProfileEverywhere,
    setLoadoutMonsterProfile,
  } from "$lib/loadouts.svelte.js";
  import {
    createMonsterProfile,
    renameMonsterProfile,
  } from "$lib/monster-monitor-profile.svelte.js";
  import { t } from "$lib/i18n/index.svelte";
  import {
    confirmProfileDeletion,
    profileDisplayName,
  } from "$lib/profile-switcher-utils";

  const profiles = $derived(SETTINGS.monsterMonitor.state.profiles);
  const loadout = $derived(activeLoadout());
  const activeProfileId = $derived(loadout?.monsterProfileId ?? "");
  const activeProfile = $derived(
    profiles.find((profile) => profile.id === activeProfileId) ?? profiles[0],
  );

  function selectProfile(profileId: string) {
    if (!loadout) return;
    setLoadoutMonsterProfile(loadout.id, profileId);
  }

  function renameActiveProfile() {
    if (!activeProfile) return;
    const currentIndex = profiles.findIndex((p) => p.id === activeProfile.id);
    const nextName = window.prompt(
      t("monsterMonitor.profile.renamePrompt"),
      profileDisplayName("monster", activeProfile.name, currentIndex),
    );
    if (!nextName) return;
    const trimmedName = nextName.trim();
    if (!trimmedName) return;
    renameMonsterProfile(activeProfile.id, trimmedName);
  }

  function addProfile() {
    const nextId = createMonsterProfile("");
    selectProfile(nextId);
  }

  function removeActiveProfile() {
    if (!activeProfile || profiles.length <= 1) return;
    if (!confirmProfileDeletion("monster")) return;
    removeMonsterProfileEverywhere(activeProfile.id);
  }
</script>

<div
  class="border-border/60 bg-card/40 space-y-4 rounded-lg border p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
>
  <div>
    <h2 class="text-foreground text-base font-semibold">
      {t("monsterMonitor.profile.title")}
    </h2>
    <p class="text-muted-foreground text-xs">
      {t("monsterMonitor.profile.pickerDescription")}
    </p>
  </div>
  <div class="flex flex-wrap items-center gap-2">
    <select
      class="border-border/60 bg-muted/30 text-foreground focus:ring-primary/50 w-full rounded border px-3 py-2 text-sm focus:ring-2 focus:outline-none sm:w-72"
      value={activeProfileId}
      onchange={(event) =>
        selectProfile((event.currentTarget as HTMLSelectElement).value)}
    >
      {#each profiles as profile, idx (profile.id)}
        <option value={profile.id}
          >{profileDisplayName("monster", profile.name, idx)}</option
        >
      {/each}
    </select>
    <button
      type="button"
      class="border-border/60 text-foreground hover:bg-muted/40 rounded border px-3 py-2 text-xs transition-colors"
      onclick={addProfile}
    >
      {t("monsterMonitor.profile.new")}
    </button>
    <button
      type="button"
      class="border-border/60 text-foreground hover:bg-muted/40 rounded border px-3 py-2 text-xs transition-colors"
      onclick={renameActiveProfile}
    >
      {t("monsterMonitor.profile.rename")}
    </button>
    <button
      type="button"
      class="border-border/60 text-destructive hover:bg-destructive/10 disabled:text-muted-foreground rounded border px-3 py-2 text-xs transition-colors disabled:hover:bg-transparent"
      onclick={removeActiveProfile}
      disabled={profiles.length <= 1}
    >
      {t("monsterMonitor.profile.delete")}
    </button>
  </div>
</div>
