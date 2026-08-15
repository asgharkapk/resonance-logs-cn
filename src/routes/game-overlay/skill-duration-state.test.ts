import { describe, expect, it } from "vitest";
import type { SkillCdState } from "$lib/api";
import type { SkillDurationState } from "./overlay-types";
import { reconcileSkillDurationStates } from "./skill-duration-state";

function cd(beginTime: number, receivedAt: number): SkillCdState {
  return {
    skillLevelId: 171_901,
    beginTime,
    duration: 10_000,
    skillCdType: 0,
    validCdTime: 0,
    receivedAt,
    calculatedDuration: 10_000,
    cdAccelerateRate: 0,
  };
}

function reconcile(
  current: ReadonlyMap<number, SkillDurationState>,
  skillCds: SkillCdState[],
) {
  return reconcileSkillDurationStates({
    current,
    skillCds,
    monitoredSkillIds: new Set([1719]),
    durationMsForSkill: () => 35_000,
    fallbackStartedAtMs: 99_000,
  });
}

describe("reconcileSkillDurationStates", () => {
  it("does not restart an effect for progress updates with the same beginTime", () => {
    const first = reconcile(new Map(), [cd(1_000, 2_000)]);
    const progressUpdate = reconcile(first, [cd(1_000, 8_000)]);

    expect(progressUpdate.get(1719)?.startedAtMs).toBe(2_000);
  });

  it("starts a new effect when beginTime changes", () => {
    const first = reconcile(new Map(), [cd(1_000, 2_000)]);
    const nextCast = reconcile(first, [cd(9_000, 10_000)]);

    expect(nextCast.get(1719)).toMatchObject({
      beginTime: 9_000,
      startedAtMs: 10_000,
    });
  });
});
