<script lang="ts">
  import { getLocale, t } from "$lib/i18n/index.svelte";
  import { buildLoadoutPresets } from "$lib/config/loadout-presets";
  import { createLoadoutFromPreset } from "$lib/loadouts.svelte.js";

  let { onclose }: { onclose?: () => void } = $props();

  const presets = $derived(buildLoadoutPresets(getLocale()));

  function applyPreset(preset: (typeof presets)[number]) {
    createLoadoutFromPreset(preset);
    onclose?.();
  }

  function skip() {
    onclose?.();
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center">
  <button
    class="absolute inset-0 bg-black/60 backdrop-blur-[2px]"
    onclick={skip}
    type="button"
    aria-label={t("loadout.firstRun.skip")}
  >
  </button>

  <div
    class="border-border bg-card relative z-10 flex max-h-[80vh] w-[90vw] max-w-2xl flex-col overflow-hidden rounded-xl border shadow-2xl"
  >
    <div class="border-border border-b px-6 py-4">
      <h2 class="text-xl font-semibold">{t("loadout.firstRun.title")}</h2>
      <p class="text-muted-foreground mt-1 text-sm">
        {t("loadout.firstRun.description")}
      </p>
    </div>
    <div class="flex-1 overflow-auto px-6 py-5">
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
        {#each presets as preset (preset.classKey)}
          <button
            type="button"
            class="flex flex-col items-center gap-1.5 rounded-lg border border-border/60 bg-muted/20 px-3 py-4 text-sm font-medium text-foreground transition-colors hover:border-primary/60 hover:bg-primary/10"
            onclick={() => applyPreset(preset)}
          >
            {preset.className}
          </button>
        {/each}
      </div>
    </div>
    <div class="border-border flex items-center justify-end border-t px-6 py-4">
      <button
        type="button"
        class="text-sm text-muted-foreground hover:text-foreground transition-colors"
        onclick={skip}
      >
        {t("loadout.firstRun.skip")}
      </button>
    </div>
  </div>
</div>
