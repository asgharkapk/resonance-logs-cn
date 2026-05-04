<script lang="ts">
  import { Switch } from "$lib/components/ui/switch";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let {
    useGpu = $bindable(true),
    gpuSupport = $bindable<{
      cuda_available: boolean;
      opencl_available: boolean;
    } | null>(null),
    combinationSize = $bindable<4 | 5>(4),
  } = $props();
</script>

<div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-3">
  <div class="text-base font-semibold text-foreground">
    {t("moduleCalc.settings.title")}
  </div>
  <div class="space-y-2">
    <div class="text-sm text-foreground">
      {t("moduleCalc.settings.moduleCount")}
    </div>
    <div class="flex items-center gap-2">
      <Button
        type="button"
        variant={combinationSize === 4 ? "default" : "outline"}
        size="sm"
        onclick={() => (combinationSize = 4)}
      >
        {t("moduleCalc.settings.moduleCountOption", { count: 4 })}
      </Button>
      <Button
        type="button"
        variant={combinationSize === 5 ? "default" : "outline"}
        size="sm"
        onclick={() => (combinationSize = 5)}
      >
        {t("moduleCalc.settings.moduleCountOption", { count: 5 })}
      </Button>
    </div>
  </div>
  <div class="flex items-center gap-3">
    <Switch bind:checked={useGpu} />
    <div class="text-sm text-foreground">
      {t("moduleCalc.settings.gpuAcceleration")}
    </div>
    {#if gpuSupport}
      <div class="text-xs text-muted-foreground">
        {t("moduleCalc.settings.cuda")}: {gpuSupport.cuda_available
          ? t("moduleCalc.settings.available")
          : t("moduleCalc.settings.unavailable")} · {t(
          "moduleCalc.settings.opencl",
        )}: {gpuSupport.opencl_available
          ? t("moduleCalc.settings.available")
          : t("moduleCalc.settings.unavailable")}
      </div>
    {/if}
  </div>
</div>
