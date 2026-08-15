import { describe, expect, it } from "vitest";
import {
  clampViewportWindow,
  dedupeMarkersByPixel,
  downsampleCurve,
  foldEncounterDamageBuckets,
  interpolateCurveValue,
  laneMarkerTier,
  normalizeEncounterBrushRange,
  panViewportWindow,
  sliceLanePointsByTime,
  timeToX,
  toCumulativeDpsCurve,
  toRollingDpsCurve,
  windowMaxValue,
  xToTime,
  zoomTierFor,
  zoomViewportWindow,
  type EncounterChart,
  type EncounterChartSeries,
} from "./timeline-data";

function series(
  values: Partial<EncounterChartSeries> &
    Pick<EncounterChartSeries, "entityUuid" | "metric">,
): EncounterChartSeries {
  const offsetsMs = values.offsetsMs ?? [];
  const zeros = offsetsMs.map(() => 0);
  return {
    entityUuid: values.entityUuid,
    metric: values.metric,
    offsetsMs,
    totals: values.totals ?? zeros,
  };
}

describe("foldEncounterDamageBuckets", () => {
  it("sums sparse damage series per entity and ignores heal/taken rows", () => {
    const chart: EncounterChart = {
      durationMs: 1_500,
      bucketMs: 1_000,
      series: [
        series({
          entityUuid: "a",
          metric: 0,
          offsetsMs: [0, 1_000],
          totals: [100, 100],
        }),
        series({
          entityUuid: "a",
          metric: 0,
          offsetsMs: [0],
          totals: [50],
        }),
        series({
          entityUuid: "b",
          metric: 0,
          offsetsMs: [0],
          totals: [25],
        }),
        series({
          entityUuid: "a",
          metric: 1,
          offsetsMs: [0],
          totals: [40],
        }),
        series({
          entityUuid: "a",
          metric: 2,
          offsetsMs: [0],
          totals: [30],
        }),
      ],
    };

    const result = foldEncounterDamageBuckets(chart);

    expect(result.durationMs).toBe(1_500);
    expect(result.bucketMs).toBe(1_000);
    expect(result.perEntityBuckets.get("a")).toEqual([150, 100]);
    expect(result.perEntityBuckets.get("b")).toEqual([25, 0]);
  });

  it("fills absent buckets with zeroes", () => {
    const result = foldEncounterDamageBuckets({
      durationMs: 3_000,
      bucketMs: 1_000,
      series: [
        series({
          entityUuid: "a",
          metric: 0,
          offsetsMs: [1_000],
          totals: [100],
        }),
      ],
    });

    expect(result.perEntityBuckets.get("a")).toEqual([0, 100, 0]);
  });

  it("drops out-of-range and non-finite samples", () => {
    const result = foldEncounterDamageBuckets({
      durationMs: 2_000,
      bucketMs: 1_000,
      series: [
        series({
          entityUuid: "a",
          metric: 0,
          offsetsMs: [-1, 0, 2_000, Number.NaN],
          totals: [1, 100, 2, Number.NaN],
        }),
      ],
    });

    expect(result.perEntityBuckets.get("a")).toEqual([100, 0]);
  });
});

describe("toRollingDpsCurve", () => {
  it("uses the actual bucket count as divisor while the window is filling", () => {
    // 1s buckets, 10s window: the first 9 points divide by (index + 1)
    // buckets, not by 10.
    const totals = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    const curve = toRollingDpsCurve(totals, 1_000, 12_000);

    expect(curve[0]).toEqual([1_000, 100]);
    expect(curve[1]).toEqual([2_000, 50]);
    expect(curve[9]).toEqual([10_000, 10]);
  });

  it("evicts damage once it leaves the trailing window", () => {
    const totals = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    const curve = toRollingDpsCurve(totals, 1_000, 12_000);

    // The window is left-closed right-open: at 10s it covers [0s, 10s) and
    // still contains the 0s hit; at 11s it covers [1s, 11s) and the hit is
    // gone.
    expect(curve[9]).toEqual([10_000, 10]);
    expect(curve[10]).toEqual([11_000, 0]);
  });

  it("sums concurrent hits inside the window", () => {
    const totals = [0, 0, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0];
    const curve = toRollingDpsCurve(totals, 1_000, 12_000);

    expect(curve[9]).toEqual([10_000, 30]);
  });

  it("clamps the final point to durationMs", () => {
    // 2.5s duration with 1s buckets: the third bucket only covers 0.5s of
    // wall-clock time, but the window divisor stays bucket-based.
    const curve = toRollingDpsCurve([0, 0, 100], 1_000, 2_500);

    expect(curve[2]?.[0]).toBe(2_500);
  });
});

describe("toCumulativeDpsCurve", () => {
  it("divides the running total by elapsed time", () => {
    const curve = toCumulativeDpsCurve([100, 0, 300], 1_000, 3_000);

    expect(curve).toEqual([
      [1_000, 100],
      [2_000, 50],
      [3_000, 400 / 3],
    ]);
  });

  it("ends at total damage over durationMs", () => {
    const totals = [10, 20, 30, 40];
    const curve = toCumulativeDpsCurve(totals, 1_000, 4_000);

    expect(curve.at(-1)).toEqual([4_000, 25]);
  });
});

describe("rolling vs cumulative", () => {
  it("coincide when the fight is shorter than the rolling window", () => {
    // 5s fight, 1s buckets: the 10s window never fills, so both curves divide
    // the same running sum by the same covered time.
    const totals = [100, 50, 0, 200, 0];
    const rolling = toRollingDpsCurve(totals, 1_000, 5_000);
    const cumulative = toCumulativeDpsCurve(totals, 1_000, 5_000);

    expect(rolling).toEqual(cumulative);
  });
});

describe("normalizeEncounterBrushRange", () => {
  it("returns a clamped half-open integer range", () => {
    expect(normalizeEncounterBrushRange([120.2, 980.1], 1_000)).toEqual([
      120, 981,
    ]);
    expect(normalizeEncounterBrushRange([900.8, 100.2], 1_000)).toEqual([
      100, 901,
    ]);
    expect(normalizeEncounterBrushRange([-20, 2_000], 1_000)).toEqual([
      0, 1_000,
    ]);
  });

  it("keeps a zero-width brush queryable and rejects non-finite input", () => {
    expect(normalizeEncounterBrushRange([500, 500], 1_000)).toEqual([500, 501]);
    expect(normalizeEncounterBrushRange([Number.NaN, 500], 1_000)).toBeNull();
  });
});

describe("clampViewportWindow", () => {
  it("clamps a window fully inside the encounter unchanged", () => {
    expect(clampViewportWindow(1_000, 2_000, 10_000)).toEqual({
      startMs: 1_000,
      endMs: 2_000,
    });
  });

  it("pins the start so the window never exceeds the encounter bounds", () => {
    expect(clampViewportWindow(-500, 500, 10_000)).toEqual({
      startMs: 0,
      endMs: 1_000,
    });
    expect(clampViewportWindow(9_500, 10_500, 10_000)).toEqual({
      startMs: 9_000,
      endMs: 10_000,
    });
  });

  it("enforces a minimum span instead of collapsing to zero", () => {
    expect(clampViewportWindow(500, 500, 10_000, 200)).toEqual({
      startMs: 500,
      endMs: 700,
    });
  });

  it("caps the span to the full duration when it is shorter than minSpanMs", () => {
    expect(clampViewportWindow(-100, 5_000, 3_000, 1_000)).toEqual({
      startMs: 0,
      endMs: 3_000,
    });
  });
});

describe("zoomViewportWindow", () => {
  it("keeps the anchor's relative position stable while zooming in", () => {
    // Window [0, 10_000), anchor at 8_000 (80% across); zooming in 2x should
    // halve the span while keeping the anchor at the same 80% position.
    const next = zoomViewportWindow(
      { startMs: 0, endMs: 10_000 },
      2,
      8_000,
      10_000,
    );

    expect(next.endMs - next.startMs).toBe(5_000);
    const ratio = (8_000 - next.startMs) / (next.endMs - next.startMs);
    expect(ratio).toBeCloseTo(0.8, 5);
  });

  it("zooming out never exceeds the full encounter duration", () => {
    const next = zoomViewportWindow(
      { startMs: 4_000, endMs: 6_000 },
      0.1,
      5_000,
      10_000,
    );

    expect(next).toEqual({ startMs: 0, endMs: 10_000 });
  });
});

describe("panViewportWindow", () => {
  it("shifts both edges by the same delta", () => {
    expect(
      panViewportWindow({ startMs: 2_000, endMs: 4_000 }, 500, 10_000),
    ).toEqual({ startMs: 2_500, endMs: 4_500 });
  });

  it("stops at the encounter bounds without changing the span", () => {
    expect(
      panViewportWindow({ startMs: 8_000, endMs: 9_500 }, 5_000, 10_000),
    ).toEqual({ startMs: 8_500, endMs: 10_000 });
    expect(
      panViewportWindow({ startMs: 500, endMs: 2_000 }, -5_000, 10_000),
    ).toEqual({ startMs: 0, endMs: 1_500 });
  });
});

describe("timeToX / xToTime", () => {
  it("round-trip a value inside the window", () => {
    const window = { startMs: 1_000, endMs: 5_000 };
    const x = timeToX(3_000, window, 400);
    expect(x).toBe(200);
    expect(xToTime(x, window, 400)).toBe(3_000);
  });
});

describe("windowMaxValue", () => {
  it("only considers points inside the window", () => {
    const curve: [number, number][] = [
      [0, 10],
      [1_000, 100],
      [2_000, 5],
      [3_000, 999],
    ];
    expect(windowMaxValue(curve, 0, 2_000)).toBe(100);
  });

  it("returns 0 for an empty or missing curve", () => {
    expect(windowMaxValue(null, 0, 1_000)).toBe(0);
    expect(windowMaxValue([], 0, 1_000)).toBe(0);
  });
});

describe("interpolateCurveValue", () => {
  const curve: [number, number][] = [
    [0, 0],
    [1_000, 100],
    [2_000, 100],
  ];

  it("interpolates linearly between bracketing points", () => {
    expect(interpolateCurveValue(curve, 500)).toBe(50);
  });

  it("clamps to the first/last value outside the curve's range", () => {
    expect(interpolateCurveValue(curve, -100)).toBe(0);
    expect(interpolateCurveValue(curve, 5_000)).toBe(100);
  });

  it("returns null for an empty curve", () => {
    expect(interpolateCurveValue([], 100)).toBeNull();
  });
});

describe("downsampleCurve", () => {
  it("leaves short curves untouched", () => {
    const curve: [number, number][] = [
      [0, 1],
      [1, 2],
    ];
    expect(downsampleCurve(curve, 10)).toBe(curve);
  });

  it("bucket-averages long curves down to roughly the target size", () => {
    const curve: [number, number][] = Array.from({ length: 100 }, (_, i) => [
      i * 10,
      i,
    ]);
    const result = downsampleCurve(curve, 10);

    expect(result.length).toBe(10);
    // First bucket covers indices [0, 10) -> average value (0+...+9)/10 = 4.5.
    expect(result[0]?.[1]).toBeCloseTo(4.5, 5);
  });
});

describe("laneMarkerTier", () => {
  it("shows a label pill when there is room for one", () => {
    expect(laneMarkerTier(60, 50, 12)).toBe("label");
    expect(laneMarkerTier(50, 50, 12)).toBe("label");
  });

  it("degrades to a tick, then a dot, as room shrinks", () => {
    expect(laneMarkerTier(30, 50, 12)).toBe("tick");
    expect(laneMarkerTier(12, 50, 12)).toBe("tick");
    expect(laneMarkerTier(5, 50, 12)).toBe("dot");
  });

  it("shows a label for an isolated marker with no neighbour", () => {
    expect(laneMarkerTier(Number.POSITIVE_INFINITY, 50, 12)).toBe("label");
  });
});

describe("sliceLanePointsByTime", () => {
  const points = [0, 100, 200, 300, 400, 500].map((timeMs) => ({ timeMs }));

  it("returns the visible run padded by one point on each side", () => {
    // Padding keeps a marker straddling the viewport edge from vanishing
    // entirely instead of showing its visible half.
    expect(sliceLanePointsByTime(points, 200, 300)).toEqual([
      { timeMs: 100 },
      { timeMs: 200 },
      { timeMs: 300 },
      { timeMs: 400 },
    ]);
  });

  it("clamps the padding at both ends of the lane", () => {
    expect(sliceLanePointsByTime(points, 0, 100)).toEqual([
      { timeMs: 0 },
      { timeMs: 100 },
      { timeMs: 200 },
    ]);
    expect(sliceLanePointsByTime(points, 500, 600)).toEqual([
      { timeMs: 400 },
      { timeMs: 500 },
    ]);
  });

  it("returns everything when the window covers the whole lane", () => {
    expect(sliceLanePointsByTime(points, -50, 1_000)).toEqual(points);
  });

  it("returns the neighbouring points for a window that contains none", () => {
    expect(sliceLanePointsByTime(points, 210, 240)).toEqual([
      { timeMs: 200 },
      { timeMs: 300 },
    ]);
  });

  it("handles an empty lane", () => {
    expect(sliceLanePointsByTime([], 0, 100)).toEqual([]);
  });
});

describe("dedupeMarkersByPixel", () => {
  it("keeps every marker when they are more than a pixel apart", () => {
    const points = [0, 250, 500, 750, 1_000].map((timeMs) => ({ timeMs }));
    expect(dedupeMarkersByPixel(points, 0, 1_000, 100)).toEqual(points);
  });

  it("keeps only the last marker of a pixel column, in time order", () => {
    // 1000ms across 10px: all four land inside the same rounded pixel, so the
    // three underneath are fully covered by the one drawn last.
    const points = [500, 520, 540, 900].map((timeMs) => ({ timeMs }));
    expect(dedupeMarkersByPixel(points, 0, 1_000, 10)).toEqual([
      { timeMs: 540 },
      { timeMs: 900 },
    ]);
  });

  it("passes short or unmeasured lanes through untouched", () => {
    const single = [{ timeMs: 10 }];
    expect(dedupeMarkersByPixel(single, 0, 100, 500)).toEqual(single);
    const points = [{ timeMs: 10 }, { timeMs: 20 }];
    expect(dedupeMarkersByPixel(points, 0, 100, 0)).toEqual(points);
  });
});

describe("zoomTierFor", () => {
  it("stays at tier 0 below the first threshold", () => {
    expect(zoomTierFor(100_000, 100_000)).toBe(0);
    expect(zoomTierFor(100_000, 30_000)).toBe(0);
  });

  it("steps to tier 1 once zoomed in 4x or more", () => {
    expect(zoomTierFor(100_000, 25_000)).toBe(1);
    expect(zoomTierFor(100_000, 11_000)).toBe(1);
  });

  it("steps to tier 2 once zoomed in 10x or more", () => {
    expect(zoomTierFor(100_000, 10_000)).toBe(2);
    expect(zoomTierFor(100_000, 500)).toBe(2);
  });

  it("treats a zero/degenerate span as unzoomed", () => {
    expect(zoomTierFor(100_000, 0)).toBe(0);
  });
});
