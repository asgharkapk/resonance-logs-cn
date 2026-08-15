import type { HistoryCastKind } from "$lib/bindings";

export type EncounterChartSeries = {
  entityUuid: string;
  metric: number;
  offsetsMs: number[];
  totals: number[];
};

export type EncounterChart = {
  durationMs: number;
  bucketMs: number;
  series: EncounterChartSeries[];
};

export type EncounterTimelineEvent = {
  tsOffsetMs: number;
  casterUuid: string;
  skillId: number;
  kind: HistoryCastKind;
};

const METRIC_DAMAGE = 0;

function positiveInteger(value: number): number {
  const numeric = Number(value);
  return Number.isFinite(numeric) && numeric > 0
    ? Math.max(1, Math.round(numeric))
    : 1;
}

export type EncounterCurvePoint = [offsetMs: number, valuePerSecond: number];

export type EncounterDamageBuckets = {
  durationMs: number;
  bucketMs: number;
  /** Per-entity damage totals by bucket index; the raw fact curves derive from. */
  perEntityBuckets: Map<string, number[]>;
};

/** Folds a sparse column-oriented chart DTO into dense per-entity damage buckets. */
export function foldEncounterDamageBuckets(
  chart: EncounterChart,
): EncounterDamageBuckets {
  const durationMs = positiveInteger(chart.durationMs);
  const bucketMs = positiveInteger(chart.bucketMs);
  const bucketCount = Math.max(1, Math.ceil(durationMs / bucketMs));
  const perEntityTotals = new Map<string, number[]>();

  for (const series of chart.series) {
    if (series.metric !== METRIC_DAMAGE) continue;

    let entityTotals = perEntityTotals.get(series.entityUuid);
    if (!entityTotals) {
      entityTotals = new Array<number>(bucketCount).fill(0);
      perEntityTotals.set(series.entityUuid, entityTotals);
    }

    for (let index = 0; index < series.offsetsMs.length; index += 1) {
      const offsetMs = Number(series.offsetsMs[index]);
      if (!Number.isFinite(offsetMs) || offsetMs < 0) continue;
      const bucketIndex = Math.floor(offsetMs / bucketMs);
      if (bucketIndex < 0 || bucketIndex >= bucketCount) continue;

      const total = Number(series.totals[index] ?? 0);
      if (!Number.isFinite(total)) continue;
      entityTotals[bucketIndex] = (entityTotals[bucketIndex] ?? 0) + total;
    }
  }

  return { durationMs, bucketMs, perEntityBuckets: perEntityTotals };
}

/** Instant-DPS trailing window length. Quantized by bucketMs (duration / 600). */
const ROLLING_WINDOW_MS = 10_000;

export function toRollingDpsCurve(
  totals: number[],
  bucketMs: number,
  durationMs: number,
): EncounterCurvePoint[] {
  const windowBuckets = Math.max(
    1,
    Math.min(totals.length, Math.round(ROLLING_WINDOW_MS / bucketMs)),
  );
  const points: EncounterCurvePoint[] = [];
  let sum = 0;
  for (let index = 0; index < totals.length; index += 1) {
    sum += totals[index] ?? 0;
    if (index >= windowBuckets) sum -= totals[index - windowBuckets] ?? 0;
    // Numerator is an integer number of buckets; use the same count as the
    // divisor so bucket-width quantization cannot skew the value. During the
    // opening ramp the window is not yet full, so dividing by the full window
    // would dilute the first 10s into a fake slope.
    const coveredMs = Math.min(index + 1, windowBuckets) * bucketMs;
    points.push([
      Math.min(durationMs, (index + 1) * bucketMs),
      (sum * 1_000) / coveredMs,
    ]);
  }
  return points;
}

export function toCumulativeDpsCurve(
  totals: number[],
  bucketMs: number,
  durationMs: number,
): EncounterCurvePoint[] {
  let sum = 0;
  return totals.map((total, index) => {
    sum += total ?? 0;
    const elapsedMs = Math.min(durationMs, (index + 1) * bucketMs);
    return [elapsedMs, (sum * 1_000) / elapsedMs];
  });
}

/** Converts a continuous brush extent into a valid half-open millisecond range. */
export function normalizeEncounterBrushRange(
  coordRange: readonly [number, number],
  durationMs: number,
): [number, number] | null {
  if (!Number.isFinite(coordRange[0]) || !Number.isFinite(coordRange[1])) {
    return null;
  }

  const normalizedDurationMs = positiveInteger(durationMs);
  const low = Math.min(coordRange[0], coordRange[1]);
  const high = Math.max(coordRange[0], coordRange[1]);
  const startMs = Math.min(
    normalizedDurationMs - 1,
    Math.max(0, Math.floor(low)),
  );
  const endMs = Math.min(
    normalizedDurationMs,
    Math.max(startMs + 1, Math.ceil(high)),
  );
  return endMs > startMs ? [startMs, endMs] : null;
}

// ---------------------------------------------------------------------------
// Viewport (zoom/pan window) math. Pure functions so the interaction layer
// (timeline-viewport.svelte.ts) and gestures (timeline-gestures.ts) stay thin
// wrappers that are easy to unit test independent of the DOM/ECharts.
// ---------------------------------------------------------------------------

export type TimelineWindow = { startMs: number; endMs: number };

/** Minimum visible span (ms): prevents zooming into a degenerate/empty window. */
export const MIN_VIEWPORT_SPAN_MS = 1_000;

/** Clamps an arbitrary [start, end) pair into a valid window inside [0, durationMs]. */
export function clampViewportWindow(
  startMs: number,
  endMs: number,
  durationMs: number,
  minSpanMs: number = MIN_VIEWPORT_SPAN_MS,
): TimelineWindow {
  const duration = Math.max(1, positiveInteger(durationMs));
  const minSpan = Math.max(1, Math.min(minSpanMs, duration));
  const rawSpan = Number.isFinite(endMs - startMs) ? endMs - startMs : duration;
  const span = Math.max(minSpan, Math.min(duration, rawSpan));
  const maxStart = Math.max(0, duration - span);
  const start = Math.min(
    maxStart,
    Math.max(0, Number.isFinite(startMs) ? startMs : 0),
  );
  return { startMs: start, endMs: Math.min(duration, start + span) };
}

/** Zooms a window by `factor` (>1 = zoom in, <1 = zoom out) around `anchorMs`,
 * keeping the anchor's relative position inside the window stable. */
export function zoomViewportWindow(
  window: TimelineWindow,
  factor: number,
  anchorMs: number,
  durationMs: number,
): TimelineWindow {
  const span = Math.max(1, window.endMs - window.startMs);
  const safeFactor = Number.isFinite(factor) && factor > 0 ? factor : 1;
  const newSpan = span / safeFactor;
  const ratio = span > 0 ? (anchorMs - window.startMs) / span : 0.5;
  const newStart = anchorMs - ratio * newSpan;
  return clampViewportWindow(newStart, newStart + newSpan, durationMs);
}

/** Pans a window by `deltaMs`, clamped so it cannot leave [0, durationMs]. */
export function panViewportWindow(
  window: TimelineWindow,
  deltaMs: number,
  durationMs: number,
): TimelineWindow {
  const span = window.endMs - window.startMs;
  return clampViewportWindow(
    window.startMs + deltaMs,
    window.startMs + deltaMs + span,
    durationMs,
  );
}

/** Maps a time value (ms) to a pixel offset within a plot of `widthPx`. */
export function timeToX(
  timeMs: number,
  window: TimelineWindow,
  widthPx: number,
): number {
  const span = Math.max(1, window.endMs - window.startMs);
  return ((timeMs - window.startMs) / span) * widthPx;
}

/** Maps a pixel offset within a plot of `widthPx` back to a time value (ms). */
export function xToTime(
  xPx: number,
  window: TimelineWindow,
  widthPx: number,
): number {
  const span = window.endMs - window.startMs;
  const width = Math.max(1, widthPx);
  return window.startMs + (xPx / width) * span;
}

/** Highest curve value inside [startMs, endMs], used to auto-scale the Y axis
 * to the visible window instead of the whole encounter. */
export function windowMaxValue(
  curve: EncounterCurvePoint[] | null | undefined,
  startMs: number,
  endMs: number,
): number {
  if (!curve || curve.length === 0) return 0;
  let max = 0;
  for (const [t, v] of curve) {
    if (t < startMs || t > endMs) continue;
    if (v > max) max = v;
  }
  return max;
}

/** Linearly interpolates a curve's value at `timeMs`; null outside its range. */
export function interpolateCurveValue(
  curve: EncounterCurvePoint[] | null | undefined,
  timeMs: number,
): number | null {
  if (!curve || curve.length === 0) return null;
  const first = curve[0];
  const last = curve[curve.length - 1];
  if (!first || !last) return null;
  if (timeMs <= first[0]) return first[1];
  if (timeMs >= last[0]) return last[1];

  // Curves are monotonically increasing in time; binary search the bracket.
  let lo = 0;
  let hi = curve.length - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    const point = curve[mid];
    if (point && point[0] <= timeMs) lo = mid;
    else hi = mid;
  }
  const a = curve[lo];
  const b = curve[hi];
  if (!a || !b) return null;
  const span = b[0] - a[0];
  if (span <= 0) return a[1];
  const ratio = (timeMs - a[0]) / span;
  return a[1] + (b[1] - a[1]) * ratio;
}

/** Downsamples a curve to roughly `targetPoints` by bucket-averaging, used by
 * the minimap sparkline so it stays cheap to render regardless of fight length. */
export function downsampleCurve(
  curve: EncounterCurvePoint[],
  targetPoints: number,
): EncounterCurvePoint[] {
  if (targetPoints <= 0 || curve.length <= targetPoints) return curve;

  const bucketSize = curve.length / targetPoints;
  const result: EncounterCurvePoint[] = [];
  for (let i = 0; i < targetPoints; i += 1) {
    const start = Math.floor(i * bucketSize);
    const end = Math.max(start + 1, Math.floor((i + 1) * bucketSize));
    let sum = 0;
    let count = 0;
    let lastT = curve[Math.min(start, curve.length - 1)]?.[0] ?? 0;
    for (let j = start; j < end && j < curve.length; j += 1) {
      const point = curve[j];
      if (!point) continue;
      sum += point[1];
      lastT = point[0];
      count += 1;
    }
    if (count > 0) result.push([lastT, sum / count]);
  }
  return result;
}

// ---------------------------------------------------------------------------
// Lane marker rendering.
//
// Markers that *have* artwork are always drawn as their skill icon, at every
// density: overlapping icons are far more useful than the abbreviated ticks
// they used to degrade into (an icon threshold of ~14px meant anything cast
// within ~3.5s of a neighbour lost its artwork on a full-fight view, which is
// most of a player's rotation). Occluded icons stay discoverable via the hover
// tooltip and by zooming in.
//
// Only markers *without* artwork (boss skills, whose DTO carries no icon) still
// degrade label pill -> tick -> dot, driven purely by the pixel gap to the
// nearest neighbour so the DOM lane renderer never needs to measure text width
// itself (CSS truncation handles the rest).
// ---------------------------------------------------------------------------

export type LaneMarkerTier = "label" | "tick" | "dot";

export function laneMarkerTier(
  availablePx: number,
  labelMinGapPx: number,
  tickMinGapPx: number,
): LaneMarkerTier {
  if (availablePx >= labelMinGapPx) return "label";
  if (availablePx >= tickMinGapPx) return "tick";
  return "dot";
}

/** Minimal shape the visibility helpers below need; both `LanePoint` and any
 * other time-stamped record satisfy it. Kept structural so this module never
 * has to import from `timeline-types` (which imports back from here). */
type TimeStamped = { timeMs: number };

/** Binary search over an ascending-by-time list. `strict` picks the bound:
 * `false` gives the first index at or after `timeMs` (the first visible point),
 * `true` the first index strictly after it (one past the last visible point).
 * Ascending order is guaranteed by the lane builder, which sorts each caster's
 * events. */
function boundByTime(
  points: readonly TimeStamped[],
  timeMs: number,
  strict: boolean,
): number {
  let lo = 0;
  let hi = points.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    const value = points[mid]?.timeMs ?? 0;
    if (strict ? value <= timeMs : value < timeMs) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** Visible slice of a time-ordered lane, padded by one point on each side so a
 * marker straddling the viewport edge still renders its visible half. Replaces
 * a full `.filter()` pass, which zoom/pan would otherwise re-run over every
 * point of every lane on each frame. */
export function sliceLanePointsByTime<T extends TimeStamped>(
  points: readonly T[],
  startMs: number,
  endMs: number,
): T[] {
  if (points.length === 0) return [];
  const first = Math.max(0, boundByTime(points, startMs, false) - 1);
  const last = Math.min(points.length, boundByTime(points, endMs, true) + 1);
  return points.slice(first, last);
}

/** Drops markers that land on a pixel column already claimed by a later one.
 *
 * Lane markers are painted in DOM order, so the later marker of an overlapping
 * pair sits on top with an opaque background - a marker sharing its rounded
 * pixel is therefore already fully hidden, and removing it is visually lossless
 * (at most a sub-pixel sliver). The point is to bound the node count of a lane
 * by the plot's pixel width instead of by the fight's cast count, which matters
 * on long encounters now that nothing degrades away. At normal density (icons
 * more than a pixel apart) this returns the input untouched. */
export function dedupeMarkersByPixel<T extends TimeStamped>(
  points: readonly T[],
  startMs: number,
  endMs: number,
  plotWidthPx: number,
): T[] {
  if (points.length < 2 || plotWidthPx <= 0) return points as T[];
  const spanMs = Math.max(1, endMs - startMs);
  const kept: T[] = [];
  let lastKeptPx = Number.POSITIVE_INFINITY;
  // Walk backwards so the marker that wins a pixel column is the one that is
  // actually drawn on top.
  for (let index = points.length - 1; index >= 0; index -= 1) {
    const point = points[index];
    if (!point) continue;
    const x = Math.round(((point.timeMs - startMs) / spanMs) * plotWidthPx);
    if (lastKeptPx - x < 1) continue;
    lastKeptPx = x;
    kept.push(point);
  }
  kept.reverse();
  return kept;
}

// ---------------------------------------------------------------------------
// Zoom tier: discrete step (rather than continuous scaling) so lane row
// height / icon size only re-layout when the viewport crosses a threshold,
// instead of reflowing on every wheel tick. See timeline-layout.ts, which
// consumes this to scale `laneH`/`iconSize`.
// ---------------------------------------------------------------------------

export type ZoomTier = 0 | 1 | 2;

const ZOOM_TIER_2_RATIO = 10;
const ZOOM_TIER_1_RATIO = 4;

/** Derives the discrete zoom tier from how much the visible span has been
 * zoomed in relative to the full encounter duration. */
export function zoomTierFor(durationMs: number, spanMs: number): ZoomTier {
  const ratio = spanMs > 0 ? durationMs / spanMs : 1;
  if (ratio >= ZOOM_TIER_2_RATIO) return 2;
  if (ratio >= ZOOM_TIER_1_RATIO) return 1;
  return 0;
}
