<script lang="ts">
  import type { LoadoutPreset } from "$lib/config/loadout-presets";

  let {
    preset,
    onselect,
  }: {
    preset: LoadoutPreset;
    onselect: () => void;
  } = $props();
</script>

<button
  type="button"
  class="border-border/70 bg-muted/20 hover:border-primary/60 hover:bg-primary/10 focus-visible:ring-primary/60 flex w-full cursor-pointer items-center gap-3 rounded-lg border p-3 text-left transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent focus-visible:outline-none"
  aria-label={`${preset.name} - ${preset.subtitle}`}
  onclick={onselect}
>
  <span
    class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-white/10 bg-slate-950"
  >
    <img
      src={preset.iconPath}
      alt=""
      aria-hidden="true"
      class="h-12 w-12 object-contain"
    />
  </span>

  <span class="flex min-w-0 flex-1 flex-col gap-1">
    <span class="text-foreground truncate text-sm font-semibold">
      {preset.name}
    </span>
    <span class="text-muted-foreground truncate text-xs">
      {preset.subtitle}
    </span>
    <span class="mt-1 flex items-center gap-1.5" aria-hidden="true">
      {#each preset.palette as color (color)}
        <span
          class="h-1.5 w-8 rounded-sm border border-black/10"
          style:background={color}
        ></span>
      {/each}
    </span>
  </span>
</button>
