import { describe, expect, it } from "vitest";
import { computeTimelineLayout } from "./timeline-layout";

describe("computeTimelineLayout", () => {
  it("uses the base sizes at zoom tier 0", () => {
    const layout = computeTimelineLayout(3, 0);
    expect(layout.laneH).toBe(44);
    expect(layout.iconSize).toBe(24);
    expect(layout.lanesHeight).toBe(132);
  });

  it("scales laneH/iconSize up at zoom tier 1 and 2, keeping curveH fixed", () => {
    const tier0 = computeTimelineLayout(3, 0);
    const tier1 = computeTimelineLayout(3, 1);
    const tier2 = computeTimelineLayout(3, 2);

    expect(tier1.laneH).toBeGreaterThan(tier0.laneH);
    expect(tier1.iconSize).toBeGreaterThan(tier0.iconSize);
    expect(tier2.laneH).toBeGreaterThan(tier1.laneH);
    expect(tier2.iconSize).toBeGreaterThan(tier1.iconSize);

    // The curve grid itself already grows via the ECharts viewport, so its
    // slot height must not also scale with the discrete zoom tier.
    expect(tier1.curveH).toBe(tier0.curveH);
    expect(tier2.curveH).toBe(tier0.curveH);
  });

  it("defaults to zoom tier 0 when omitted", () => {
    const withDefault = computeTimelineLayout(2);
    const explicitTier0 = computeTimelineLayout(2, 0);
    expect(withDefault).toEqual(explicitTier0);
  });

  it("stacks the curve below however many lanes there are", () => {
    const two = computeTimelineLayout(2, 0);
    const five = computeTimelineLayout(5, 0);
    expect(five.lanesHeight - two.lanesHeight).toBe(3 * two.laneH);
    expect(five.curveTop - two.curveTop).toBe(3 * two.laneH);
    expect(five.totalHeight).toBe(five.curveTop + five.curveH);
  });
});
