<script lang="ts">
  import {
    SETTINGS,
    createDefaultSkillMonitorProfile,
  } from "$lib/settings-store";
  import {
    activeProfileOrDefault,
    clampedProfileIndex,
    updateActiveProfile,
  } from "$lib/skill-monitor-profile.svelte.js";
  import { t } from "$lib/i18n/index.svelte";

  const profiles = $derived(SETTINGS.skillMonitor.state.profiles);
  const activeProfileIndex = $derived.by(() => clampedProfileIndex());
  const activeProfile = $derived.by(() => activeProfileOrDefault());

  function setActiveProfileIndex(index: number) {
    const maxIndex = Math.max(0, SETTINGS.skillMonitor.state.profiles.length - 1);
    SETTINGS.skillMonitor.state.activeProfileIndex = Math.min(
      Math.max(index, 0),
      maxIndex,
    );
  }

  function updateActiveProfileName(name: string) {
    updateActiveProfile((profile) => ({ ...profile, name }));
  }

  function getProfileDisplayName(name: string | undefined, index: number): string {
    const trimmed = name?.trim();
    if (trimmed) return trimmed;
    return index === 0
      ? t("skillMonitor.defaults.defaultProfileName")
      : t("skillMonitor.defaults.profileName", { index: index + 1 });
  }

  function addProfile() {
    const nextProfile = createDefaultSkillMonitorProfile("");
    SETTINGS.skillMonitor.state.profiles = [
      ...SETTINGS.skillMonitor.state.profiles,
      nextProfile,
    ];
    SETTINGS.skillMonitor.state.activeProfileIndex =
      SETTINGS.skillMonitor.state.profiles.length - 1;
  }

  function renameActiveProfile() {
    const nextName = window.prompt(
      t("skillMonitor.profile.renamePrompt"),
      getProfileDisplayName(activeProfile.name, activeProfileIndex),
    );
    if (!nextName) return;
    const trimmedName = nextName.trim();
    if (!trimmedName) return;
    updateActiveProfileName(trimmedName);
  }

  function removeActiveProfile() {
    const state = SETTINGS.skillMonitor.state;
    if (state.profiles.length <= 1) return;
    const index = Math.min(
      Math.max(state.activeProfileIndex, 0),
      state.profiles.length - 1,
    );
    state.profiles = state.profiles.filter((_, i) => i !== index);
    state.activeProfileIndex = Math.min(index, state.profiles.length - 1);
  }
</script>

<div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
  <div>
    <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.profile.title")}</h2>
    <p class="text-xs text-muted-foreground">{t("skillMonitor.profile.description")}</p>
  </div>
  <div class="flex flex-wrap items-center gap-2">
    <select
      class="w-full sm:w-72 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
      value={activeProfileIndex}
      onchange={(event) =>
        setActiveProfileIndex(Number((event.currentTarget as HTMLSelectElement).value))}
    >
      {#each profiles as profile, idx (idx)}
        <option value={idx}>{getProfileDisplayName(profile.name, idx)}</option>
      {/each}
    </select>
    <button
      type="button"
      class="text-xs px-3 py-2 rounded border border-border/60 text-foreground hover:bg-muted/40 transition-colors"
      onclick={addProfile}
    >
      {t("skillMonitor.profile.new")}
    </button>
    <button
      type="button"
      class="text-xs px-3 py-2 rounded border border-border/60 text-foreground hover:bg-muted/40 transition-colors"
      onclick={renameActiveProfile}
    >
      {t("skillMonitor.profile.rename")}
    </button>
    <button
      type="button"
      class="text-xs px-3 py-2 rounded border border-border/60 text-destructive hover:bg-destructive/10 transition-colors disabled:text-muted-foreground disabled:hover:bg-transparent"
      onclick={removeActiveProfile}
      disabled={profiles.length <= 1}
    >
      {t("skillMonitor.profile.delete")}
    </button>
  </div>
</div>
