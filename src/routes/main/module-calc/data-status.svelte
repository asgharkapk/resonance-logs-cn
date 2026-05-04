<script lang="ts">
  import type { ModuleInfo } from "$lib/api";
  import { formatNumber, t } from "$lib/i18n/index.svelte";

  let {
    moduleCount = null,
    modules = [],
    minTotalValue = 12,
  }: {
    moduleCount: number | null;
    modules: ModuleInfo[];
    minTotalValue: number;
  } = $props();

  const filteredModuleCount = $derived(
    modules.filter(
      (module) =>
        module.parts.reduce((total, part) => total + part.value, 0) >=
        minTotalValue,
    ).length,
  );
</script>

<div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-1">
  <div class="text-base font-semibold text-foreground">
    {t("moduleCalc.dataStatus.title")}
  </div>
  <div class="text-sm text-muted-foreground">
    {t("moduleCalc.dataStatus.moduleCount", {
      count:
        moduleCount === null
          ? t("moduleCalc.dataStatus.unsynced")
          : formatNumber(moduleCount),
    })}
  </div>
  <div class="text-sm text-muted-foreground">
    {t("moduleCalc.dataStatus.filteredCount", {
      count:
        moduleCount === null
          ? t("moduleCalc.dataStatus.unsynced")
          : formatNumber(filteredModuleCount),
    })}
  </div>
</div>
