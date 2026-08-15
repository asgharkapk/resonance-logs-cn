import { describe, expect, it } from "vitest";

import type { BuffUpdateState } from "$lib/bindings";
import {
  buildBuffTextRow,
  formatTimerText,
  getBuffRemainingMs,
  isBuffActive,
} from "./overlay-utils";

function buff(overrides: Partial<BuffUpdateState> = {}): BuffUpdateState {
  return {
    baseId: 100,
    durationMs: 1_000,
    createTimeMs: 1_000,
    layer: 1,
    sourceRemodelLevel: null,
    ...overrides,
  };
}

describe("buff display membership", () => {
  it("hides a finite buff after its estimated deadline", () => {
    const value = buff();

    expect(isBuffActive(value, 1_999)).toBe(true);
    expect(isBuffActive(value, 2_000)).toBe(false);
    expect(getBuffRemainingMs(value, 2_001)).toBe(0);
    expect(buildBuffTextRow("buff_100", "Buff", value, 2_001)).toBeNull();
    expect(formatTimerText(0)).toBe("--");
  });

  it("hides a single-layer zero-duration buff unless forced", () => {
    const value = buff({ durationMs: 0 });

    expect(isBuffActive(value, 10_000)).toBe(true);
    expect(getBuffRemainingMs(value, 10_000)).toBe(Number.POSITIVE_INFINITY);
    expect(buildBuffTextRow("buff_100", "Buff", value, 10_000)).toBeNull();

    const forced = buildBuffTextRow(
      "buff_100",
      "Buff",
      value,
      10_000,
      false,
      true,
    );
    expect(forced).toMatchObject({
      valueText: "∞",
      progressPercent: 0,
      showProgress: false,
    });
  });

  it("shows a multi-layer permanent buff with its layer count", () => {
    const value = buff({ durationMs: 0, layer: 3 });

    const row = buildBuffTextRow("buff_100", "Buff", value, 10_000);
    expect(row).toMatchObject({
      valueText: "∞",
      metaText: "x3",
      progressPercent: 0,
      showProgress: false,
    });
  });

  it("keeps placeholders visible regardless of guards", () => {
    const expired = buff();
    expect(buildBuffTextRow("buff_100", "Buff", expired, 2_001, true))
      .toMatchObject({ valueText: "--", isPlaceholder: true });

    const permanentSingleLayer = buff({ durationMs: 0 });
    expect(
      buildBuffTextRow("buff_100", "Buff", permanentSingleLayer, 10_000, true),
    ).toMatchObject({ valueText: "--", isPlaceholder: true });
  });

  it("treats only a missing object as absent", () => {
    expect(isBuffActive(undefined, 1_000)).toBe(false);
  });
});
