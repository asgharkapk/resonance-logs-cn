import type { Attachment } from "svelte/attachments";
import { normalizeEncounterBrushRange, xToTime } from "./timeline-data";
import type { TimelineViewport } from "./timeline-viewport.svelte";

/** Multiplicative zoom applied per wheel notch. */
const ZOOM_STEP = 1.25;
/** Screen-space drag distance (px) below which a click is treated as "clear
 * the selection" rather than a (degenerate) brush. */
const MIN_DRAG_PX = 4;

export type TimelineGestureOptions = {
  /** Always reads the live viewport; called on every gesture event so the
   * attachment never needs to re-run when the viewport's fields change. */
  getViewport: () => TimelineViewport;
  getSelectionEnabled: () => boolean;
  /** Live (possibly unordered) millisecond range while a brush drag is in
   * progress; null once released or cancelled. */
  onBrushPreview: (msRange: [number, number] | null) => void;
  /** Final, normalized selection range committed on pointer-up. */
  onSelectRange: (range: [number, number] | null) => void;
  /** Cursor location in the plot's local coordinate space, or null when the
   * pointer leaves the plot (or a pan/brush drag is in progress). */
  onHover: (point: { timeMs: number; y: number } | null) => void;
};

type DragMode = "idle" | "brush" | "pan";

/**
 * Pointer/wheel gesture surface for the timeline plot. The attached node's
 * bounding box *is* the time-axis plot rect: x in [0, rect.width) maps to
 * [viewport.startMs, viewport.endMs), y=0 is the first lane row's top edge.
 *
 * Gesture model (mirrors common timeline/DAW conventions):
 *   - wheel                    -> zoom, anchored at the cursor
 *   - left-drag                -> brush-select a range (for range recount)
 *   - shift+left-drag / middle -> pan
 *   - double-click              -> reset to the full encounter
 */
export function timelineGestures(
  options: TimelineGestureOptions,
): Attachment<HTMLElement> {
  return (node) => {
    let mode: DragMode = "idle";
    let brushStartMs = 0;
    let pointerDownClientX = 0;
    let lastPanClientX = 0;
    let activePointerId: number | null = null;

    function currentWindow() {
      const viewport = options.getViewport();
      return { startMs: viewport.startMs, endMs: viewport.endMs };
    }

    function toTimeAndY(event: PointerEvent) {
      const rect = node.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      const timeMs =
        rect.width > 0 ? xToTime(x, currentWindow(), rect.width) : 0;
      return { timeMs, y };
    }

    function onWheel(event: WheelEvent) {
      const rect = node.getBoundingClientRect();
      if (rect.width <= 0) return;
      event.preventDefault();
      const anchorMs = xToTime(
        event.clientX - rect.left,
        currentWindow(),
        rect.width,
      );
      const factor = event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
      options.getViewport().zoomAt(factor, anchorMs);
    }

    function onPointerDown(event: PointerEvent) {
      const isPanGesture =
        event.button === 1 || (event.button === 0 && event.shiftKey);
      const isBrushGesture =
        event.button === 0 && !event.shiftKey && options.getSelectionEnabled();

      if (isPanGesture) {
        mode = "pan";
        lastPanClientX = event.clientX;
      } else if (isBrushGesture) {
        mode = "brush";
        pointerDownClientX = event.clientX;
        brushStartMs = toTimeAndY(event).timeMs;
        options.onBrushPreview([brushStartMs, brushStartMs]);
      } else {
        return;
      }

      options.onHover(null);
      activePointerId = event.pointerId;
      node.setPointerCapture(activePointerId);
      event.preventDefault();
    }

    function onPointerMove(event: PointerEvent) {
      if (mode === "pan") {
        const rect = node.getBoundingClientRect();
        if (rect.width <= 0) return;
        const viewport = options.getViewport();
        const deltaPx = event.clientX - lastPanClientX;
        lastPanClientX = event.clientX;
        const spanMs = viewport.endMs - viewport.startMs;
        viewport.panBy(-(deltaPx / rect.width) * spanMs);
        return;
      }
      if (mode === "brush") {
        options.onBrushPreview([brushStartMs, toTimeAndY(event).timeMs]);
        return;
      }
      options.onHover(toTimeAndY(event));
    }

    function endDrag(event: PointerEvent) {
      if (mode === "brush") {
        const dragPx = Math.abs(event.clientX - pointerDownClientX);
        if (dragPx >= MIN_DRAG_PX) {
          const endMs = toTimeAndY(event).timeMs;
          const viewport = options.getViewport();
          options.onSelectRange(
            normalizeEncounterBrushRange(
              [brushStartMs, endMs],
              viewport.durationMs,
            ),
          );
        } else {
          // A plain click without a meaningful drag clears the selection,
          // matching the common "click empty space to deselect" convention.
          options.onSelectRange(null);
        }
        options.onBrushPreview(null);
      }
      mode = "idle";
      if (activePointerId !== null) {
        node.releasePointerCapture(activePointerId);
        activePointerId = null;
      }
    }

    function onDoubleClick() {
      options.getViewport().reset();
    }

    function onPointerLeave() {
      if (mode === "idle") options.onHover(null);
    }

    node.addEventListener("wheel", onWheel, { passive: false });
    node.addEventListener("pointerdown", onPointerDown);
    node.addEventListener("pointermove", onPointerMove);
    node.addEventListener("pointerup", endDrag);
    node.addEventListener("pointercancel", endDrag);
    node.addEventListener("dblclick", onDoubleClick);
    node.addEventListener("pointerleave", onPointerLeave);

    return () => {
      node.removeEventListener("wheel", onWheel);
      node.removeEventListener("pointerdown", onPointerDown);
      node.removeEventListener("pointermove", onPointerMove);
      node.removeEventListener("pointerup", endDrag);
      node.removeEventListener("pointercancel", endDrag);
      node.removeEventListener("dblclick", onDoubleClick);
      node.removeEventListener("pointerleave", onPointerLeave);
    };
  };
}
