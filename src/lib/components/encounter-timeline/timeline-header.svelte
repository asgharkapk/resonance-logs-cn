<script lang="ts">
  // Header bar: curve legend toggles, viewport indicator/reset, and the
  // teammate lane selector popover.
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import UsersIcon from "@lucide/svelte/icons/users";
  import { getClassIcon } from "$lib/utils.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { formatTimeMs } from "./timeline-format";
  import { playerColor } from "./timeline-colors";
  import { TIMELINE_PALETTE } from "./timeline-palette";
  import type { TimelineViewport } from "./timeline-viewport.svelte";
  import type { TimelinePlayerMeta } from "./timeline-types";

  type Props = {
    showAverageCurve: boolean;
    onToggleAverage: () => void;
    teammates: TimelinePlayerMeta[];
    selectedTeammateUuids: string[];
    onToggleTeammate: (entityUuid: string) => void;
    onSelectAllTeammates: () => void;
    onClearTeammates: () => void;
    viewport: TimelineViewport;
  };

  let {
    showAverageCurve,
    onToggleAverage,
    teammates,
    selectedTeammateUuids,
    onToggleTeammate,
    onSelectAllTeammates,
    onClearTeammates,
    viewport,
  }: Props = $props();

  let selectorOpen = $state(false);
</script>

<div class="tl-header flex items-center justify-between gap-2 px-3 py-1.5">
  <div class="flex min-w-0 flex-wrap items-center gap-1">
    <span class="flex items-center gap-1.5 rounded px-1.5 py-0.5">
      <span
        class="size-1.5 shrink-0 rounded-full"
        style="background: {TIMELINE_PALETTE.mine}; box-shadow: 0 0 5px {TIMELINE_PALETTE.mine}"
      ></span>
      <span class="text-[10px] font-medium" style="color: var(--tl-fg)">
        {t("history.timeline.series.instant")}
      </span>
    </span>
    <button
      type="button"
      class="tl-chip flex cursor-pointer items-center gap-1.5 rounded px-1.5 py-0.5 transition-opacity duration-150 {showAverageCurve
        ? ''
        : 'opacity-40'}"
      onclick={onToggleAverage}
    >
      <span
        class="size-1.5 shrink-0 rounded-full"
        style="background: {TIMELINE_PALETTE.average}"
      ></span>
      <span class="text-[10px]" style="color: var(--tl-fg-muted)">
        {t("history.timeline.series.average")}
      </span>
    </button>

    {#if viewport.isZoomed}
      <span
        class="ml-1 flex items-center gap-1.5 rounded px-1.5 py-0.5 tabular-nums"
        style="color: var(--tl-fg-muted)"
      >
        <span class="text-[10px]">
          {t("history.timeline.zoom.windowLabel", {
            start: formatTimeMs(viewport.startMs),
            end: formatTimeMs(viewport.endMs),
            duration: formatTimeMs(viewport.durationMs),
          })}
        </span>
      </span>
      <button
        type="button"
        class="tl-chip flex cursor-pointer items-center gap-1 rounded px-1.5 py-0.5"
        style="color: var(--tl-fg-muted)"
        onclick={() => viewport.reset()}
        title={t("history.timeline.zoom.reset")}
      >
        <RotateCcwIcon class="size-3 shrink-0" />
        <span class="text-[10px]">{t("history.timeline.zoom.reset")}</span>
      </button>
    {/if}
  </div>

  <div class="flex shrink-0 items-center gap-1">
    {#if teammates.length > 0}
      <div class="relative shrink-0">
        <button
          type="button"
          class="tl-chip flex cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-[10px] transition-colors duration-150"
          style="color: var(--tl-fg-muted)"
          onclick={() => (selectorOpen = !selectorOpen)}
        >
          <UsersIcon class="size-3 shrink-0" />
          <span>{t("history.timeline.lanes.selectTeammates")}</span>
          <span
            class="rounded px-1 tabular-nums"
            style="background: rgba(148,163,184,0.12); color: var(--tl-fg)"
          >
            {selectedTeammateUuids.length}/{teammates.length}
          </span>
          <ChevronDownIcon
            class="size-2.5 shrink-0 transition-transform duration-150 {selectorOpen
              ? 'rotate-180'
              : ''}"
            strokeWidth={2.5}
          />
        </button>

        {#if selectorOpen}
          <!-- Click-away backdrop. -->
          <button
            type="button"
            class="fixed inset-0 z-10 cursor-default"
            aria-label={t("history.timeline.lanes.closeSelector")}
            onclick={() => (selectorOpen = false)}
          ></button>
          <div
            class="tl-popover absolute right-0 z-20 mt-1 w-52 rounded-md py-1 shadow-xl"
          >
            <div
              class="flex items-center justify-between px-2.5 pt-1 pb-1.5"
              style="border-bottom: 1px solid var(--tl-row-line)"
            >
              <button
                type="button"
                class="cursor-pointer text-[10px] transition-colors duration-150 hover:underline"
                style="color: var(--tl-fg-muted)"
                onclick={onSelectAllTeammates}
              >
                {t("history.timeline.lanes.selectAll")}
              </button>
              <button
                type="button"
                class="cursor-pointer text-[10px] transition-colors duration-150 hover:underline"
                style="color: var(--tl-fg-muted)"
                onclick={onClearTeammates}
              >
                {t("history.timeline.lanes.clearAll")}
              </button>
            </div>
            <div class="max-h-56 overflow-y-auto">
              {#each teammates as player (player.entityUuid)}
                {@const checked = selectedTeammateUuids.includes(
                  player.entityUuid,
                )}
                <label
                  class="tl-chip flex cursor-pointer items-center gap-2 px-2.5 py-1.5 transition-colors duration-150"
                >
                  <input
                    type="checkbox"
                    class="size-3 shrink-0 accent-blue-400"
                    {checked}
                    onchange={() => onToggleTeammate(player.entityUuid)}
                  />
                  <img
                    class="size-3.5 shrink-0 object-contain"
                    src={getClassIcon(player.className)}
                    alt=""
                  />
                  <span
                    class="truncate text-[11px]"
                    style="color: {playerColor(player)}"
                  >
                    {player.name}
                  </span>
                </label>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .tl-header {
    background: var(--tl-header-bg);
    border-bottom: 1px solid var(--tl-row-line);
  }

  .tl-chip:hover {
    background: rgba(148, 163, 184, 0.1);
  }

  .tl-popover {
    background: var(--tl-popover-bg);
    border: 1px solid rgba(148, 163, 184, 0.18);
  }
</style>
