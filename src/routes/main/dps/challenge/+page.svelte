<script lang="ts">
  /**
   * @file Challenge-watch (forbidden-damage) settings — promoted to its own
   * top-level DPS tab (peer to 历史/主题/设置) since the forbidden-damage
   * list now travels with the active loadout and deserves the same
   * visibility as the other loadout-scoped pages.
   */
  import { SETTINGS } from "$lib/settings-store";
  import { t, getLocale } from "$lib/i18n/index.svelte";
  import X from "virtual:icons/lucide/x";
  import Check from "virtual:icons/lucide/check";
  import { CHALLENGE_PRESETS } from "$lib/challenge-presets";
  import { searchForbiddenDamage } from "$lib/challenge-damage-search";
  import { lookupDamageIdName } from "$lib/config/recount-table";

  let challengeSearch = $state("");
  const challengeLocale = $derived(getLocale());
  const forbiddenDamageIds = $derived(
    SETTINGS.challengeWatch.state.forbiddenDamageIds,
  );
  // Name-based search over game damage/mechanic names (users never see raw ids).
  const challengeSearchResult = $derived(
    searchForbiddenDamage(challengeSearch, challengeLocale),
  );
  // Configured ids resolved back to readable names for display.
  const configuredDamageItems = $derived(
    forbiddenDamageIds.map((id) => ({
      id,
      name: lookupDamageIdName(id, challengeLocale),
    })),
  );

  function setForbiddenDamageIds(ids: number[]) {
    // Reassign the whole array so the RuneStore proxy persists the change.
    SETTINGS.challengeWatch.state.forbiddenDamageIds = ids;
  }

  function addForbiddenIds(ids: number[]) {
    const merged = [...forbiddenDamageIds];
    for (const id of ids) {
      if (Number.isFinite(id) && id > 0 && !merged.includes(id)) {
        merged.push(id);
      }
    }
    if (merged.length !== forbiddenDamageIds.length) {
      setForbiddenDamageIds(merged);
    }
  }

  function removeForbiddenId(id: number) {
    setForbiddenDamageIds(
      forbiddenDamageIds.filter((existing) => existing !== id),
    );
  }

  function isResultAdded(ids: number[]): boolean {
    return ids.length > 0 && ids.every((id) => forbiddenDamageIds.includes(id));
  }

  function toggleResult(ids: number[]) {
    if (isResultAdded(ids)) {
      const remove = new Set(ids);
      setForbiddenDamageIds(
        forbiddenDamageIds.filter((id) => !remove.has(id)),
      );
    } else {
      addForbiddenIds(ids);
    }
  }
</script>

<div class="space-y-3">
  <p class="text-muted-foreground text-xs">
    {t("settings.scope.live")}
  </p>

  <div
    class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
  >
    <div class="px-4 py-3">
      <h2 class="text-base font-semibold text-foreground">
        {t("challengeWatch.title")}
      </h2>
    </div>
    <div class="px-4 pb-4 pt-1 space-y-3">
      <p class="text-xs text-muted-foreground">
        {t("challengeWatch.description")}
      </p>

      <!-- Curated challenge presets (one-click bundles) -->
      <div class="space-y-1.5">
        <div class="text-xs font-medium text-muted-foreground">
          {t("challengeWatch.presets.title")}
        </div>
        <div class="flex flex-wrap gap-2">
          {#each CHALLENGE_PRESETS as preset (preset.id)}
            <button
              type="button"
              onclick={() => addForbiddenIds(preset.damageIds)}
              class="h-8 px-3 rounded-full border border-border/60 bg-muted/30 hover:bg-muted/60 text-xs font-medium text-foreground transition-colors cursor-pointer"
            >
              + {t(preset.labelKey)}
            </button>
          {/each}
        </div>
      </div>

      <!-- Search by mechanic / damage name -->
      <div class="space-y-1.5">
        <input
          type="text"
          bind:value={challengeSearch}
          placeholder={t("challengeWatch.searchPlaceholder")}
          class="w-full h-9 px-3 rounded-md border border-border/60 bg-background/60 text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-ring/40 transition-colors"
        />

        {#if challengeSearch.trim()}
          {#if challengeSearchResult.results.length === 0}
            <div class="text-xs text-muted-foreground/70 italic py-1">
              {t("challengeWatch.searchNoResults")}
            </div>
          {:else}
            <div class="text-xs text-muted-foreground">
              {t("challengeWatch.searchCount", {
                count: challengeSearchResult.total,
              })}
            </div>
            <div
              class="max-h-56 overflow-y-auto rounded-md border border-border/60 divide-y divide-border/40"
            >
              {#each challengeSearchResult.results as result (result.key)}
                {@const added = isResultAdded(result.ids)}
                <button
                  type="button"
                  onclick={() => toggleResult(result.ids)}
                  class="w-full flex items-center gap-2 px-3 py-1.5 text-left hover:bg-muted/40 transition-colors cursor-pointer"
                >
                  <span
                    class="inline-flex items-center justify-center w-4 h-4 shrink-0 rounded border {added
                      ? 'bg-primary/80 border-primary text-primary-foreground'
                      : 'border-border/60 text-transparent'}"
                  >
                    <Check class="w-3 h-3" />
                  </span>
                  <span class="flex-1 min-w-0 truncate text-sm text-foreground">
                    {result.name}
                  </span>
                  {#if result.isGroup}
                    <span class="text-[11px] text-muted-foreground shrink-0">
                      {t("challengeWatch.groupCount", {
                        count: result.ids.length,
                      })}
                    </span>
                  {:else}
                    <span
                      class="text-[11px] text-muted-foreground/60 tabular-nums shrink-0"
                    >
                      {result.ids[0]}
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
            {#if challengeSearchResult.total > challengeSearchResult.results.length}
              <div class="text-[11px] text-muted-foreground/70 italic">
                {t("challengeWatch.searchMore", {
                  shown: challengeSearchResult.results.length,
                })}
              </div>
            {/if}
          {/if}
        {/if}
      </div>

      <!-- Configured forbidden damages (shown by name) -->
      <div class="space-y-1.5">
        <div class="text-xs font-medium text-muted-foreground">
          {t("challengeWatch.configuredTitle")}
        </div>
        {#if configuredDamageItems.length === 0}
          <div class="text-xs text-muted-foreground/70 italic py-1">
            {t("challengeWatch.empty")}
          </div>
        {:else}
          <div class="flex flex-wrap gap-2">
            {#each configuredDamageItems as item (item.id)}
              <span
                class="inline-flex items-center gap-1.5 h-7 pl-2.5 pr-1.5 rounded-full bg-muted/40 border border-border/60 text-xs text-foreground"
              >
                <span class="max-w-[16rem] truncate">{item.name}</span>
                <button
                  type="button"
                  onclick={() => removeForbiddenId(item.id)}
                  aria-label={t("challengeWatch.remove")}
                  class="inline-flex items-center justify-center w-4 h-4 rounded-full text-muted-foreground hover:text-red-500 hover:bg-red-500/10 transition-colors cursor-pointer"
                >
                  <X class="w-3 h-3" />
                </button>
              </span>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
