import { describe, expect, it } from "vitest";
import {
  aggregateRawSkillStats,
  type RawSkillStatsLike,
} from "./recount-table";

function raw(values: Partial<RawSkillStatsLike>): RawSkillStatsLike {
  return {
    totalValue: "0",
    effectiveTotalValue: "0",
    hits: "0",
    critHits: "0",
    critTotalValue: "0",
    luckyHits: "0",
    luckyTotalValue: "0",
    triggerHits: "0",
    blockHits: "0",
    luckyBlockHits: "0",
    ...values,
  };
}

describe("aggregateRawSkillStats", () => {
  it("keeps group totals exact and applies trigger-hit fallback per skill", () => {
    const result = aggregateRawSkillStats([
      raw({
        totalValue: "9007199254740993",
        hits: "7",
        triggerHits: "0",
      }),
      raw({ totalValue: "11", hits: "5", triggerHits: "3" }),
    ]);

    expect(result.totalValue).toBe(9_007_199_254_741_004n);
    expect(result.hits).toBe(12n);
    expect(result.triggerHits).toBe(10n);
  });
});
