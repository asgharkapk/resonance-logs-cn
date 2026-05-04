<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Slider } from "$lib/components/ui/slider";
  import { t } from "$lib/i18n/index.svelte";
  import type { ModuleAttrOption } from "$lib/i18n/module-calc";

  type MinReq = { attrId: number | null; value: number | null };

  let {
    attributeOptions = [],
    targetAttributes = $bindable<number[]>([]),
    excludeAttributes = $bindable<number[]>([]),
    minTotalValue = $bindable(12),
    minRequirements = $bindable<MinReq[]>([{ attrId: null, value: null }]),
  }: {
    attributeOptions: ModuleAttrOption[];
    targetAttributes: number[];
    excludeAttributes: number[];
    minTotalValue: number;
    minRequirements: MinReq[];
  } = $props();

  function toggle(list: number[], id: number): number[] {
    return list.includes(id) ? list.filter((x) => x !== id) : [...list, id];
  }

  function updateMin(
    idx: number,
    field: "attrId" | "value",
    val: number | null,
  ) {
    const next = [...minRequirements];
    const current = next[idx] ?? { attrId: null, value: null };
    if (field === "attrId") {
      next[idx] = { attrId: val, value: current.value };
    } else {
      next[idx] = { attrId: current.attrId, value: val };
    }
    minRequirements = next;
  }

  function parseNullableNumber(raw: string): number | null {
    const trimmed = raw.trim();
    if (trimmed === "") return null;
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function addMin() {
    minRequirements = [...minRequirements, { attrId: null, value: null }];
  }

  function removeMin(idx: number) {
    minRequirements = minRequirements.filter((_, i) => i !== idx);
  }
</script>

<div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4">
  <div class="text-base font-semibold text-foreground">
    {t("moduleCalc.filters.title")}
  </div>

  <div class="space-y-2">
    <div class="text-sm text-foreground">
      {t("moduleCalc.filters.minTotalValue")}
    </div>
    <div class="flex items-center gap-4">
      <Slider
        type="single"
        bind:value={minTotalValue}
        min={0}
        max={30}
        step={1}
        class="max-w-[70%]"
      />
      <div class="min-w-12 text-sm text-foreground">
        {t("moduleCalc.filters.level", { level: minTotalValue })}
      </div>
    </div>
  </div>

  <div class="space-y-2">
    <div class="text-sm text-muted-foreground">
      {t("moduleCalc.filters.targetAttributes")}
    </div>
    <div class="flex flex-wrap gap-2">
      {#each attributeOptions as opt}
        <Button
          size="sm"
          variant={targetAttributes.includes(opt.id) ? "default" : "outline"}
          onclick={() => (targetAttributes = toggle(targetAttributes, opt.id))}
        >
          {opt.label}
        </Button>
      {/each}
    </div>
  </div>

  <div class="space-y-2">
    <div class="text-sm text-muted-foreground">
      {t("moduleCalc.filters.excludeAttributes")}
    </div>
    <div class="flex flex-wrap gap-2">
      {#each attributeOptions as opt}
        <Button
          size="sm"
          variant={excludeAttributes.includes(opt.id) ? "default" : "outline"}
          onclick={() =>
            (excludeAttributes = toggle(excludeAttributes, opt.id))}
        >
          {opt.label}
        </Button>
      {/each}
    </div>
  </div>

  <div class="space-y-3">
    <div class="text-sm text-muted-foreground">
      {t("moduleCalc.filters.minRequirements")}
    </div>
    <div class="space-y-2">
      {#each minRequirements as req, idx}
        <div class="flex items-center gap-2">
          <select
            class="h-9 rounded-md border border-border bg-background px-2 text-sm"
            value={req.attrId ?? ""}
            onchange={(e) =>
              updateMin(
                idx,
                "attrId",
                parseNullableNumber((e.target as HTMLSelectElement).value),
              )}
          >
            <option value="">{t("moduleCalc.filters.selectAttribute")}</option>
            {#each attributeOptions as opt}
              <option value={opt.id}>{opt.label}</option>
            {/each}
          </select>
          <Input
            type="number"
            min="0"
            class="w-24"
            value={req.value ?? ""}
            onchange={(e) =>
              updateMin(
                idx,
                "value",
                parseNullableNumber((e.target as HTMLInputElement).value),
              )}
          />
          <Button size="sm" variant="ghost" onclick={() => removeMin(idx)}>
            {t("moduleCalc.filters.remove")}
          </Button>
        </div>
      {/each}
    </div>
    <Button size="sm" variant="outline" onclick={addMin}>
      {t("moduleCalc.filters.add")}
    </Button>
  </div>
</div>
