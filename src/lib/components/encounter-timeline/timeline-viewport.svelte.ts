import {
  MIN_VIEWPORT_SPAN_MS,
  clampViewportWindow,
  panViewportWindow,
  zoomViewportWindow,
} from "./timeline-data";

/**
 * Single reactive state source for the timeline's visible time window. Every
 * region (lanes, curve, minimap, header indicator) reads `startMs`/`endMs`
 * from one instance instead of each owning its own zoom/pan logic, which is
 * what made the old ECharts-only implementation impossible to keep in sync
 * (dataZoom, brush and the HTML gutter all had independent state).
 */
export class TimelineViewport {
  durationMs = $state(1);
  startMs = $state(0);
  endMs = $state(1);

  constructor(durationMs: number) {
    this.setDuration(durationMs);
  }

  readonly isZoomed = $derived(
    this.startMs > 0 || this.endMs < this.durationMs,
  );
  readonly spanMs = $derived(Math.max(1, this.endMs - this.startMs));

  /** Resets the window to the full encounter. Call when the encounter
   * identity changes (a new fight was loaded), not on every data refresh. */
  setDuration(durationMs: number) {
    const next = Math.max(MIN_VIEWPORT_SPAN_MS, Math.round(durationMs) || 1);
    this.durationMs = next;
    this.startMs = 0;
    this.endMs = next;
  }

  /** Zooms by `factor` (>1 = in, <1 = out), keeping `anchorMs` stationary. */
  zoomAt(factor: number, anchorMs: number) {
    const next = zoomViewportWindow(
      { startMs: this.startMs, endMs: this.endMs },
      factor,
      anchorMs,
      this.durationMs,
    );
    this.startMs = next.startMs;
    this.endMs = next.endMs;
  }

  /** Shifts the window by `deltaMs`, clamped to the encounter bounds. */
  panBy(deltaMs: number) {
    const next = panViewportWindow(
      { startMs: this.startMs, endMs: this.endMs },
      deltaMs,
      this.durationMs,
    );
    this.startMs = next.startMs;
    this.endMs = next.endMs;
  }

  /** Jumps straight to an explicit [start, end) window (e.g. "zoom to
   * selection" or minimap handle drags), clamped to the encounter bounds. */
  zoomToRange(range: readonly [number, number]) {
    const next = clampViewportWindow(
      range[0],
      range[1],
      this.durationMs,
      Math.min(MIN_VIEWPORT_SPAN_MS, Math.max(1, range[1] - range[0])),
    );
    this.startMs = next.startMs;
    this.endMs = next.endMs;
  }

  /** Restores the full-encounter view. */
  reset() {
    this.startMs = 0;
    this.endMs = this.durationMs;
  }
}
