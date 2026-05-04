<script lang="ts">
  import SettingsSwitch from "../dps/settings/settings-switch.svelte";
  import BuffSearchResultGrid from "$lib/components/BuffSearchResultGrid.svelte";
  import {
    getAvailableBuffDefinitions,
    lookupDefaultBuffName,
    resolveBuffDisplayName,
    searchBuffsByName,
    type BuffDefinition,
    type BuffNameInfo,
  } from "$lib/config/buff-name-table";
  import { t } from "$lib/i18n/index.svelte";
  import { SETTINGS, ensureBuffAliases } from "$lib/settings-store";

  type SearchTarget = "global" | "self";
  type MonsterMonitorTab = "buff" | "hate";

  const availableBuffDefinitions = getAvailableBuffDefinitions();
  const availableBuffMap = new Map<number, BuffDefinition>(
    availableBuffDefinitions.map((buff) => [buff.baseId, buff]),
  );

  let searchKeyword = $state("");
  let prioritySearchKeyword = $state("");
  let searchTarget = $state<SearchTarget>("self");
  let activeTab = $state<MonsterMonitorTab>("buff");

  const monsterMonitor = $derived(SETTINGS.monsterMonitor.state);
  const buffAliases = $derived.by(() =>
    ensureBuffAliases(monsterMonitor.buffAliases),
  );
  const hatePanelStyle = $derived.by(() =>
    monsterMonitor.hatePanelStyle ?? monsterMonitor.panelStyle,
  );
  const globalBuffIds = $derived(monsterMonitor.monitoredBuffIds);
  const selfAppliedBuffIds = $derived(monsterMonitor.selfAppliedBuffIds);
  const buffPriorityIds = $derived(monsterMonitor.buffPriorityIds ?? []);
  const combinedBuffIds = $derived.by(() =>
    Array.from(new Set([...globalBuffIds, ...selfAppliedBuffIds])),
  );
  const searchResults = $derived.by(() =>
    searchKeyword.trim().length > 0
      ? searchBuffsByName(searchKeyword, buffAliases)
      : ([] as BuffNameInfo[]),
  );
  const prioritySearchResults = $derived.by(() => {
    if (prioritySearchKeyword.trim().length === 0) return [];
    
    const matching = searchBuffsByName(prioritySearchKeyword, buffAliases);
    const combinedSet = new Set(combinedBuffIds);
    const prioritySet = new Set(buffPriorityIds);
    
    return matching.filter((item) => 
      combinedSet.has(item.baseId) && !prioritySet.has(item.baseId)
    );
  });

  function updateMonsterMonitor(
    updater: (state: typeof SETTINGS.monsterMonitor.state) => Partial<typeof SETTINGS.monsterMonitor.state>,
  ) {
    Object.assign(
      SETTINGS.monsterMonitor.state,
      updater(SETTINGS.monsterMonitor.state),
    );
  }

  function toggleSelectedBuff(buffId: number) {
    updateMonsterMonitor((state) => {
      const nextGlobal = state.monitoredBuffIds.filter((id) => id !== buffId);
      const nextSelf = state.selfAppliedBuffIds.filter((id) => id !== buffId);
      const targetIds = searchTarget === "global" ? nextGlobal : nextSelf;
      const existsInTarget = (searchTarget === "global"
        ? state.monitoredBuffIds
        : state.selfAppliedBuffIds).includes(buffId);
      const nextTargetIds = existsInTarget ? targetIds : [...targetIds, buffId];

      const monitoredBuffIds = searchTarget === "global" ? nextTargetIds : nextGlobal;
      const selfAppliedBuffIds = searchTarget === "self" ? nextTargetIds : nextSelf;
      
      const stillMonitored = monitoredBuffIds.includes(buffId) || selfAppliedBuffIds.includes(buffId);
      const buffPriorityIds = !stillMonitored && state.buffPriorityIds
        ? state.buffPriorityIds.filter(id => id !== buffId)
        : state.buffPriorityIds;

      return {
        ...state,
        monitoredBuffIds,
        selfAppliedBuffIds,
        buffPriorityIds,
      };
    });
  }

  function removeBuff(target: SearchTarget, buffId: number) {
    updateMonsterMonitor((state) => {
      const nextMonitored = target === "global"
        ? state.monitoredBuffIds.filter((id) => id !== buffId)
        : state.monitoredBuffIds;
      const nextSelfApplied = target === "self"
        ? state.selfAppliedBuffIds.filter((id) => id !== buffId)
        : state.selfAppliedBuffIds;
        
      const stillMonitored = nextMonitored.includes(buffId) || nextSelfApplied.includes(buffId);
      const nextPriorityIds = !stillMonitored && state.buffPriorityIds 
        ? state.buffPriorityIds.filter((id) => id !== buffId)
        : state.buffPriorityIds;
        
      return {
        ...state,
        monitoredBuffIds: nextMonitored,
        selfAppliedBuffIds: nextSelfApplied,
        buffPriorityIds: nextPriorityIds,
      };
    });
  }

  function setAlias(buffId: number, alias: string) {
    updateMonsterMonitor((state) => {
      const nextAliases = { ...state.buffAliases };
      const trimmed = alias.trim();
      if (trimmed) {
        nextAliases[buffId] = trimmed;
      } else {
        delete nextAliases[buffId];
      }
      return {
        ...state,
        buffAliases: nextAliases,
      };
    });
  }

  function updatePanelStyle<K extends keyof typeof monsterMonitor.panelStyle>(
    key: K,
    value: (typeof monsterMonitor.panelStyle)[K],
  ) {
    updateMonsterMonitor((state) => ({
      ...state,
      panelStyle: {
        ...state.panelStyle,
        [key]: value,
      },
    }));
  }

  function updateHatePanelStyle<K extends keyof typeof hatePanelStyle>(
    key: K,
    value: (typeof hatePanelStyle)[K],
  ) {
    updateMonsterMonitor((state) => ({
      ...state,
      hatePanelStyle: {
        ...(state.hatePanelStyle ?? state.panelStyle),
        [key]: value,
      },
    }));
  }

  function isSelectedInCurrentTarget(buffId: number) {
    return searchTarget === "global"
      ? globalBuffIds.includes(buffId)
      : selfAppliedBuffIds.includes(buffId);
  }

  function searchStatusLabel(buffId: number): string | null {
    if (searchTarget === "global") {
      if (globalBuffIds.includes(buffId)) return t("monsterMonitor.buffSearch.status.addedGlobal");
      if (selfAppliedBuffIds.includes(buffId)) return t("monsterMonitor.buffSearch.status.currentSelf");
      return null;
    }
    if (selfAppliedBuffIds.includes(buffId)) return t("monsterMonitor.buffSearch.status.addedSelf");
    if (globalBuffIds.includes(buffId)) return t("monsterMonitor.buffSearch.status.currentGlobal");
    return null;
  }

  function buffName(buffId: number) {
    return resolveBuffDisplayName(buffId, buffAliases);
  }

  function defaultBuffName(buffId: number) {
    return lookupDefaultBuffName(buffId) ?? t("monsterMonitor.buffFallback", { id: buffId });
  }

  function toggleMonsterBuffPriority(buffId: number) {
    updateMonsterMonitor((state) => {
      const current = state.buffPriorityIds ?? [];
      const exists = current.includes(buffId);
      return {
        ...state,
        buffPriorityIds: exists 
          ? current.filter((id) => id !== buffId) 
          : [...current, buffId],
      };
    });
  }

  function moveMonsterBuffPriority(buffId: number, direction: "up" | "down") {
    updateMonsterMonitor((state) => {
      const current = state.buffPriorityIds ?? [];
      const idx = current.indexOf(buffId);
      if (idx === -1) return state;
      
      const next = [...current];
      if (direction === "up" && idx > 0) {
        next[idx] = next[idx - 1]!;
        next[idx - 1] = buffId;
      } else if (direction === "down" && idx < next.length - 1) {
        next[idx] = next[idx + 1]!;
        next[idx + 1] = buffId;
      }
      return { ...state, buffPriorityIds: next };
    });
  }
</script>

<div class="space-y-6">
  <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-4">
    <div class="flex justify-start">
      <div class="min-w-[220px]">
        <SettingsSwitch
          label={t("monsterMonitor.enabled")}
          bind:checked={SETTINGS.monsterMonitor.state.enabled}
        />
      </div>
    </div>
  </section>

  <section class="rounded-xl border border-border/60 bg-card/60 p-2">
    <div class="flex flex-wrap gap-2">
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {activeTab === 'buff'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => {
          activeTab = "buff";
        }}
      >
        {t("monsterMonitor.tabs.buff")}
      </button>
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium border transition-colors {activeTab === 'hate'
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted/30 text-foreground border-border/60 hover:bg-muted/50'}"
        onclick={() => {
          activeTab = "hate";
        }}
      >
        {t("monsterMonitor.tabs.hate")}
      </button>
    </div>
  </section>

  {#if activeTab === "buff"}
    <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
    <div class="space-y-1">
      <h2 class="text-base font-semibold text-foreground">{t("monsterMonitor.buffSearch.title")}</h2>
    </div>

    <div class="flex flex-wrap gap-2">
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium transition-colors {searchTarget === 'self'
          ? 'bg-primary text-primary-foreground'
          : 'bg-muted/40 text-foreground hover:bg-muted/60'}"
        onclick={() => {
          searchTarget = "self";
        }}
      >
        {t("monsterMonitor.buffSearch.self")}
      </button>
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-sm font-medium transition-colors {searchTarget === 'global'
          ? 'bg-primary text-primary-foreground'
          : 'bg-muted/40 text-foreground hover:bg-muted/60'}"
        onclick={() => {
          searchTarget = "global";
        }}
      >
        {t("monsterMonitor.buffSearch.global")}
      </button>
    </div>

    <div class="space-y-3">
      <input
        type="text"
        bind:value={searchKeyword}
        placeholder={searchTarget === "global"
          ? t("monsterMonitor.buffSearch.placeholderGlobal")
          : t("monsterMonitor.buffSearch.placeholderSelf")}
        class="w-full px-3 py-2.5 rounded-lg border border-border bg-background text-sm outline-none focus:border-primary"
      />

      {#if searchKeyword.trim().length > 0}
        <BuffSearchResultGrid
          items={searchResults}
          {availableBuffMap}
          onSelect={toggleSelectedBuff}
          isSelected={isSelectedInCurrentTarget}
          getStatusLabel={searchStatusLabel}
          emptyMessage={t("monsterMonitor.buffSearch.empty")}
        />
      {/if}
    </div>

    <div class="grid gap-4 xl:grid-cols-2">
      <div class="rounded-lg border border-border/60 bg-background/50 p-4 space-y-3">
        <div>
          <div class="text-sm font-semibold text-foreground">{t("monsterMonitor.buffGroups.self.title")}</div>
          <div class="text-xs text-muted-foreground">{t("monsterMonitor.buffGroups.self.description")}</div>
        </div>
        {#if selfAppliedBuffIds.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each selfAppliedBuffIds as buffId (buffId)}
              {@const iconBuff = availableBuffMap.get(buffId)}
              <button
                type="button"
                class="selected-buff"
                onclick={() => removeBuff("self", buffId)}
                title={t("monsterMonitor.buffGroups.removeTitle")}
              >
                {#if iconBuff}
                  <img
                    src={`/images/buff/${iconBuff.spriteFile}`}
                    alt={buffName(buffId)}
                    class="w-8 h-8 rounded object-contain bg-muted/20"
                  />
                {/if}
                <span>{buffName(buffId)}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="text-xs text-muted-foreground">{t("monsterMonitor.buffGroups.empty")}</div>
        {/if}
      </div>

      <div class="rounded-lg border border-border/60 bg-background/50 p-4 space-y-3">
        <div>
          <div class="text-sm font-semibold text-foreground">{t("monsterMonitor.buffGroups.global.title")}</div>
          <div class="text-xs text-muted-foreground">{t("monsterMonitor.buffGroups.global.description")}</div>
        </div>
        {#if globalBuffIds.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each globalBuffIds as buffId (buffId)}
              {@const iconBuff = availableBuffMap.get(buffId)}
              <button
                type="button"
                class="selected-buff"
                onclick={() => removeBuff("global", buffId)}
                title={t("monsterMonitor.buffGroups.removeTitle")}
              >
                {#if iconBuff}
                  <img
                    src={`/images/buff/${iconBuff.spriteFile}`}
                    alt={buffName(buffId)}
                    class="w-8 h-8 rounded object-contain bg-muted/20"
                  />
                {/if}
                <span>{buffName(buffId)}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="text-xs text-muted-foreground">{t("monsterMonitor.buffGroups.empty")}</div>
        {/if}
      </div>
    </div>
  </section>

  <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
    <div class="space-y-2">
      <div class="text-base font-semibold text-foreground">{t("monsterMonitor.priority.title")}</div>
      <div class="text-xs text-muted-foreground">{t("monsterMonitor.priority.description")}</div>
      
      <input
        class="w-full sm:w-72 mt-2 rounded border border-border/60 bg-muted/30 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
        placeholder={t("monsterMonitor.priority.placeholder")}
        value={prioritySearchKeyword}
        oninput={(event) => { prioritySearchKeyword = (event.currentTarget as HTMLInputElement).value; }}
      />
      {#if prioritySearchKeyword.trim().length > 0}
        <BuffSearchResultGrid
          items={prioritySearchResults}
          {availableBuffMap}
          onSelect={toggleMonsterBuffPriority}
          emptyMessage={t("monsterMonitor.priority.emptySearch")}
          minColumnWidth={180}
        />
      {/if}
      <div class="space-y-1 mt-3">
        {#each buffPriorityIds as buffId, idx (buffId)}
          <div class="flex items-center gap-2 rounded border border-border/60 bg-muted/20 px-2 py-1">
            <span class="w-6 text-center text-xs text-muted-foreground">{idx + 1}</span>
            <span class="flex-1 text-xs text-foreground truncate">
              {buffName(buffId)}
            </span>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40" onclick={() => toggleMonsterBuffPriority(buffId)}>{t("monsterMonitor.priority.remove")}</button>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveMonsterBuffPriority(buffId, "up")} disabled={idx === 0}>{t("monsterMonitor.priority.moveUp")}</button>
            <button type="button" class="text-xs px-2 py-0.5 rounded border border-border/60 hover:bg-muted/40 disabled:opacity-50" onclick={() => moveMonsterBuffPriority(buffId, "down")} disabled={idx === buffPriorityIds.length - 1}>{t("monsterMonitor.priority.moveDown")}</button>
          </div>
        {:else}
          <div class="text-xs text-muted-foreground py-2">{t("monsterMonitor.priority.empty")}</div>
        {/each}
      </div>
    </div>
  </section>

  <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
    <div class="space-y-1">
      <h2 class="text-base font-semibold text-foreground">{t("monsterMonitor.alias.title")}</h2>
    </div>

    {#if combinedBuffIds.length > 0}
      <div class="grid gap-3">
        {#each combinedBuffIds as buffId (buffId)}
          <div class="grid gap-2 md:grid-cols-[220px_minmax(0,1fr)] md:items-center">
            <div class="text-sm text-foreground">{defaultBuffName(buffId)}</div>
            <input
              type="text"
              value={buffAliases[buffId] ?? ""}
              placeholder={defaultBuffName(buffId)}
              class="w-full px-3 py-2 rounded-lg border border-border bg-background text-sm outline-none focus:border-primary"
              oninput={(event) =>
                setAlias(
                  buffId,
                  (event.currentTarget as HTMLInputElement).value,
                )}
            />
          </div>
        {/each}
      </div>
    {:else}
      <div class="text-sm text-muted-foreground">{t("monsterMonitor.alias.empty")}</div>
    {/if}
  </section>

  <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
    <div class="space-y-1">
      <h2 class="text-base font-semibold text-foreground">{t("monsterMonitor.style.panelTitle")}</h2>
    </div>

    <div class="grid gap-4 lg:grid-cols-3">
      <label class="style-field">
        <span>{t("monsterMonitor.style.gap")}</span>
        <input
          type="range"
          min="0"
          max="24"
          value={monsterMonitor.panelStyle.gap}
          oninput={(event) =>
            updatePanelStyle(
              "gap",
              Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
            )}
        />
        <strong>{monsterMonitor.panelStyle.gap}px</strong>
      </label>

      <label class="style-field">
        <span>{t("monsterMonitor.style.columnGap")}</span>
        <input
          type="range"
          min="0"
          max="40"
          value={monsterMonitor.panelStyle.columnGap}
          oninput={(event) =>
            updatePanelStyle(
              "columnGap",
              Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
            )}
        />
        <strong>{monsterMonitor.panelStyle.columnGap}px</strong>
      </label>

      <label class="style-field">
        <span>{t("monsterMonitor.style.fontSize")}</span>
        <input
          type="range"
          min="10"
          max="28"
          value={monsterMonitor.panelStyle.fontSize}
          oninput={(event) =>
            updatePanelStyle(
              "fontSize",
              Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
            )}
        />
        <strong>{monsterMonitor.panelStyle.fontSize}px</strong>
      </label>
    </div>

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <label class="color-field">
        <span>{t("monsterMonitor.style.nameColor")}</span>
        <input
          type="color"
          value={monsterMonitor.panelStyle.nameColor}
          oninput={(event) =>
            updatePanelStyle(
              "nameColor",
              (event.currentTarget as HTMLInputElement).value,
            )}
        />
      </label>

      <label class="color-field">
        <span>{t("monsterMonitor.style.valueColor")}</span>
        <input
          type="color"
          value={monsterMonitor.panelStyle.valueColor}
          oninput={(event) =>
            updatePanelStyle(
              "valueColor",
              (event.currentTarget as HTMLInputElement).value,
            )}
        />
      </label>

      <label class="color-field">
        <span>{t("monsterMonitor.style.progressColor")}</span>
        <input
          type="color"
          value={monsterMonitor.panelStyle.progressColor}
          oninput={(event) =>
            updatePanelStyle(
              "progressColor",
              (event.currentTarget as HTMLInputElement).value,
            )}
        />
      </label>

      <label class="color-field">
        <span>{t("monsterMonitor.style.progressOpacity")}</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={monsterMonitor.panelStyle.progressOpacity ?? 0.4}
          oninput={(event) =>
            updatePanelStyle(
              "progressOpacity",
              Number((event.currentTarget as HTMLInputElement).value),
            )}
        />
        <strong>{Math.round((monsterMonitor.panelStyle.progressOpacity ?? 0.4) * 100)}%</strong>
      </label>
    </div>
    </section>
  {:else}
    <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
      <div class="space-y-1">
        <h2 class="text-base font-semibold text-foreground">{t("monsterMonitor.hate.displayTitle")}</h2>
        <p class="text-sm text-muted-foreground">{t("monsterMonitor.hate.displayDescription")}</p>
      </div>

      <div class="flex justify-start">
        <div class="min-w-[220px]">
          <SettingsSwitch
            label={t("monsterMonitor.hate.enabled")}
            bind:checked={SETTINGS.monsterMonitor.state.hateListEnabled}
          />
        </div>
      </div>

      <div class="grid gap-4 lg:grid-cols-3">
        <label class="style-field">
          <span>{t("monsterMonitor.hate.maxDisplay")}</span>
          <input
            type="range"
            min="5"
            max="20"
            step="1"
            value={monsterMonitor.hateListMaxDisplay ?? 5}
            oninput={(event) =>
              updateMonsterMonitor((state) => ({
                ...state,
                hateListMaxDisplay: Number.parseInt(
                  (event.currentTarget as HTMLInputElement).value,
                  10,
                ),
              }))}
          />
          <strong>{monsterMonitor.hateListMaxDisplay ?? 5}</strong>
        </label>
      </div>
    </section>

    <section class="rounded-xl border border-border/60 bg-card/60 p-5 space-y-5">
      <div class="space-y-1">
        <h2 class="text-base font-semibold text-foreground">{t("monsterMonitor.hate.styleTitle")}</h2>
      </div>

      <div class="grid gap-4 lg:grid-cols-3">
        <label class="style-field">
          <span>{t("monsterMonitor.style.gap")}</span>
          <input
            type="range"
            min="0"
            max="24"
            value={hatePanelStyle.gap}
            oninput={(event) =>
              updateHatePanelStyle(
                "gap",
                Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
              )}
          />
          <strong>{hatePanelStyle.gap}px</strong>
        </label>

        <label class="style-field">
          <span>{t("monsterMonitor.style.columnGap")}</span>
          <input
            type="range"
            min="0"
            max="40"
            value={hatePanelStyle.columnGap}
            oninput={(event) =>
              updateHatePanelStyle(
                "columnGap",
                Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
              )}
          />
          <strong>{hatePanelStyle.columnGap}px</strong>
        </label>

        <label class="style-field">
          <span>{t("monsterMonitor.style.fontSize")}</span>
          <input
            type="range"
            min="10"
            max="28"
            value={hatePanelStyle.fontSize}
            oninput={(event) =>
              updateHatePanelStyle(
                "fontSize",
                Number.parseInt((event.currentTarget as HTMLInputElement).value, 10),
              )}
          />
          <strong>{hatePanelStyle.fontSize}px</strong>
        </label>
      </div>

      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <label class="color-field">
          <span>{t("monsterMonitor.style.nameColor")}</span>
          <input
            type="color"
            value={hatePanelStyle.nameColor}
            oninput={(event) =>
              updateHatePanelStyle(
                "nameColor",
                (event.currentTarget as HTMLInputElement).value,
              )}
          />
        </label>

        <label class="color-field">
          <span>{t("monsterMonitor.style.valueColor")}</span>
          <input
            type="color"
            value={hatePanelStyle.valueColor}
            oninput={(event) =>
              updateHatePanelStyle(
                "valueColor",
                (event.currentTarget as HTMLInputElement).value,
              )}
          />
        </label>

        <label class="color-field">
          <span>{t("monsterMonitor.style.progressColor")}</span>
          <input
            type="color"
            value={hatePanelStyle.progressColor}
            oninput={(event) =>
              updateHatePanelStyle(
                "progressColor",
                (event.currentTarget as HTMLInputElement).value,
              )}
          />
        </label>

        <label class="color-field">
          <span>{t("monsterMonitor.style.progressOpacity")}</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={hatePanelStyle.progressOpacity ?? 0.4}
            oninput={(event) =>
              updateHatePanelStyle(
                "progressOpacity",
                Number((event.currentTarget as HTMLInputElement).value),
              )}
          />
          <strong>{Math.round((hatePanelStyle.progressOpacity ?? 0.4) * 100)}%</strong>
        </label>
      </div>
    </section>
  {/if}
</div>

<style>
  .selected-buff {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 10px;
    border: 1px solid rgba(148, 163, 184, 0.35);
    background: rgba(15, 23, 42, 0.35);
    color: var(--foreground, #fff);
    font-size: 12px;
    cursor: pointer;
  }

  .selected-buff:hover {
    border-color: rgba(96, 165, 250, 0.65);
    background: rgba(30, 41, 59, 0.55);
  }

  .style-field,
  .color-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(148, 163, 184, 0.16);
    background: rgba(15, 23, 42, 0.22);
    font-size: 13px;
    color: var(--foreground, #fff);
  }

  .style-field strong {
    font-size: 12px;
    color: var(--muted-foreground, rgba(255, 255, 255, 0.72));
  }

  .color-field input[type="color"] {
    width: 100%;
    height: 42px;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
  }
</style>
