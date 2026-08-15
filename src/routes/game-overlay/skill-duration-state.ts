import type { SkillCdState } from "$lib/api";
import type { SkillDurationState } from "./overlay-types";

type SkillDurationReconcileOptions = {
  current: ReadonlyMap<number, SkillDurationState>;
  skillCds: readonly SkillCdState[];
  monitoredSkillIds: ReadonlySet<number>;
  durationMsForSkill: (skillId: number) => number | undefined;
  fallbackStartedAtMs: number;
};

/**
 * Reconciles effect-duration timers from authoritative cooldown samples.
 * A server progress update for the same beginTime must not restart the effect.
 */
export function reconcileSkillDurationStates({
  current,
  skillCds,
  monitoredSkillIds,
  durationMsForSkill,
  fallbackStartedAtMs,
}: SkillDurationReconcileOptions): Map<number, SkillDurationState> {
  const next = new Map(current);

  for (const cd of skillCds) {
    const skillId = Math.floor(cd.skillLevelId / 100);
    if (!monitoredSkillIds.has(skillId) || cd.beginTime <= 0) continue;

    const durationMs = durationMsForSkill(skillId);
    if (!durationMs) continue;
    if (next.get(skillId)?.beginTime === cd.beginTime) continue;

    next.set(skillId, {
      skillId,
      startedAtMs: cd.receivedAt || fallbackStartedAtMs,
      durationMs,
      beginTime: cd.beginTime,
    });
  }

  for (const skillId of next.keys()) {
    if (!monitoredSkillIds.has(skillId)) next.delete(skillId);
  }

  return next;
}
