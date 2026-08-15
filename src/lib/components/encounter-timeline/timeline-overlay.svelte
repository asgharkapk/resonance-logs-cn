<script lang="ts">
  // Visual-only layer stacked on top of the gesture surface: crosshair +
  // hover tooltip (replaces ECharts' axis tooltip), plus the live brush
  // preview and the persisted selection highlight. Everything here is
  // `pointer-events: none` - the gesture layer underneath owns all input.
  import { interpolateCurveValue } from "./timeline-data";
  import { formatTimeMs, formatValue } from "./timeline-format";
  import { t } from "$lib/i18n/index.svelte";
  import { SETTINGS } from "$lib/settings-store";
  import type {
    EncounterCurvePoint,
    EncounterTimelineEvent,
  } from "./timeline-data";
  import type {
    Lane,
    TimelineEventDisplay,
    TimelineHoverPoint,
  } from "./timeline-types";

  type Props = {
    lanes: Lane[];
    laneH: number;
    lanesHeight: number;
    startMs: number;
    endMs: number;
    hoverPoint: TimelineHoverPoint | null;
    brushPreviewMs: [number, number] | null;
    selectedRange: [number, number] | null;
    mineInstantCurve: EncounterCurvePoint[] | null;
    mineAverageCurve: EncounterCurvePoint[] | null;
    showAverageCurve: boolean;
    resolveEvent: (event: EncounterTimelineEvent) => TimelineEventDisplay;
  };

  let {
    lanes,
    laneH,
    lanesHeight,
    startMs,
    endMs,
    hoverPoint,
    brushPreviewMs,
    selectedRange,
    mineInstantCurve,
    mineAverageCurve,
    showAverageCurve,
    resolveEvent,
  }: Props = $props();

  const spanMs = $derived(Math.max(1, endMs - startMs));

  /** Same abbreviation the curve's Y axis uses, so the tooltip and the axis
   * never disagree about a value's unit. Called from the markup, so the
   * settings read is tracked by the template's own effect. */
  function formatCurveValue(value: number): string {
    return formatValue(
      value,
      SETTINGS.history.general.state.abbreviationStyle,
      SETTINGS.history.general.state.abbreviatedDecimalPlaces ?? 1,
    );
  }

  function toLeftPct(timeMs: number): number {
    return ((timeMs - startMs) / spanMs) * 100;
  }

  // ~1.2% of the visible span, floored at 30ms: gives a consistent "hit
  // radius" around the cursor regardless of zoom level.
  const toleranceMs = $derived(Math.max(30, spanMs * 0.012));

  const hoverLaneIndex = $derived.by(() => {
    if (!hoverPoint) return null;
    if (hoverPoint.y < 0 || hoverPoint.y >= lanesHeight) return null;
    const index = Math.floor(hoverPoint.y / laneH);
    return index >= 0 && index < lanes.length ? index : null;
  });

  const hoverLaneEvents = $derived.by(() => {
    if (hoverLaneIndex === null || !hoverPoint) return [];
    const lane = lanes[hoverLaneIndex];
    if (!lane) return [];
    const point = hoverPoint;
    return lane.points
      .filter((p) => Math.abs(p.timeMs - point.timeMs) <= toleranceMs)
      .sort(
        (a, b) =>
          Math.abs(a.timeMs - point.timeMs) - Math.abs(b.timeMs - point.timeMs),
      )
      .slice(0, 6);
  });

  const hoverCurveInfo = $derived.by(() => {
    if (!hoverPoint || hoverLaneIndex !== null) return null;
    const instant = interpolateCurveValue(mineInstantCurve, hoverPoint.timeMs);
    const average = showAverageCurve
      ? interpolateCurveValue(mineAverageCurve, hoverPoint.timeMs)
      : null;
    if (instant === null && average === null) return null;
    return { instant, average };
  });

  const showTooltip = $derived(
    hoverPoint !== null &&
      (hoverLaneEvents.length > 0 || hoverCurveInfo !== null),
  );

  const tooltipLeftPct = $derived(hoverPoint ? toLeftPct(hoverPoint.timeMs) : 0);
  const tooltipFlip = $derived(tooltipLeftPct > 65);

  const brushRectStyle = $derived.by(() => {
    if (!brushPreviewMs) return null;
    const [a, b] = brushPreviewMs;
    const left = toLeftPct(Math.min(a, b));
    const width = toLeftPct(Math.max(a, b)) - left;
    return `left: ${left}%; width: ${width}%`;
  });

  const selectionRectStyle = $derived.by(() => {
    if (!selectedRange) return null;
    const left = toLeftPct(selectedRange[0]);
    const width = toLeftPct(selectedRange[1]) - left;
    if (width <= 0) return null;
    return `left: ${Math.max(0, left)}%; width: ${width}%`;
  });
</script>

<div class="pointer-events-none absolute inset-0 overflow-hidden">
  {#if selectionRectStyle}
    <div class="tl-selection-rect" style={selectionRectStyle}></div>
  {/if}
  {#if brushRectStyle}
    <div class="tl-brush-rect" style={brushRectStyle}></div>
  {/if}
  {#if hoverPoint}
    <div class="tl-crosshair" style="left: {tooltipLeftPct}%"></div>
  {/if}
  {#if showTooltip && hoverPoint}
    <div
      class="tl-tooltip"
      class:tl-tooltip--flip={tooltipFlip}
      style="left: {tooltipLeftPct}%; top: {hoverPoint.y}px"
    >
      <div class="tl-tooltip-time">{formatTimeMs(hoverPoint.timeMs, true)}</div>
      {#if hoverLaneIndex !== null}
        {#each hoverLaneEvents as point (point.event.tsOffsetMs + ":" + point.event.casterUuid + ":" + point.event.skillId)}
          {@const display = resolveEvent(point.event)}
          <div class="tl-tooltip-row">
            {#if display.iconPath}
              <img src={display.iconPath} alt="" class="tl-tooltip-icon" />
            {/if}
            <span class="tl-tooltip-name">{display.name}</span>
          </div>
          {#if display.casterName}
            <div class="tl-tooltip-sub">{display.casterName}</div>
          {/if}
        {/each}
      {:else if hoverCurveInfo}
        {#if hoverCurveInfo.instant !== null}
          <div class="tl-tooltip-row">
            <span class="tl-tooltip-dot"></span>
            {t("history.timeline.series.instant")}:
            <b>{formatCurveValue(hoverCurveInfo.instant)}</b>
          </div>
        {/if}
        {#if hoverCurveInfo.average !== null}
          <div class="tl-tooltip-row">
            <span class="tl-tooltip-dot tl-tooltip-dot--muted"></span>
            {t("history.timeline.series.average")}:
            <b>{formatCurveValue(hoverCurveInfo.average)}</b>
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .tl-crosshair {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(148, 163, 184, 0.35);
    transform: translateX(-0.5px);
  }

  .tl-brush-rect {
    position: absolute;
    top: 0;
    bottom: 0;
    background: rgba(96, 165, 250, 0.08);
    border-left: 1px solid rgba(96, 165, 250, 0.55);
    border-right: 1px solid rgba(96, 165, 250, 0.55);
  }

  .tl-selection-rect {
    position: absolute;
    top: 0;
    bottom: 0;
    background: rgba(96, 165, 250, 0.06);
    border-left: 1px solid rgba(96, 165, 250, 0.4);
    border-right: 1px solid rgba(96, 165, 250, 0.4);
  }

  .tl-tooltip {
    position: absolute;
    z-index: 30;
    min-width: 120px;
    max-width: 220px;
    transform: translate(12px, -50%);
    background: var(--tl-tooltip-bg);
    border: 1px solid var(--tl-tooltip-border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 11px;
    color: var(--tl-fg);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  }

  .tl-tooltip--flip {
    transform: translate(calc(-100% - 12px), -50%);
  }

  .tl-tooltip-time {
    font-weight: 600;
    margin-bottom: 2px;
  }

  .tl-tooltip-row {
    display: flex;
    align-items: center;
    gap: 4px;
    line-height: 1.5;
  }

  .tl-tooltip-sub {
    color: var(--tl-fg-muted);
    font-size: 10px;
    margin-bottom: 2px;
  }

  .tl-tooltip-icon {
    /* Bigger than the in-lane marker (which shrinks/hides at low zoom) so
       the tooltip stays a reliable fallback for "what skill is this" even
       when the timeline itself is too dense to show icons. */
    width: 32px;
    height: 32px;
    border-radius: 4px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .tl-tooltip-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--tl-mine);
  }

  .tl-tooltip-dot--muted {
    background: var(--tl-average);
  }
</style>
