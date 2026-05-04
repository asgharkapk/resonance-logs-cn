<script lang="ts">
  import ChevronDown from "virtual:icons/lucide/chevron-down";
  import BuffSearchResultGrid from "$lib/components/BuffSearchResultGrid.svelte";
  import type {
    BuffCategoryDefinition,
    BuffCategoryKey,
    BuffDefinition,
    BuffNameInfo,
  } from "$lib/config/buff-name-table";
  import type {
    BuffDisplayMode,
    BuffGroup,
    TextBuffPanelDisplayMode,
    TextBuffPanelStyle,
  } from "$lib/settings-store";
  import { t } from "$lib/i18n/index.svelte";

  type BuffGroupUpdateHandler = (updater: (curr: BuffGroup) => BuffGroup) => void;

  interface Props {
    buffSearch: string;
    filteredBuffs: BuffNameInfo[];
    monitoredBuffIds: number[];
    monitoredBuffCategories: BuffCategoryKey[];
    expandedSelectedBuffIds: number[];
    selectedBuffs: BuffDefinition[];
    selectedBuffCategories: BuffCategoryDefinition[];
    availableBuffs: BuffDefinition[];
    buffCategoryDefinitions: BuffCategoryDefinition[];
    availableBuffMap: Map<number, BuffDefinition>;
    buffAliasSectionExpanded: boolean;
    setBuffAliasSectionExpanded: (expanded: boolean) => void;
    buffAliasSearch: string;
    setBuffAliasSearch: (value: string) => void;
    buffAliasSearchResults: BuffNameInfo[];
    buffAliasEditingBuffId: number | null;
    setBuffAliasEditingBuffId: (buffId: number | null) => void;
    configuredBuffAliasIds: number[];
    getBuffDisplayName: (buffId: number) => string;
    getBuffDefaultName: (buffId: number) => string;
    getBuffAlias: (buffId: number) => string;
    setBuffAlias: (buffId: number, alias: string) => void;
    resetBuffAlias: (buffId: number) => void;
    isBuffSelected: (buffId: number) => boolean;
    isBuffCategorySelected: (categoryKey: BuffCategoryKey) => boolean;
    toggleBuff: (buffId: number) => void;
    toggleBuffCategory: (categoryKey: BuffCategoryKey) => void;
    clearBuffs: () => void;
    setBuffSearch: (value: string) => void;

    buffDisplayMode: BuffDisplayMode;
    setBuffDisplayMode: (mode: BuffDisplayMode) => void;
    textBuffMaxVisible: number;
    setTextBuffMaxVisible: (value: number) => void;
    textBuffPanelStyle: TextBuffPanelStyle;
    setTextBuffPanelDisplayMode: (value: TextBuffPanelDisplayMode) => void;
    setTextBuffPanelGap: (value: number) => void;
    setTextBuffPanelFontSize: (value: number) => void;
    setTextBuffPanelColumnGap: (value: number) => void;
    setTextBuffPanelNameColor: (value: string) => void;
    setTextBuffPanelValueColor: (value: string) => void;
    setTextBuffPanelProgressColor: (value: string) => void;
    setTextBuffPanelProgressOpacity: (value: number) => void;

    globalPrioritySearch: string;
    globalPrioritySearchResults: BuffNameInfo[];
    setGlobalPrioritySearch: (value: string) => void;
    buffPriorityIds: number[];
    toggleGlobalPriority: (buffId: number) => void;
    moveGlobalPriority: (buffId: number, direction: "up" | "down") => void;

    individualMonitorAllGroup: BuffGroup | null;
    addIndividualMonitorAll: () => void;
    removeIndividualMonitorAll: () => void;
    updateIndividualMonitorAllGroup: (updater: (group: BuffGroup) => BuffGroup) => void;

    buffGroups: BuffGroup[];
    addBuffGroup: () => void;
    removeBuffGroup: (groupId: string) => void;
    updateBuffGroup: (groupId: string, updater: (group: BuffGroup) => BuffGroup) => void;
    getGroupSearchKeyword: (groupId: string) => string;
    setGroupSearchKeyword: (groupId: string, value: string) => void;
    getGroupSearchResults: (group: BuffGroup) => BuffNameInfo[];
    getGroupPrioritySearchKeyword: (groupId: string) => string;
    setGroupPrioritySearchKeyword: (groupId: string, value: string) => void;
    getGroupPrioritySearchResults: (group: BuffGroup) => BuffNameInfo[];
    getGroupPriorityIds: (group: BuffGroup) => number[];
    toggleBuffCategoryInGroup: (groupId: string, categoryKey: BuffCategoryKey) => void;
    hasCompleteBuffCategoryInGroup: (group: BuffGroup, categoryKey: BuffCategoryKey) => boolean;
    toggleBuffInGroup: (groupId: string, buffId: number) => void;
    togglePriorityInGroup: (groupId: string, buffId: number) => void;
    moveGroupPriority: (groupId: string, buffId: number, direction: "up" | "down") => void;
  }

  let {
    buffSearch,
    filteredBuffs,
    monitoredBuffIds,
    monitoredBuffCategories,
    expandedSelectedBuffIds,
    selectedBuffs,
    selectedBuffCategories,
    availableBuffs,
    buffCategoryDefinitions,
    availableBuffMap,
    buffAliasSectionExpanded,
    setBuffAliasSectionExpanded,
    buffAliasSearch,
    setBuffAliasSearch,
    buffAliasSearchResults,
    buffAliasEditingBuffId,
    setBuffAliasEditingBuffId,
    configuredBuffAliasIds,
    getBuffDisplayName,
    getBuffDefaultName,
    getBuffAlias,
    setBuffAlias,
    resetBuffAlias,
    isBuffSelected,
    isBuffCategorySelected,
    toggleBuff,
    toggleBuffCategory,
    clearBuffs,
    setBuffSearch,
    buffDisplayMode,
    setBuffDisplayMode,
    textBuffMaxVisible,
    setTextBuffMaxVisible,
    textBuffPanelStyle,
    setTextBuffPanelDisplayMode,
    setTextBuffPanelGap,
    setTextBuffPanelFontSize,
    setTextBuffPanelColumnGap,
    setTextBuffPanelNameColor,
    setTextBuffPanelValueColor,
    setTextBuffPanelProgressColor,
    setTextBuffPanelProgressOpacity,
    globalPrioritySearch,
    globalPrioritySearchResults,
    setGlobalPrioritySearch,
    buffPriorityIds,
    toggleGlobalPriority,
    moveGlobalPriority,
    individualMonitorAllGroup,
    addIndividualMonitorAll,
    removeIndividualMonitorAll,
    updateIndividualMonitorAllGroup,
    buffGroups,
    addBuffGroup,
    removeBuffGroup,
    updateBuffGroup,
    getGroupSearchKeyword,
    setGroupSearchKeyword,
    getGroupSearchResults,
    getGroupPrioritySearchKeyword,
    setGroupPrioritySearchKeyword,
    getGroupPrioritySearchResults,
    getGroupPriorityIds,
    toggleBuffCategoryInGroup,
    hasCompleteBuffCategoryInGroup,
    toggleBuffInGroup,
    togglePriorityInGroup,
    moveGroupPriority,
  }: Props = $props();

  function buffSearchStatusLabel(buffId: number): string | null {
    return isBuffSelected(buffId) ? t("skillMonitor.buff.status.selected") : null;
  }

  function buffAliasStatusLabel(buffId: number): string | null {
    if (buffAliasEditingBuffId === buffId) return t("skillMonitor.buff.status.editing");
    return configuredBuffAliasIds.includes(buffId)
      ? t("skillMonitor.buff.status.aliased")
      : null;
  }

  function getBuffGroupDisplayName(group: BuffGroup, index: number): string {
    const trimmedName = group.name.trim();
    if (trimmedName) return trimmedName;
    return group.monitorAll
      ? t("skillMonitor.defaults.allBuffGroupName")
      : t("skillMonitor.defaults.buffGroupName", { index: index + 1 });
  }

  function getIndividualAllGroupDisplayName(group: BuffGroup): string {
    return group.name.trim() || t("skillMonitor.defaults.allBuffGroupName");
  }

  function getFilteredGlobalPrioritySearchResults(): BuffNameInfo[] {
    const ids = new Set<number>();
    return globalPrioritySearchResults.filter((item) => {
      if (ids.has(item.baseId)) return false;
      if (!expandedSelectedBuffIds.includes(item.baseId)) return false;
      if (buffPriorityIds.includes(item.baseId)) return false;
      ids.add(item.baseId);
      return true;
    });
  }
</script>

{#snippet buffGroupLayoutControls(group: BuffGroup, onUpdate: BuffGroupUpdateHandler)}
  <div class="grid grid-cols-2 gap-3">
    <label class="text-xs text-muted-foreground">
      {t("skillMonitor.layoutControls.iconSize", { value: group.iconSize })}
      <input
        class="w-full mt-1"
        type="range"
        min="24"
        max="120"
        step="1"
        value={group.iconSize}
        oninput={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            iconSize: Number((event.currentTarget as HTMLInputElement).value),
          }))}
      />
    </label>
    <label class="text-xs text-muted-foreground">
      {t("skillMonitor.layoutControls.columns", { value: group.columns })}
      <input
        class="w-full mt-1"
        type="range"
        min="1"
        max="12"
        step="1"
        value={group.columns}
        oninput={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            columns: Number((event.currentTarget as HTMLInputElement).value),
          }))}
      />
    </label>
    <label class="text-xs text-muted-foreground">
      {t("skillMonitor.layoutControls.rows", { value: group.rows })}
      <input
        class="w-full mt-1"
        type="range"
        min="1"
        max="12"
        step="1"
        value={group.rows}
        oninput={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            rows: Number((event.currentTarget as HTMLInputElement).value),
          }))}
      />
    </label>
    <label class="text-xs text-muted-foreground">
      {t("skillMonitor.layoutControls.gap", { value: group.gap })}
      <input
        class="w-full mt-1"
        type="range"
        min="0"
        max="16"
        step="1"
        value={group.gap}
        oninput={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            gap: Number((event.currentTarget as HTMLInputElement).value),
          }))}
      />
    </label>
  </div>
  <div class="flex flex-wrap gap-3">
    <label class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-foreground">
      <input
        type="checkbox"
        checked={group.showName}
        onchange={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            showName: (event.currentTarget as HTMLInputElement).checked,
          }))}
      />
      {t("skillMonitor.layoutControls.showName")}
    </label>
    <label class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-foreground">
      <input
        type="checkbox"
        checked={group.showTime}
        onchange={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            showTime: (event.currentTarget as HTMLInputElement).checked,
          }))}
      />
      {t("skillMonitor.layoutControls.showTime")}
    </label>
    <label class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-foreground">
      <input
        type="checkbox"
        checked={group.showLayer}
        onchange={(event) =>
          onUpdate((curr: BuffGroup) => ({
            ...curr,
            showLayer: (event.currentTarget as HTMLInputElement).checked,
          }))}
      />
      {t("skillMonitor.layoutControls.showLayer")}
    </label>
  </div>
{/snippet}

<div class="space-y-6">
  <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.title")}</h2>
        <p class="text-xs text-muted-foreground">{t("skillMonitor.buff.description")}</p>
      </div>
      <div class="flex items-center gap-3">
        <div class="text-xs text-muted-foreground">
          {t("skillMonitor.buff.selectedSummary", {
            buffCount: monitoredBuffIds.length,
            categoryCount: monitoredBuffCategories.length,
          })}
        </div>
        <button
          type="button"
          class="text-xs px-2 py-1 rounded border border-border/60 text-muted-foreground hover:text-foreground hover:bg-muted/40 transition-colors"
          onclick={clearBuffs}
        >
          {t("skillMonitor.common.clear")}
        </button>
      </div>
    </div>

    <input
      class="w-full sm:w-64 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
      placeholder={t("skillMonitor.buff.placeholder")}
      value={buffSearch}
      oninput={(event) => setBuffSearch((event.currentTarget as HTMLInputElement).value)}
    />

    {#if buffSearch.trim().length > 0}
      <BuffSearchResultGrid
        items={filteredBuffs}
        {availableBuffMap}
        onSelect={toggleBuff}
        isSelected={isBuffSelected}
        getStatusLabel={buffSearchStatusLabel}
        emptyMessage={t("components.buffSearchResultGrid.empty")}
      />
    {:else}
      <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.searchPrompt")}</div>
    {/if}

    <div class="space-y-2">
      <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.selectedTitle")}</div>
      <div class="flex flex-wrap gap-2">
        {#each monitoredBuffIds as buffId (buffId)}
          {@const iconBuff = selectedBuffs.find((buff) => buff.baseId === buffId)}
          {#if iconBuff}
            <button
              type="button"
              class="relative rounded-md border border-border/60 overflow-hidden bg-muted/20 size-12 hover:border-border hover:bg-muted/30"
              title={getBuffDisplayName(buffId)}
              onclick={() => toggleBuff(iconBuff.baseId)}
            >
              <img
                src={`/images/buff/${iconBuff.spriteFile}`}
                alt={getBuffDisplayName(buffId)}
                class="w-full h-full object-contain"
              />
            </button>
          {:else}
            <button
              type="button"
              class="rounded-md border border-border/60 bg-muted/20 px-2 py-1 text-[11px] text-foreground hover:border-border hover:bg-muted/30"
              title={getBuffDisplayName(buffId)}
              onclick={() => toggleBuff(buffId)}
            >
              {getBuffDisplayName(buffId)}
            </button>
          {/if}
        {/each}
      </div>
    </div>

  </div>

  <div class="rounded-lg border border-border/60 bg-card/40 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
    <button
      type="button"
      class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
      onclick={() => setBuffAliasSectionExpanded(!buffAliasSectionExpanded)}
    >
      <div class="text-left">
        <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.alias.title")}</h2>
      </div>
      <ChevronDown
        class="w-5 h-5 text-muted-foreground transition-transform duration-200 {buffAliasSectionExpanded
          ? 'rotate-180'
          : ''}"
      />
    </button>

    {#if buffAliasSectionExpanded}
      <div class="px-4 pb-4 space-y-4">
        <div class="space-y-2">
          <div class="text-xs text-muted-foreground">
            {t("skillMonitor.buff.alias.configuredCount", { count: configuredBuffAliasIds.length })}
          </div>
          <input
            class="w-full sm:w-80 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
            placeholder={t("skillMonitor.buff.alias.placeholder")}
            value={buffAliasSearch}
            oninput={(event) =>
              setBuffAliasSearch((event.currentTarget as HTMLInputElement).value)}
          />
        </div>

        {#if buffAliasSearch.trim().length > 0}
          <div class="space-y-2">
            <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.alias.searchResults")}</div>
            <BuffSearchResultGrid
              items={buffAliasSearchResults}
              {availableBuffMap}
              onSelect={(buffId) => setBuffAliasEditingBuffId(buffId)}
              isSelected={(buffId) => buffAliasEditingBuffId === buffId}
              getStatusLabel={buffAliasStatusLabel}
              emptyMessage={t("components.buffSearchResultGrid.empty")}
            />

            {#if buffAliasEditingBuffId !== null}
              <div class="rounded-md border border-border/60 bg-muted/20 p-3 space-y-2">
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <div class="text-sm text-foreground truncate">
                      {getBuffDisplayName(buffAliasEditingBuffId)}
                    </div>
                    <div class="text-xs text-muted-foreground truncate">
                      {t("skillMonitor.buff.alias.defaultWithId", {
                        name: getBuffDefaultName(buffAliasEditingBuffId),
                        id: buffAliasEditingBuffId,
                      })}
                    </div>
                  </div>
                  <button
                    type="button"
                    class="text-xs px-2 py-1 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50"
                    onclick={() => resetBuffAlias(buffAliasEditingBuffId)}
                    disabled={!getBuffAlias(buffAliasEditingBuffId)}
                  >
                    {t("skillMonitor.buff.alias.reset")}
                  </button>
                </div>
                <input
                  class="w-full rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                  placeholder={getBuffDefaultName(buffAliasEditingBuffId)}
                  value={getBuffAlias(buffAliasEditingBuffId)}
                  oninput={(event) =>
                    setBuffAlias(buffAliasEditingBuffId, (event.currentTarget as HTMLInputElement).value)}
                />
              </div>
            {/if}
          </div>
        {:else if configuredBuffAliasIds.length > 0}
          <div class="space-y-2">
            <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.alias.configuredTitle")}</div>
            <div class="space-y-2">
              {#each configuredBuffAliasIds as buffId (buffId)}
                {@const iconBuff = availableBuffMap.get(buffId)}
                <div class="rounded-md border border-border/60 bg-muted/20 p-3 space-y-2">
                  <div class="flex items-center gap-3">
                    {#if iconBuff}
                      <img
                        src={`/images/buff/${iconBuff.spriteFile}`}
                        alt={getBuffDisplayName(buffId)}
                        class="size-10 rounded border border-border/40 bg-muted/20 object-contain"
                      />
                    {:else}
                      <div class="size-10 rounded border border-border/40 bg-muted/20 flex items-center justify-center text-[10px] text-muted-foreground">
                        {t("components.buffSearchResultGrid.fallbackIcon")}
                      </div>
                    {/if}
                    <div class="min-w-0 flex-1">
                      <div class="text-sm text-foreground truncate">{getBuffDisplayName(buffId)}</div>
                      <div class="text-xs text-muted-foreground truncate">
                        {t("skillMonitor.buff.alias.defaultWithId", {
                          name: getBuffDefaultName(buffId),
                          id: buffId,
                        })}
                      </div>
                    </div>
                    <button
                      type="button"
                      class="text-xs px-2 py-1 rounded border border-border/60 hover:bg-muted/40"
                      onclick={() => resetBuffAlias(buffId)}
                    >
                      {t("skillMonitor.buff.alias.reset")}
                    </button>
                  </div>
                  <input
                    class="w-full rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                    placeholder={getBuffDefaultName(buffId)}
                    value={getBuffAlias(buffId)}
                    oninput={(event) =>
                      setBuffAlias(buffId, (event.currentTarget as HTMLInputElement).value)}
                  />
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="text-xs text-muted-foreground">
            {t("skillMonitor.buff.alias.empty")}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
    <div>
      <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.display.title")}</h2>
      <p class="text-xs text-muted-foreground">{t("skillMonitor.buff.display.description")}</p>
    </div>
    <div class="flex flex-wrap gap-2">
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {buffDisplayMode === 'individual'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => setBuffDisplayMode("individual")}
      >
        {t("skillMonitor.buff.display.individual")}
      </button>
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {buffDisplayMode === 'grouped'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => setBuffDisplayMode("grouped")}
      >
        {t("skillMonitor.buff.display.grouped")}
      </button>
    </div>
    <div class="flex flex-wrap gap-2">
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {textBuffPanelStyle.displayMode === 'modern'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => setTextBuffPanelDisplayMode("modern")}
      >
        {t("skillMonitor.buff.display.modernText")}
      </button>
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {textBuffPanelStyle.displayMode === 'classic'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => setTextBuffPanelDisplayMode("classic")}
      >
        {t("skillMonitor.buff.display.classicText")}
      </button>
    </div>
    <label class="block text-xs text-muted-foreground max-w-md">
      {t("skillMonitor.buff.display.maxTextBuffs", { count: textBuffMaxVisible })}
      <input
        class="w-full mt-1"
        type="range"
        min="1"
        max="20"
        step="1"
        value={textBuffMaxVisible}
        oninput={(event) => setTextBuffMaxVisible(Number((event.currentTarget as HTMLInputElement).value))}
      />
    </label>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 max-w-4xl">
      <label class="text-xs text-muted-foreground">
        {t("skillMonitor.style.gap", { value: textBuffPanelStyle.gap })}
        <input
          class="w-full mt-1"
          type="range"
          min="0"
          max="24"
          step="1"
          value={textBuffPanelStyle.gap}
          oninput={(event) => setTextBuffPanelGap(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
      <label class="text-xs text-muted-foreground">
        {t("skillMonitor.style.fontSize", { value: textBuffPanelStyle.fontSize })}
        <input
          class="w-full mt-1"
          type="range"
          min="10"
          max="28"
          step="1"
          value={textBuffPanelStyle.fontSize}
          oninput={(event) => setTextBuffPanelFontSize(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
      {#if textBuffPanelStyle.displayMode === "modern"}
        <label class="text-xs text-muted-foreground">
          {t("skillMonitor.style.columnGap", { value: textBuffPanelStyle.columnGap })}
          <input
            class="w-full mt-1"
            type="range"
            min="0"
            max="240"
            step="1"
            value={textBuffPanelStyle.columnGap}
            oninput={(event) =>
              setTextBuffPanelColumnGap(Number((event.currentTarget as HTMLInputElement).value))}
          />
        </label>
      {/if}
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 max-w-4xl">
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.nameColor")}
        <input
          type="color"
          value={textBuffPanelStyle.nameColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setTextBuffPanelNameColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.valueColor")}
        <input
          type="color"
          value={textBuffPanelStyle.valueColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setTextBuffPanelValueColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="flex items-center justify-between gap-2 rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        {t("skillMonitor.style.progressColor")}
        <input
          type="color"
          value={textBuffPanelStyle.progressColor}
          class="h-7 w-12 rounded border border-border/60 bg-transparent p-0"
          onchange={(event) => setTextBuffPanelProgressColor((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label class="rounded border border-border/60 bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
        <div>{t("skillMonitor.style.progressOpacity", { value: Math.round(textBuffPanelStyle.progressOpacity * 100) })}</div>
        <input
          class="mt-2 w-full"
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={textBuffPanelStyle.progressOpacity}
          oninput={(event) =>
            setTextBuffPanelProgressOpacity(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>
    </div>
  </div>

  <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
    <div class="space-y-2">
      <div class="text-xs font-medium text-foreground">{t("skillMonitor.buff.priority.globalTitle")}</div>
      <input
        class="w-full sm:w-72 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
        placeholder={t("skillMonitor.buff.priority.globalPlaceholder")}
        value={globalPrioritySearch}
        oninput={(event) => setGlobalPrioritySearch((event.currentTarget as HTMLInputElement).value)}
      />
      {#if globalPrioritySearch.trim().length > 0}
        <BuffSearchResultGrid
          items={getFilteredGlobalPrioritySearchResults()}
          {availableBuffMap}
          onSelect={toggleGlobalPriority}
          emptyMessage={t("skillMonitor.buff.priority.globalEmpty")}
          minColumnWidth={180}
        />
      {/if}
      <div class="space-y-1">
        {#each buffPriorityIds as buffId, idx (buffId)}
          <div class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-2 py-1">
            <span class="w-6 text-center text-xs text-muted-foreground">{idx + 1}</span>
            <span class="flex-1 text-xs text-foreground truncate">
              {getBuffDisplayName(buffId)}
            </span>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40" onclick={() => toggleGlobalPriority(buffId)}>{t("skillMonitor.common.remove")}</button>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveGlobalPriority(buffId, "up")} disabled={idx === 0}>{t("skillMonitor.common.moveUp")}</button>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveGlobalPriority(buffId, "down")} disabled={idx === buffPriorityIds.length - 1}>{t("skillMonitor.common.moveDown")}</button>
          </div>
        {/each}
      </div>
    </div>
  </div>

  {#if buffDisplayMode === "individual"}
    <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <div>
          <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.category.title")}</h2>
      </div>
      <div class="flex flex-wrap gap-2">
        {#each buffCategoryDefinitions as category (category.key)}
          <button
            type="button"
            class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {isBuffCategorySelected(category.key)
              ? 'bg-primary text-primary-foreground border-primary'
              : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
            onclick={() => toggleBuffCategory(category.key)}
          >
            {category.label} ({category.count})
          </button>
        {/each}
      </div>
      <div class="space-y-2">
        <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.category.selectedTitle")}</div>
        <div class="flex flex-wrap gap-2">
          {#if selectedBuffCategories.length > 0}
            {#each selectedBuffCategories as category (category.key)}
              <button
                type="button"
                class="rounded-md border border-primary/60 bg-primary/10 px-3 py-1.5 text-xs text-foreground hover:bg-primary/15"
                onclick={() => toggleBuffCategory(category.key)}
              >
                {category.label} ({category.count})
              </button>
            {/each}
          {:else}
            <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.category.empty")}</div>
          {/if}
        </div>
      </div>
    </div>

    <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.all.title")}</h2>
          <p class="text-xs text-muted-foreground">
            {t("skillMonitor.buff.all.description")}
          </p>
        </div>
        {#if !individualMonitorAllGroup}
          <button type="button" class="text-xs px-3 py-2 rounded border border-border/60 text-foreground hover:bg-muted/40 transition-colors" onclick={addIndividualMonitorAll}>
            {t("skillMonitor.buff.all.add")}
          </button>
        {:else}
          <button type="button" class="text-xs px-3 py-2 rounded border border-border/60 text-destructive hover:bg-destructive/10 transition-colors" onclick={removeIndividualMonitorAll}>
            {t("skillMonitor.buff.all.remove")}
          </button>
        {/if}
      </div>
      {#if individualMonitorAllGroup}
        <div class="rounded-lg border border-border/60 bg-muted/20 p-3 space-y-3">
          <div class="flex flex-wrap items-center gap-2">
            <input
              class="w-52 rounded border border-border/60 bg-muted/30 px-2 py-1.5 text-sm text-foreground"
              value={individualMonitorAllGroup.name}
              placeholder={getIndividualAllGroupDisplayName(individualMonitorAllGroup)}
              oninput={(event) =>
                updateIndividualMonitorAllGroup((curr) => ({
                  ...curr,
                  name: (event.currentTarget as HTMLInputElement).value || curr.name,
                }))}
            />
            <span class="text-xs text-muted-foreground">{t("skillMonitor.buff.all.fixed")}</span>
          </div>
          {@render buffGroupLayoutControls(individualMonitorAllGroup, updateIndividualMonitorAllGroup)}
        </div>
      {/if}
    </div>
  {/if}

  {#if buffDisplayMode === "grouped"}
    <div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-4 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h2 class="text-base font-semibold text-foreground">{t("skillMonitor.buff.group.title")}</h2>
          <p class="text-xs text-muted-foreground">
            {t("skillMonitor.buff.group.description")}
          </p>
        </div>
        <button
          type="button"
          class="text-xs px-3 py-2 rounded border border-border/60 text-foreground hover:bg-muted/40 transition-colors"
          onclick={addBuffGroup}
        >
          {t("skillMonitor.buff.group.new")}
        </button>
      </div>

      <div class="space-y-3">
        {#each buffGroups as group, groupIndex (group.id)}
          <div class="rounded-lg border border-border/60 bg-muted/20 p-3 space-y-3">
            <div class="flex flex-wrap items-center gap-2">
              <input
                class="w-52 rounded border border-border/60 bg-muted/30 px-2 py-1.5 text-sm text-foreground"
                value={group.name}
                placeholder={getBuffGroupDisplayName(group, groupIndex)}
                oninput={(event) =>
                  updateBuffGroup(group.id, (curr) => ({
                    ...curr,
                    name: (event.currentTarget as HTMLInputElement).value || curr.name,
                  }))}
              />
              <button
                type="button"
                class="text-xs px-2 py-1 rounded border border-border/60 text-destructive hover:bg-destructive/10 transition-colors"
                onclick={() => removeBuffGroup(group.id)}
              >
                {t("skillMonitor.buff.group.delete")}
              </button>
              <label class="ml-auto flex items-center gap-2 text-xs text-foreground">
                <input
                  type="checkbox"
                  checked={group.monitorAll}
                  onchange={(event) =>
                    updateBuffGroup(group.id, (curr) => ({
                      ...curr,
                      monitorAll: (event.currentTarget as HTMLInputElement).checked,
                    }))}
                />
                {t("skillMonitor.buff.group.monitorAll")}
              </label>
            </div>

            <div class="space-y-2">
              <div class="flex flex-wrap gap-2">
                {#each buffCategoryDefinitions as category (category.key)}
                  <button
                    type="button"
                    class="rounded-md border px-3 py-1.5 text-xs transition-colors {hasCompleteBuffCategoryInGroup(group, category.key)
                      ? 'border-primary/60 bg-primary/10 text-foreground'
                      : 'border-border/60 bg-muted/20 text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
                    onclick={() => toggleBuffCategoryInGroup(group.id, category.key)}
                    disabled={group.monitorAll}
                  >
                    {hasCompleteBuffCategoryInGroup(group, category.key)
                      ? t("skillMonitor.buff.group.removeCategory", {
                        name: category.label,
                        count: category.count,
                      })
                      : t("skillMonitor.buff.group.addCategory", {
                        name: category.label,
                        count: category.count,
                      })}
                  </button>
                {/each}
              </div>
              <input
                class="w-full sm:w-72 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                placeholder={t("skillMonitor.buff.group.placeholder")}
                value={getGroupSearchKeyword(group.id)}
                oninput={(event) =>
                  setGroupSearchKeyword(group.id, (event.currentTarget as HTMLInputElement).value)}
              />
              {#if getGroupSearchResults(group).length > 0}
                <BuffSearchResultGrid
                  items={getGroupSearchResults(group)}
                  {availableBuffMap}
                  onSelect={(buffId) => toggleBuffInGroup(group.id, buffId)}
                  emptyMessage={t("skillMonitor.buff.group.emptySearch")}
                  minColumnWidth={180}
                />
              {/if}

              {#if !group.monitorAll}
                <div class="space-y-2">
                  <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.group.joinedCount", { count: group.buffIds.length })}</div>
                  <div class="flex flex-wrap gap-2">
                    {#if group.buffIds.length > 0}
                      {#each group.buffIds as buffId (buffId)}
                        {@const selectedBuff = availableBuffMap.get(buffId)}
                        {#if selectedBuff}
                          <button
                            type="button"
                            class="relative rounded-md border border-border/60 overflow-hidden bg-muted/20 size-12 hover:border-border hover:bg-muted/30"
                            title={t("skillMonitor.buff.group.removeTitle", {
                              name: getBuffDisplayName(buffId),
                            })}
                            onclick={() => toggleBuffInGroup(group.id, buffId)}
                          >
                            <img
                              src={`/images/buff/${selectedBuff.spriteFile}`}
                              alt={getBuffDisplayName(buffId)}
                              class="w-full h-full object-contain"
                            />
                          </button>
                        {:else}
                          <button
                            type="button"
                            class="rounded-md border border-border/60 bg-muted/20 px-2 py-1 text-[11px] text-foreground hover:border-border hover:bg-muted/30"
                            title={t("skillMonitor.buff.group.removeTitle", {
                              name: getBuffDisplayName(buffId),
                            })}
                            onclick={() => toggleBuffInGroup(group.id, buffId)}
                          >
                            {getBuffDisplayName(buffId)}
                          </button>
                        {/if}
                      {/each}
                    {:else}
                      <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.group.empty")}</div>
                    {/if}
                  </div>
                </div>
              {/if}

              <div class="space-y-1">
                <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.priority.groupTitle")}</div>
                <input
                  class="w-full sm:w-72 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                  placeholder={t("skillMonitor.buff.priority.groupPlaceholder")}
                  value={getGroupPrioritySearchKeyword(group.id)}
                  oninput={(event) =>
                    setGroupPrioritySearchKeyword(group.id, (event.currentTarget as HTMLInputElement).value)}
                />
                {#if getGroupPrioritySearchResults(group).length > 0}
                  <BuffSearchResultGrid
                    items={getGroupPrioritySearchResults(group)}
                    {availableBuffMap}
                    onSelect={(buffId) => togglePriorityInGroup(group.id, buffId)}
                    emptyMessage={t("skillMonitor.buff.priority.groupEmpty")}
                    minColumnWidth={180}
                  />
                {/if}
                {#each getGroupPriorityIds(group) as buffId, idx (buffId)}
                  <div class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-2 py-1">
                    <span class="w-6 text-center text-xs text-muted-foreground">{idx + 1}</span>
                    <span class="flex-1 text-xs text-foreground truncate">
                      {getBuffDisplayName(buffId)}
                    </span>
                    <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40" onclick={() => togglePriorityInGroup(group.id, buffId)}>{t("skillMonitor.common.remove")}</button>
                    <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveGroupPriority(group.id, buffId, "up")} disabled={idx === 0}>{t("skillMonitor.common.moveUp")}</button>
                    <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveGroupPriority(group.id, buffId, "down")} disabled={idx === getGroupPriorityIds(group).length - 1}>{t("skillMonitor.common.moveDown")}</button>
                  </div>
                {/each}
              </div>
              <div class="space-y-2">
                <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.group.layoutTitle")}</div>
                {@render buffGroupLayoutControls(group, (updater: (curr: BuffGroup) => BuffGroup) => updateBuffGroup(group.id, updater))}
              </div>
            </div>
          </div>
        {/each}
      </div>

      <div class="rounded-md border border-border/60 bg-muted/20 p-3 space-y-2">
        <div class="text-xs text-muted-foreground">{t("skillMonitor.buff.group.previewTitle")}</div>
        <div class="space-y-2">
          {#each buffGroups as group, groupIndex (group.id)}
            <div class="rounded border border-border/50 p-2">
              <div class="text-xs mb-2 text-foreground">
                {getBuffGroupDisplayName(group, groupIndex)}{group.monitorAll
                  ? t("skillMonitor.buff.group.allSuffix")
                  : ""}
              </div>
              <div
                class="grid"
                style:grid-template-columns={`repeat(${Math.max(1, group.columns)}, minmax(0, ${group.iconSize / 2}px))`}
                style:gap={`${Math.max(0, group.gap / 2)}px`}
              >
                {#if group.monitorAll}
                  {#each availableBuffs.slice(0, Math.max(6, group.columns * group.rows)) as buff (buff.baseId)}
                    <img src={`/images/buff/${buff.spriteFile}`} alt={buff.name} class="w-full aspect-square object-contain rounded border border-border/30 bg-muted/20" />
                  {/each}
                {:else}
                  {#each group.buffIds.slice(0, Math.max(6, group.columns * group.rows)) as buffId (buffId)}
                    {@const buff = availableBuffMap.get(buffId)}
                    {#if buff}
                      <img src={`/images/buff/${buff.spriteFile}`} alt={buff.name} class="w-full aspect-square object-contain rounded border border-border/30 bg-muted/20" />
                    {:else}
                      <div class="w-full aspect-square rounded border border-border/30 bg-muted/20"></div>
                    {/if}
                  {/each}
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
