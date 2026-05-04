<script lang="ts">
  import BuffSearchResultGrid from "$lib/components/BuffSearchResultGrid.svelte";
  import type { BuffDefinition, BuffNameInfo } from "$lib/config/buff-name-table";
  import type {
    CustomPanelGroup,
    CustomPanelStyle,
    InlineBuffEntry,
    UserCounterRule,
  } from "$lib/settings-store";
  import type {
    CounterRulePreset,
    SlotTemplate,
    SourceTemplate,
  } from "$lib/skill-mappings";
  import { t } from "$lib/i18n/index.svelte";

  type CounterRuleOption = CounterRulePreset & { origin: "preset" | "user" };

  interface Props {
    counterRules: CounterRuleOption[];
    sourceTemplates: SourceTemplate[];
    slotTemplates: SlotTemplate[];
    userCounterRules: UserCounterRule[];
    availableBuffMap: Map<number, BuffDefinition>;
    getBuffDisplayName: (buffId: number) => string;
    inlineBuffSearch: string;
    filteredInlineBuffSearchResults: BuffNameInfo[];
    customPanelGroups: CustomPanelGroup[];
    customPanelStyle: CustomPanelStyle;
    setInlineBuffSearch: (value: string) => void;
    addCustomPanelGroup: () => void;
    removeCustomPanelGroup: (groupId: string) => void;
    renameCustomPanelGroup: (groupId: string, name: string) => void;
    addCustomPanelEntry: (
      groupId: string,
      sourceType: "buff" | "counter",
      sourceId: number,
      counterSlotId?: number,
    ) => void;
    addUserCounterRule: (name: string, sourceRefs: string[], slotRefs: string[]) => void;
    removeUserCounterRule: (ruleId: number) => void;
    updateUserCounterRule: (ruleId: number, updates: Partial<UserCounterRule>) => void;
    removeCustomPanelEntry: (groupId: string, entryId: string) => void;
    setCustomPanelEntryLabel: (groupId: string, entryId: string, label: string) => void;
    moveCustomPanelEntry: (
      groupId: string,
      entryId: string,
      direction: "up" | "down",
    ) => void;
    setCustomPanelGap: (value: number) => void;
    setCustomPanelFontSize: (value: number) => void;
    setCustomPanelColumnGap: (value: number) => void;
    setCustomPanelNameColor: (value: string) => void;
    setCustomPanelValueColor: (value: string) => void;
    setCustomPanelProgressColor: (value: string) => void;
    setCustomPanelProgressOpacity: (value: number) => void;
  }

  let {
    counterRules,
    sourceTemplates,
    slotTemplates,
    userCounterRules,
    availableBuffMap,
    getBuffDisplayName,
    inlineBuffSearch,
    filteredInlineBuffSearchResults,
    customPanelGroups,
    customPanelStyle,
    setInlineBuffSearch,
    addCustomPanelGroup,
    removeCustomPanelGroup,
    renameCustomPanelGroup,
    addCustomPanelEntry,
    addUserCounterRule,
    removeUserCounterRule,
    updateUserCounterRule,
    removeCustomPanelEntry,
    setCustomPanelEntryLabel,
    moveCustomPanelEntry,
    setCustomPanelGap,
    setCustomPanelFontSize,
    setCustomPanelColumnGap,
    setCustomPanelNameColor,
    setCustomPanelValueColor,
    setCustomPanelProgressColor,
    setCustomPanelProgressOpacity,
  }: Props = $props();

  let selectedGroupId = $state<string | null>(customPanelGroups[0]?.id ?? null);
  let draftRuleName = $state("");
  let draftSourceRefs = $state<string[]>([]);
  let draftSlotRefs = $state<string[]>([]);
  let isCreatingUserRule = $state(false);

  $effect(() => {
    if (customPanelGroups.length === 0) {
      selectedGroupId = null;
      return;
    }
    if (!selectedGroupId || !customPanelGroups.some((group) => group.id === selectedGroupId)) {
      selectedGroupId = customPanelGroups[0]?.id ?? null;
    }
  });

  const selectedGroup = $derived.by(
    () => customPanelGroups.find((group) => group.id === selectedGroupId) ?? null,
  );
  const selectedGroupIndex = $derived.by(
    () => customPanelGroups.findIndex((group) => group.id === selectedGroupId),
  );
  const sourceTemplateMap = $derived.by(
    () => new Map(sourceTemplates.map((template) => [template.sourceId, template])),
  );
  const slotTemplateMap = $derived.by(
    () => new Map(slotTemplates.map((template) => [template.slotTemplateId, template])),
  );
  const canSaveDraftRule = $derived(
    draftRuleName.trim().length > 0 && draftSourceRefs.length > 0 && draftSlotRefs.length > 0,
  );

  function getEntryLocation(
    sourceType: InlineBuffEntry["sourceType"],
    sourceId: number,
    counterSlotId?: number,
  ): { groupId: string; groupName: string } | null {
    for (let index = 0; index < customPanelGroups.length; index += 1) {
      const group = customPanelGroups[index];
      if (!group) continue;
      if (group.entries.some((entry) =>
        entry.sourceType === sourceType
        && entry.sourceId === sourceId
        && (sourceType !== "counter" || entry.counterSlotId === counterSlotId)
      )) {
        return { groupId: group.id, groupName: getCustomPanelGroupDisplayName(group, index) };
      }
    }
    return null;
  }

  function buffStatusLabel(buffId: number): string | null {
    const location = getEntryLocation("buff", buffId);
    if (!location) return null;
    return location.groupId === selectedGroup?.id
      ? t("skillMonitor.customPanel.status.currentGroupAdded")
      : t("skillMonitor.customPanel.status.alreadyIn", { name: location.groupName });
  }

  function getCustomPanelGroupDisplayName(
    group: CustomPanelGroup,
    index: number,
  ): string {
    return group.name.trim() || t("skillMonitor.defaults.customPanelGroupName", {
      index: index + 1,
    });
  }

  function getSelectedGroupDisplayName(): string {
    if (!selectedGroup) return "";
    return getCustomPanelGroupDisplayName(
      selectedGroup,
      selectedGroupIndex >= 0 ? selectedGroupIndex : 0,
    );
  }

  function toggleDraftRef(
    current: string[],
    value: string,
  ): string[] {
    return current.includes(value)
      ? current.filter((item) => item !== value)
      : [...current, value];
  }

  function resetDraftRule() {
    draftRuleName = "";
    draftSourceRefs = [];
    draftSlotRefs = [];
    isCreatingUserRule = false;
  }

  function submitDraftRule() {
    if (!canSaveDraftRule) return;
    addUserCounterRule(draftRuleName, draftSourceRefs, draftSlotRefs);
    resetDraftRule();
  }

  function getUserRuleSourceNames(rule: UserCounterRule): string {
    return rule.sourceRefs
      .map((ref) => sourceTemplateMap.get(ref)?.name ?? ref)
      .join("、");
  }

  function getUserRuleSlotNames(rule: UserCounterRule): string {
    return rule.slotRefs
      .map((ref) => slotTemplateMap.get(ref)?.name ?? ref)
      .join("、");
  }
</script>

<div class="space-y-6">
  <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-4">
    <div>
      <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.customPanel.title")}</h2>
      <p class="text-xs text-muted-foreground">
        {t("skillMonitor.customPanel.description")}
      </p>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <button
        type="button"
        class="min-h-11 rounded-lg border border-border/60 bg-muted/20 px-4 py-2 text-sm font-medium text-foreground transition-colors hover:bg-muted/40 cursor-pointer"
        onclick={addCustomPanelGroup}
      >
        {t("skillMonitor.customPanel.new")}
      </button>
      <div class="text-xs text-muted-foreground" role="status" aria-live="polite">
        {#if selectedGroup}
          {t("skillMonitor.customPanel.currentEditing", { name: getSelectedGroupDisplayName() })}
        {:else}
          {t("skillMonitor.customPanel.chooseOrCreate")}
        {/if}
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {#each customPanelGroups as group, groupIndex (group.id)}
        {@const isSelected = group.id === selectedGroup?.id}
        <div
          class="rounded-lg border px-3 py-3 transition-colors {isSelected
            ? 'border-primary bg-primary/10'
            : 'border-border/60 bg-muted/20'}"
        >
          <div class="flex items-start justify-between gap-3">
            <button
              type="button"
              class="flex-1 text-left cursor-pointer"
              onclick={() => (selectedGroupId = group.id)}
            >
              <div class="text-sm font-medium text-foreground">{getCustomPanelGroupDisplayName(group, groupIndex)}</div>
              <div class="mt-1 text-xs text-muted-foreground">
                {t("skillMonitor.customPanel.entryCount", { count: group.entries.length })}
              </div>
            </button>
            <button
              type="button"
              class="min-h-11 rounded-md border border-border/60 px-3 py-1.5 text-xs text-destructive transition-colors hover:bg-destructive/10 cursor-pointer"
              onclick={() => removeCustomPanelGroup(group.id)}
            >
              {t("skillMonitor.common.delete")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>

  {#if selectedGroup}
    <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-4">
      <div class="space-y-1">
        <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.currentGroup")}</div>
        <p class="text-xs text-muted-foreground">
          {t("skillMonitor.customPanel.currentGroupDescription")}
        </p>
      </div>
      <label class="block text-xs text-muted-foreground">
        {t("skillMonitor.customPanel.groupName")}
        <input
          class="mt-1 w-full max-w-sm rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
          value={selectedGroup.name}
          placeholder={getSelectedGroupDisplayName()}
          oninput={(event) =>
            renameCustomPanelGroup(selectedGroup.id, (event.currentTarget as HTMLInputElement).value)}
        />
      </label>
    </div>

    <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-3">
      <div class="space-y-1">
        <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.addBuff")}</div>
        <p class="text-xs text-muted-foreground">{t("skillMonitor.customPanel.addBuffDescription")}</p>
      </div>
      <input
        class="w-full sm:w-80 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
        placeholder={t("skillMonitor.customPanel.addBuffPlaceholder")}
        value={inlineBuffSearch}
        oninput={(event) => setInlineBuffSearch((event.currentTarget as HTMLInputElement).value)}
      />
      {#if inlineBuffSearch.trim().length > 0}
        <BuffSearchResultGrid
          items={filteredInlineBuffSearchResults}
          {availableBuffMap}
          onSelect={(buffId) => addCustomPanelEntry(selectedGroup.id, "buff", buffId)}
          isDisabled={(buffId) => Boolean(getEntryLocation("buff", buffId))}
          getStatusLabel={buffStatusLabel}
          emptyMessage={t("skillMonitor.customPanel.noMatchingBuff")}
        />
      {/if}
    </div>

    <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-3">
      <div class="space-y-1">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.rule.title")}</div>
            <p class="text-xs text-muted-foreground">
              {t("skillMonitor.customPanel.rule.description")}
            </p>
          </div>
          <button
            type="button"
            class="min-h-11 rounded-lg border border-border/60 bg-muted/20 px-4 py-2 text-sm font-medium text-foreground transition-colors hover:bg-muted/40 cursor-pointer"
            onclick={() => {
              isCreatingUserRule = !isCreatingUserRule;
              if (!isCreatingUserRule) {
                draftRuleName = "";
                draftSourceRefs = [];
                draftSlotRefs = [];
              }
            }}
          >
            {isCreatingUserRule
              ? t("skillMonitor.customPanel.rule.collapse")
              : t("skillMonitor.customPanel.rule.new")}
          </button>
        </div>
      </div>

      {#if userCounterRules.length === 0}
        <div class="rounded-lg border border-dashed border-border/60 bg-muted/10 px-3 py-6 text-center text-sm text-muted-foreground">
          {t("skillMonitor.customPanel.rule.empty")}
        </div>
      {/if}

      <div class="space-y-3">
        {#each userCounterRules as rule (rule.ruleId)}
          <div class="rounded-lg border border-border/60 bg-muted/20 p-3 space-y-2">
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div class="flex-1 min-w-0 space-y-2">
                <label class="block text-xs text-muted-foreground">
                  {t("skillMonitor.customPanel.rule.name")}
                  <input
                    class="mt-1 w-full rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                    value={rule.name}
                    oninput={(event) =>
                      updateUserCounterRule(rule.ruleId, {
                        name: (event.currentTarget as HTMLInputElement).value,
                      })}
                  />
                </label>
                <div class="text-xs text-muted-foreground">
                  {t("skillMonitor.customPanel.rule.sourceSummary", {
                    value: getUserRuleSourceNames(rule)
                      || t("skillMonitor.customPanel.rule.notConfigured"),
                  })}
                </div>
                <div class="text-xs text-muted-foreground">
                  {t("skillMonitor.customPanel.rule.slotSummary", {
                    value: getUserRuleSlotNames(rule)
                      || t("skillMonitor.customPanel.rule.notConfigured"),
                  })}
                </div>
              </div>
              <button
                type="button"
                class="min-h-11 rounded-md border border-border/60 px-3 py-1.5 text-xs text-destructive transition-colors hover:bg-destructive/10 cursor-pointer"
                onclick={() => removeUserCounterRule(rule.ruleId)}
              >
                {t("skillMonitor.common.delete")}
              </button>
            </div>
          </div>
        {/each}
      </div>

      {#if isCreatingUserRule}
        <div class="rounded-lg border border-primary/30 bg-primary/5 p-4 space-y-4">
          <label class="block text-xs text-muted-foreground">
            {t("skillMonitor.customPanel.rule.name")}
            <input
              class="mt-1 w-full rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
              value={draftRuleName}
              placeholder={t("skillMonitor.customPanel.rule.placeholder")}
              oninput={(event) => (draftRuleName = (event.currentTarget as HTMLInputElement).value)}
            />
          </label>

          <div class="space-y-2">
            <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.rule.sources")}</div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
              {#each sourceTemplates as template (template.sourceId)}
                {@const selected = draftSourceRefs.includes(template.sourceId)}
                <button
                  type="button"
                  class="min-h-11 text-left rounded border px-3 py-2 transition-colors cursor-pointer {selected
                    ? 'border-primary bg-primary/10'
                    : 'border-border/60 bg-muted/20 hover:bg-muted/40'}"
                  onclick={() => (draftSourceRefs = toggleDraftRef(draftSourceRefs, template.sourceId))}
                >
                  <div class="text-sm font-medium text-foreground">{template.name}</div>
                </button>
              {/each}
            </div>
          </div>

          <div class="space-y-2">
            <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.rule.slots")}</div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
              {#each slotTemplates as template (template.slotTemplateId)}
                {@const selected = draftSlotRefs.includes(template.slotTemplateId)}
                <button
                  type="button"
                  class="min-h-11 text-left rounded border px-3 py-2 transition-colors cursor-pointer {selected
                    ? 'border-primary bg-primary/10'
                    : 'border-border/60 bg-muted/20 hover:bg-muted/40'}"
                  onclick={() => (draftSlotRefs = toggleDraftRef(draftSlotRefs, template.slotTemplateId))}
                >
                  <div class="text-sm font-medium text-foreground">{template.name}</div>
                </button>
              {/each}
            </div>
          </div>

          <div class="flex flex-wrap justify-end gap-2">
            <button
              type="button"
              class="min-h-11 rounded border border-border/60 px-4 py-2 text-sm text-foreground hover:bg-muted/40 cursor-pointer"
              onclick={resetDraftRule}
            >
              {t("skillMonitor.common.cancel")}
            </button>
            <button
              type="button"
              class="min-h-11 rounded border border-primary/60 bg-primary/15 px-4 py-2 text-sm font-medium text-foreground transition-colors disabled:cursor-not-allowed disabled:opacity-60 cursor-pointer"
              onclick={submitDraftRule}
              disabled={!canSaveDraftRule}
            >
              {t("skillMonitor.common.save")}
            </button>
          </div>
        </div>
      {/if}
    </div>

    <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-3">
      <div class="space-y-1">
        <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.counter.title")}</div>
        <p class="text-xs text-muted-foreground">
          {t("skillMonitor.customPanel.counter.description")}
        </p>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        {#each counterRules as rule (rule.ruleId)}
          {#each rule.effectSlots as slot (slot.slotId)}
            {@const location = getEntryLocation("counter", rule.ruleId, slot.slotId)}
            {@const exists = Boolean(location)}
            <button
              type="button"
              class="min-h-11 text-left rounded border px-3 py-2 transition-colors cursor-pointer disabled:cursor-not-allowed disabled:opacity-100 {exists
                ? 'border-primary bg-primary/10'
                : 'border-border/60 bg-muted/20 hover:bg-muted/40'}"
              onclick={() => addCustomPanelEntry(selectedGroup.id, "counter", rule.ruleId, slot.slotId)}
              disabled={exists}
            >
              <div class="flex items-center justify-between gap-2">
                <div>
                  <div class="text-sm font-medium text-foreground">
                    {rule.name}{rule.effectSlots.length > 1 ? ` #${slot.slotId}` : ""}
                  </div>
                  <div class="mt-1 text-xs text-muted-foreground">
                    <span class="inline-block rounded border border-border/60 bg-muted/30 px-1.5 py-0.5">
                      {rule.origin === "user"
                        ? t("skillMonitor.common.custom")
                        : t("skillMonitor.common.preset")}
                    </span>
                  </div>
                </div>
                <div class="text-xs {exists ? 'text-primary' : 'text-muted-foreground'}">
                  {#if !exists}
                    {t("skillMonitor.customPanel.counter.add")}
                  {:else if location?.groupId === selectedGroup.id}
                    {t("skillMonitor.customPanel.status.currentGroupAdded")}
                  {:else}
                    {t("skillMonitor.customPanel.status.alreadyIn", {
                      name: location?.groupName ?? "",
                    })}
                  {/if}
                </div>
              </div>
            </button>
          {/each}
        {/each}
      </div>
    </div>

    <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-3">
      <div class="text-sm font-medium text-foreground">{t("skillMonitor.customPanel.entries.title")}</div>
      {#if selectedGroup.entries.length === 0}
        <div class="rounded-lg border border-dashed border-border/60 bg-muted/10 px-3 py-6 text-center text-sm text-muted-foreground">
          {t("skillMonitor.customPanel.entries.empty")}
        </div>
      {/if}
      {#each selectedGroup.entries as entry, idx (entry.id)}
        {@const counterRule = entry.sourceType === "counter"
          ? counterRules.find((item) => item.ruleId === entry.sourceId)
          : null}
        {@const counterSlot = entry.sourceType === "counter"
          ? counterRule?.effectSlots.find((slot) => slot.slotId === entry.counterSlotId)
          : null}
        {@const buffName = entry.sourceType === "buff" ? getBuffDisplayName(entry.sourceId) : null}
        <div class="rounded-lg border border-border/60 bg-muted/20 p-3 space-y-2">
          <div class="text-xs text-muted-foreground">
            {entry.sourceType === "counter"
              ? t("skillMonitor.customPanel.entries.sourceCounter", {
                name: `${counterRule?.name ?? `#${entry.sourceId}`}${counterSlot ? ` #${counterSlot.slotId}` : ""}`,
              })
              : t("skillMonitor.customPanel.entries.sourceBuff", { name: buffName ?? "" })}
          </div>
          {#if entry.sourceType === "counter"}
            <input
              class="w-full rounded border border-border/60 bg-muted/30 px-2 py-1.5 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
              value={entry.label}
              placeholder={t("skillMonitor.customPanel.entries.labelPlaceholder")}
              oninput={(event) =>
                setCustomPanelEntryLabel(
                  selectedGroup.id,
                  entry.id,
                  (event.currentTarget as HTMLInputElement).value,
                )}
            />
          {:else}
            <div class="rounded border border-border/60 bg-muted/30 px-2 py-1.5 text-sm text-foreground">
              {buffName}
            </div>
          {/if}
          <div class="flex justify-end gap-2">
            <button
              type="button"
              class="min-h-11 rounded border border-border/60 px-3 py-1 text-xs hover:bg-muted/40 disabled:opacity-50 cursor-pointer"
              onclick={() => moveCustomPanelEntry(selectedGroup.id, entry.id, "up")}
              disabled={idx === 0}
            >
              {t("skillMonitor.common.moveUp")}
            </button>
            <button
              type="button"
              class="min-h-11 rounded border border-border/60 px-3 py-1 text-xs hover:bg-muted/40 disabled:opacity-50 cursor-pointer"
              onclick={() => moveCustomPanelEntry(selectedGroup.id, entry.id, "down")}
              disabled={idx === selectedGroup.entries.length - 1}
            >
              {t("skillMonitor.common.moveDown")}
            </button>
            <button
              type="button"
              class="min-h-11 rounded border border-border/60 px-3 py-1 text-xs text-destructive transition-colors hover:bg-destructive/10 cursor-pointer"
              onclick={() => removeCustomPanelEntry(selectedGroup.id, entry.id)}
            >
              {t("skillMonitor.common.delete")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="rounded-lg border border-border/60 bg-card/40 p-6 text-sm text-muted-foreground shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      {t("skillMonitor.customPanel.empty")}
    </div>
  {/if}

  <div class="rounded-lg border border-border/60 bg-card/40 p-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)] space-y-4">
    <div>
      <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.customPanel.style.title")}</h2>
      <p class="text-xs text-muted-foreground">{t("skillMonitor.customPanel.style.description")}</p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
      <label class="text-xs text-muted-foreground">
        {t("skillMonitor.style.gap", { value: customPanelStyle.gap })}
        <input
          class="mt-1 w-full"
          type="range"
          min="0"
          max="24"
          step="1"
          value={customPanelStyle.gap}
          oninput={(event) => setCustomPanelGap(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
      <label class="text-xs text-muted-foreground">
        {t("skillMonitor.style.fontSize", { value: customPanelStyle.fontSize })}
        <input
          class="mt-1 w-full"
          type="range"
          min="10"
          max="28"
          step="1"
          value={customPanelStyle.fontSize}
          oninput={(event) => setCustomPanelFontSize(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
      <label class="text-xs text-muted-foreground">
        {t("skillMonitor.style.columnGap", { value: customPanelStyle.columnGap })}
        <input
          class="mt-1 w-full"
          type="range"
          min="0"
          max="240"
          step="1"
          value={customPanelStyle.columnGap}
          oninput={(event) => setCustomPanelColumnGap(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.nameColor")}
        <input
          type="color"
          value={customPanelStyle.nameColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setCustomPanelNameColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.valueColor")}
        <input
          type="color"
          value={customPanelStyle.valueColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setCustomPanelValueColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.progressColor")}
        <input
          type="color"
          value={customPanelStyle.progressColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setCustomPanelProgressColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        <div>{t("skillMonitor.style.progressOpacity", { value: Math.round(customPanelStyle.progressOpacity * 100) })}</div>
        <input
          class="mt-2 w-full"
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={customPanelStyle.progressOpacity}
          oninput={(event) =>
            setCustomPanelProgressOpacity(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
    </div>
  </div>
</div>
