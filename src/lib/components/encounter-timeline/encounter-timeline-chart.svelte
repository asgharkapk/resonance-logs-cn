<script lang="ts">
  // Per-encounter timeline panel, rendered as a self-contained dark "chart
  // island" that does not follow the global color theme (canvas cannot read
  // CSS variables; the semantic palette is tuned for a dark surface).
  //
  // Architecture (see plan for the full rationale): a single `TimelineViewport`
  // is the only source of truth for the visible time window. Lane markers and
  // gesture/tooltip handling are plain DOM (`timeline-lanes.svelte`,
  // `timeline-gestures.ts`, `timeline-overlay.svelte`); ECharts
  // (`timeline-curve.svelte`) is reduced to a "dumb" renderer that only draws
  // the DPS curves and owns no interaction state at all. This replaces the
  // old canvas-only implementation, where lane markers were hand-drawn
  // `renderItem` shapes with a manual text-width estimator, and the ECharts
  // brush permanently occupied left-drag (so pan and zoom were effectively
  // unusable once the encounter was zoomed in).
  //
  // Structure, top to bottom:
  //   - header: curve legend, viewport indicator/reset, teammate lane
  //     selector;
  //   - chart body: swim-lane markers + DPS curve, sharing one coordinate
  //     system driven by `viewport`;
  //   - minimap: full-encounter overview with a draggable/resizable window;
  //   - selection bar: active brush range, "zoom to selection", clear button.
  import { untrack } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import Clock3Icon from "@lucide/svelte/icons/clock-3";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import ZoomInIcon from "@lucide/svelte/icons/zoom-in";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import { t } from "$lib/i18n/index.svelte";
  import { tooltip } from "$lib/utils.svelte";
  import { laneColor, playerColor } from "./timeline-colors";
  import {
    foldEncounterDamageBuckets,
    toCumulativeDpsCurve,
    toRollingDpsCurve,
    zoomTierFor,
    type EncounterChart,
    type EncounterTimelineEvent,
  } from "./timeline-data";
  import { formatTimeMs } from "./timeline-format";
  import { RIGHT_PADDING, computeTimelineLayout } from "./timeline-layout";
  import { timelineGestures } from "./timeline-gestures";
  import { TIMELINE_PALETTE, timelinePaletteCssVars } from "./timeline-palette";
  import TimelineCurve from "./timeline-curve.svelte";
  import TimelineHeader from "./timeline-header.svelte";
  import TimelineLanes from "./timeline-lanes.svelte";
  import TimelineMinimap from "./timeline-minimap.svelte";
  import TimelineOverlay from "./timeline-overlay.svelte";
  import type {
    Lane,
    LanePoint,
    TimelineBossMeta as TimelineBossMetaSource,
    TimelineEventDisplay as TimelineEventDisplaySource,
    TimelineHoverPoint,
    TimelinePlayerMeta as TimelinePlayerMetaSource,
  } from "./timeline-types";
  import { TimelineViewport } from "./timeline-viewport.svelte";

  // Re-exported for backward compatibility: callers import these types from
  // this component's module, e.g.
  // `import EncounterTimelineChart, { type TimelineEventDisplay } from "...";`
  // (a bare `export type { X }` re-export isn't picked up by svelte-check's
  // component typing, so these need to be real local alias declarations.)
  export type TimelineBossMeta = TimelineBossMetaSource;
  export type TimelineEventDisplay = TimelineEventDisplaySource;
  export type TimelinePlayerMeta = TimelinePlayerMetaSource;

  /** Support classes whose key casts (heal / concerto) are shown by default. */
  const DEFAULT_LANE_CLASSES = new Set(["Verdant Oracle", "Beat Performer"]);

  type Props = {
    chart: EncounterChart;
    events: EncounterTimelineEvent[];
    /** Party members shown in the lanes / curves. */
    players?: TimelinePlayerMeta[];
    /** Boss casters used to label their lanes; unknown casters fall back to a
     * generic label. */
    bosses?: TimelineBossMeta[];
    /** Whether exact millisecond recount is available for this encounter. */
    selectionEnabled?: boolean;
    /** Whether the parent is recounting the currently selected range. */
    selectionPending?: boolean;
    /** Selected half-open [startMs, endMs) range, null when unselected. */
    selectedRange?: [number, number] | null;
    /** Resolve the display strings/icon for one marker event. */
    resolveEvent: (event: EncounterTimelineEvent) => TimelineEventDisplay;
  };

  let {
    chart,
    events,
    players = [],
    bosses = [],
    selectionEnabled = true,
    selectionPending = false,
    selectedRange = $bindable(null),
    resolveEvent,
  }: Props = $props();

  // ---- Damage buckets / DPS curves -----------------------------------------
  // The DTO is sparse and column-oriented. Fold it once into per-entity
  // damage buckets; curves are built on demand from those buckets so the
  // cost stays independent of party size.
  const damageBuckets = $derived(foldEncounterDamageBuckets(chart));
  const chartDurationMs = $derived(damageBuckets.durationMs);
  const chartBucketMs = $derived(damageBuckets.bucketMs);
  const perEntityBuckets = $derived(damageBuckets.perEntityBuckets);

  const localPlayer = $derived(players.find((p) => p.isLocalPlayer) ?? null);

  const mineBuckets = $derived.by(() =>
    localPlayer ? (perEntityBuckets.get(localPlayer.entityUuid) ?? null) : null,
  );

  const mineInstantCurve = $derived.by(() =>
    mineBuckets
      ? toRollingDpsCurve(mineBuckets, chartBucketMs, chartDurationMs)
      : null,
  );

  const mineAverageCurve = $derived.by(() =>
    mineBuckets
      ? toCumulativeDpsCurve(mineBuckets, chartBucketMs, chartDurationMs)
      : null,
  );

  function clampEventOffsetMs(ev: EncounterTimelineEvent): number {
    const offsetMs = Number(ev.tsOffsetMs);
    if (!Number.isFinite(offsetMs)) return 0;
    return Math.min(chartDurationMs, Math.max(0, offsetMs));
  }

  // ---- Lane grouping --------------------------------------------------------
  /** Boss-cast events grouped by caster uuid, one lane per caster. */
  let bossEventsByCaster = $derived.by(() => {
    const map = new SvelteMap<string, EncounterTimelineEvent[]>();
    for (const ev of events) {
      if (ev.kind !== "boss_skill") continue;
      const list = map.get(ev.casterUuid) ?? [];
      list.push(ev);
      map.set(ev.casterUuid, list);
    }
    for (const list of map.values()) {
      list.sort((a, b) => a.tsOffsetMs - b.tsOffsetMs);
    }
    return map;
  });

  let bossNameByUuid = $derived.by(() => {
    const map = new SvelteMap<string, string>();
    for (const boss of bosses) {
      if (!boss.name || map.has(boss.entityUuid)) continue;
      map.set(boss.entityUuid, boss.name);
    }
    return map;
  });

  /** Player-cast events (fantasy + key skill) grouped by caster uuid. */
  let playerEventsByCaster = $derived.by(() => {
    const map = new SvelteMap<string, EncounterTimelineEvent[]>();
    for (const ev of events) {
      if (ev.kind === "boss_skill") continue;
      const list = map.get(ev.casterUuid) ?? [];
      list.push(ev);
      map.set(ev.casterUuid, list);
    }
    for (const list of map.values()) {
      list.sort((a, b) => a.tsOffsetMs - b.tsOffsetMs);
    }
    return map;
  });

  /** Non-local players that have casts or damage worth a lane/curve. */
  let teammates = $derived.by(() => {
    return players.filter((p) => {
      if (localPlayer && p.entityUuid === localPlayer.entityUuid) return false;
      return (
        (playerEventsByCaster.get(p.entityUuid)?.length ?? 0) > 0 ||
        (perEntityBuckets.get(p.entityUuid)?.some((total) => total > 0) ??
          false)
      );
    });
  });

  /** Default lane selection: supports that actually cast key skills. */
  let defaultTeammateUuids = $derived(
    teammates
      .filter(
        (p) =>
          DEFAULT_LANE_CLASSES.has(p.className) &&
          (playerEventsByCaster.get(p.entityUuid)?.length ?? 0) > 0,
      )
      .map((p) => p.entityUuid),
  );

  // null = "use the default" (support players); a string[] once the user has
  // interacted with the selector.
  let manualTeammateSelection = $state<string[] | null>(null);

  let selectedTeammateUuids = $derived(
    manualTeammateSelection ?? defaultTeammateUuids,
  );

  let selectedTeammates = $derived(
    teammates.filter((p) => selectedTeammateUuids.includes(p.entityUuid)),
  );

  function toggleTeammate(entityUuid: string) {
    const current = manualTeammateSelection ?? [...selectedTeammateUuids];
    manualTeammateSelection = current.includes(entityUuid)
      ? current.filter((uuid) => uuid !== entityUuid)
      : [...current, entityUuid];
  }

  function selectAllTeammates() {
    manualTeammateSelection = teammates.map((p) => p.entityUuid);
  }

  function clearTeammates() {
    manualTeammateSelection = [];
  }

  function toPoints(list: EncounterTimelineEvent[]): LanePoint[] {
    return list.map((ev) => ({ timeMs: clampEventOffsetMs(ev), event: ev }));
  }

  let lanes = $derived.by<Lane[]>(() => {
    const result: Lane[] = [];
    // One lane per boss caster, ordered by its first cast so the boss that
    // engages first sits on top.
    const bossCasters = [...bossEventsByCaster].sort(
      (a, b) => (a[1][0]?.tsOffsetMs ?? 0) - (b[1][0]?.tsOffsetMs ?? 0),
    );
    for (const [casterUuid, list] of bossCasters) {
      result.push({
        key: `boss-${casterUuid}`,
        type: "boss",
        name:
          bossNameByUuid.get(casterUuid) ?? t("history.timeline.lanes.boss"),
        points: toPoints(list),
      });
    }
    if (localPlayer) {
      result.push({
        key: "mine",
        type: "mine",
        player: localPlayer,
        points: toPoints(playerEventsByCaster.get(localPlayer.entityUuid) ?? []),
      });
    }
    for (const player of selectedTeammates) {
      result.push({
        key: `teammate-${player.entityUuid}`,
        type: "teammate",
        player,
        points: toPoints(playerEventsByCaster.get(player.entityUuid) ?? []),
      });
    }
    return result;
  });

  let mineLaneIndex = $derived(lanes.findIndex((l) => l.type === "mine"));

  // ---- Layout / viewport ----------------------------------------------------
  // Seeded once from the initial value (avoids a [0, 1] first frame); the
  // effect below keeps it in sync as `chartDurationMs` changes thereafter.
  const viewport = new TimelineViewport(untrack(() => chartDurationMs));

  // Discrete zoom tier (0/1/2), stepped rather than continuous so the panel
  // only reflows when the viewport crosses a threshold instead of on every
  // wheel tick. Drives `laneH`/`iconSize` growth; see timeline-layout.ts.
  let zoomTier = $derived(zoomTierFor(viewport.durationMs, viewport.spanMs));
  let layout = $derived(computeTimelineLayout(lanes.length, zoomTier));
  // `chart` only changes when a different encounter is loaded (range
  // recounts reuse the same full-encounter chart), so this effectively
  // resets zoom/pan on encounter switches without fighting user interaction.
  $effect(() => {
    viewport.setDuration(chartDurationMs);
  });

  let showAverageCurve = $state(true);
  let hoverPoint = $state<TimelineHoverPoint | null>(null);
  let brushPreviewMs = $state<[number, number] | null>(null);
  let plotWidthPx = $state(0);

  // Created once: the options object's accessors close over the component's
  // reactive bindings directly, so the attachment's pointer/wheel listeners
  // are wired up exactly once per mount instead of on every re-render.
  const gestureAttachment = timelineGestures({
    getViewport: () => viewport,
    getSelectionEnabled: () => selectionEnabled,
    onBrushPreview: (range) => {
      brushPreviewMs = range;
    },
    onSelectRange: (range) => {
      selectedRange = range;
    },
    onHover: (point) => {
      hoverPoint = point;
    },
  });

  function zoomToSelection() {
    if (selectedRange) viewport.zoomToRange(selectedRange);
  }

  // ---- Icon prewarm -----------------------------------------------------
  // zrender repaints once each marker image finishes loading; warming icon
  // URLs up front just removes the first-frame flicker for canvas rendering
  // and is a cheap no-op for the DOM `<img>` markers.
  const warmedIconUrls = new SvelteSet<string>();
  $effect(() => {
    for (const ev of events) {
      const iconPath = resolveEvent(ev).iconPath;
      if (iconPath === null || warmedIconUrls.has(iconPath)) continue;
      warmedIconUrls.add(iconPath);
      const img = new Image();
      img.src = iconPath;
    }
  });

  // Clear the visual brush preview if the parent resets the selection
  // externally (e.g. switching encounters).
  $effect(() => {
    if (selectedRange === null) brushPreviewMs = null;
  });
</script>

<!-- Fixed dark "chart island": interior colors are intentionally decoupled
     from the app theme; only the outer border follows the theme. -->
<div
  class="tl-panel border-border overflow-hidden rounded-md border"
  style={timelinePaletteCssVars()}
>
  <TimelineHeader
    {showAverageCurve}
    onToggleAverage={() => (showAverageCurve = !showAverageCurve)}
    {teammates}
    {selectedTeammateUuids}
    onToggleTeammate={toggleTeammate}
    onSelectAllTeammates={selectAllTeammates}
    onClearTeammates={clearTeammates}
    {viewport}
  />

  <!-- Chart body: backdrop rows + curve + lane markers + gesture/overlay
       surface, pixel-aligned via a single shared layout module. -->
  <div class="relative w-full" style="height: {layout.totalHeight}px">
    <!-- Backdrop layer: local-player row highlight + lane separators. -->
    <div class="pointer-events-none absolute inset-0">
      {#if mineLaneIndex >= 0}
        <div
          class="absolute right-0 left-0"
          style="top: {layout.laneTop +
            mineLaneIndex * layout.laneH}px; height: {layout.laneH}px;
                 background: rgba(96,165,250,0.06);
                 border-left: 2px solid rgba(96,165,250,0.7)"
        ></div>
      {/if}
      {#each lanes as lane, i (lane.key)}
        <div
          class="absolute right-0 left-0"
          style="top: {layout.laneTop +
            (i + 1) * layout.laneH}px; height: 1px;
                 background: var(--tl-row-line)"
        ></div>
      {/each}
    </div>

    <!-- DPS curve (ECharts, non-interactive; the overlay above draws its own
         crosshair/tooltip). Spans the full body width; its internal grid
         reserves the same GUTTER column the lane labels use. -->
    <div
      class="absolute left-0 right-0"
      style="top: {layout.curveTop}px; height: {layout.curveH}px"
    >
      <TimelineCurve
        {mineInstantCurve}
        {mineAverageCurve}
        {showAverageCurve}
        startMs={viewport.startMs}
        endMs={viewport.endMs}
        curveH={layout.curveH}
      />
    </div>

    <!-- Lane markers, positioned within the plot column only (right of the
         gutter), matching the gesture layer's coordinate space below. -->
    <div
      class="absolute"
      style="left: {layout.gutter}px; right: {RIGHT_PADDING}px;
             top: {layout.laneTop}px; height: {layout.lanesHeight}px"
    >
      <TimelineLanes
        {lanes}
        startMs={viewport.startMs}
        endMs={viewport.endMs}
        laneH={layout.laneH}
        iconSize={layout.iconSize}
        {plotWidthPx}
        {resolveEvent}
      />
    </div>

    <!-- Interactive plot surface: owns every pointer/wheel gesture (zoom,
         pan, brush-select, hover) for both the lanes and the curve below. -->
    <div
      class="absolute cursor-crosshair"
      style="left: {layout.gutter}px; right: {RIGHT_PADDING}px;
             top: {layout.laneTop}px;
             height: {layout.lanesHeight + layout.laneGap + layout.curveH}px"
      bind:clientWidth={plotWidthPx}
      {@attach gestureAttachment}
      role="img"
      aria-label={t("history.timeline.chartAriaLabel")}
      title={t("history.timeline.hint.gestures")}
    >
      <TimelineOverlay
        {lanes}
        laneH={layout.laneH}
        lanesHeight={layout.lanesHeight}
        startMs={viewport.startMs}
        endMs={viewport.endMs}
        {hoverPoint}
        {brushPreviewMs}
        selectedRange={selectionEnabled ? selectedRange : null}
        {mineInstantCurve}
        {mineAverageCurve}
        {showAverageCurve}
        {resolveEvent}
      />
    </div>

    <!-- Lane gutter labels. Narrow on purpose so the plot gets the width, so
         names truncate hard - each row carries the full one as a tooltip.
         Pointer events are enabled here (unlike the rest of the overlays)
         because this column sits entirely left of the gesture surface. -->
    <div
      class="pointer-events-none absolute top-0 left-0"
      style="width: {layout.gutter}px; height: {layout.curveTop +
        layout.curveH}px"
    >
      {#each lanes as lane, i (lane.key)}
        {@const laneName = lane.type === "boss" ? lane.name : lane.player.name}
        <div
          class="pointer-events-auto absolute right-2 left-2.5 flex items-center gap-1.5 overflow-hidden"
          style="top: {layout.laneTop + i * layout.laneH}px; height: {layout.laneH}px"
          {@attach tooltip(() => laneName)}
        >
          {#if lane.type === "boss"}
            <TriangleAlertIcon
              class="size-3.5 shrink-0"
              style="color: {TIMELINE_PALETTE.boss}"
            />
            <span
              class="truncate text-[11px]"
              style="color: {laneColor(lane)}"
            >
              {lane.name}
            </span>
          {:else if lane.type === "mine"}
            <span
              class="size-1.5 shrink-0 rounded-full"
              style="background: {TIMELINE_PALETTE.mine}; box-shadow: 0 0 6px {TIMELINE_PALETTE.mine}"
            ></span>
            <span class="truncate text-[11px] font-medium" style="color: #dbeafe">
              {lane.player.name}
            </span>
          {:else}
            <span
              class="size-1.5 shrink-0 rounded-full"
              style="background: {playerColor(lane.player)}"
            ></span>
            <span
              class="truncate text-[11px]"
              style="color: {playerColor(lane.player)}"
            >
              {lane.player.name}
            </span>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Minimap: full-encounter overview + draggable/resizable viewport. -->
  <div
    class="px-3 py-2"
    style="padding-left: {layout.gutter}px; padding-right: {RIGHT_PADDING}px;
           border-top: 1px solid var(--tl-row-line)"
  >
    <!-- The minimap is a navigation strip, so it shows the instant curve: its
         burst peaks are landmarks you can steer by, whereas the cumulative
         average flattens into a shape with nothing to aim at. It also keeps
         the minimap blue, matching the series it now actually draws. -->
    <TimelineMinimap
      {viewport}
      curve={mineInstantCurve ?? mineAverageCurve}
      selectedRange={selectionEnabled ? selectedRange : null}
    />
  </div>

  <!-- Selection bar: active brush range + zoom-to-selection + clear. -->
  {#if selectionEnabled && selectedRange}
    <div
      class="flex items-center justify-between gap-2 px-3 py-1.5"
      style="border-top: 1px solid var(--tl-row-line); background: rgba(96,165,250,0.05)"
    >
      <div class="flex min-w-0 items-center gap-1.5">
        <Clock3Icon class="size-3.5 shrink-0" style="color: {TIMELINE_PALETTE.mine}" />
        <span class="truncate text-[11px] tabular-nums" style="color: #bfdbfe">
          {t("history.timeline.selection.label", {
            start: formatTimeMs(selectedRange[0], true),
            end: formatTimeMs(selectedRange[1], true),
            duration: formatTimeMs(selectedRange[1] - selectedRange[0], true),
          })}
        </span>
        {#if selectionPending}
          <span
            class="flex shrink-0 items-center gap-1 text-[10px]"
            style="color: var(--tl-fg-muted)"
          >
            <LoaderCircleIcon class="size-3 animate-spin" />
            {t("history.detail.loading")}
          </span>
        {/if}
      </div>
      <div class="flex shrink-0 items-center gap-1.5">
        <button
          type="button"
          class="flex cursor-pointer items-center gap-1 rounded border px-2 py-0.5 text-[10px] transition-colors duration-150 hover:bg-blue-400/15"
          style="border-color: rgba(96,165,250,0.4); color: {TIMELINE_PALETTE.mine}"
          onclick={zoomToSelection}
        >
          <ZoomInIcon class="size-3" />
          {t("history.timeline.selection.zoomToRange")}
        </button>
        <button
          type="button"
          class="cursor-pointer rounded border px-2 py-0.5 text-[10px] transition-colors duration-150 hover:bg-blue-400/15"
          style="border-color: rgba(96,165,250,0.4); color: {TIMELINE_PALETTE.mine}"
          onclick={() => (selectedRange = null)}
        >
          {t("history.timeline.selection.clear")}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .tl-panel {
    background: var(--tl-bg);
  }
</style>
