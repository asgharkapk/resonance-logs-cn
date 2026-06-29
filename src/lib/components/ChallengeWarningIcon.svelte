<script lang="ts">
  import AlertTriangle from "virtual:icons/lucide/alert-triangle";
  import { tooltip } from "$lib/utils.svelte";
  import { t, getLocale } from "$lib/i18n/index.svelte";
  import { getPresetForDamageId } from "$lib/challenge-presets";
  import { lookupDamageIdName } from "$lib/config/recount-table";

  let { ids = [] }: { ids?: number[] } = $props();

  // Resolve each matched damage_id to a readable name: prefer a curated preset
  // label (names the challenge), else the game's own damage name. Dedupe so
  // repeated ids of the same mechanic collapse into one line.
  const detail = $derived.by(() => {
    const locale = getLocale();
    const labels = new Set<string>();
    for (const id of ids) {
      const preset = getPresetForDamageId(id);
      labels.add(preset ? t(preset.labelKey) : lookupDamageIdName(id, locale));
    }
    return [...labels].join(", ");
  });

  const tooltipText = $derived(
    t("challengeWatch.warningTooltip", { ids: detail }),
  );
</script>

<span
  class="inline-flex shrink-0 items-center text-red-500"
  role="img"
  aria-label={t("challengeWatch.warningAria")}
  {@attach tooltip(() => tooltipText)}
>
  <AlertTriangle class="h-[1em] w-[1em]" />
</span>
