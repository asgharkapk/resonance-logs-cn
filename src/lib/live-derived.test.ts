import { describe, expect, it } from "vitest";
import type { LiveDataPayload, RawCombatStats, RawEntityData } from "./api";
import { computePlayerRows } from "./live-derived";

function stats(values: Partial<RawCombatStats> = {}): RawCombatStats {
  return {
    total: "0",
    effectiveTotal: "0",
    hits: "0",
    critHits: "0",
    critTotal: "0",
    luckyHits: "0",
    luckyTotal: "0",
    triggerHits: "0",
    blockHits: "0",
    luckyBlockHits: "0",
    ...values,
  };
}

function entity(damage: RawCombatStats): RawEntityData {
  return {
    entityUuid: "player-1",
    displayUid: 1,
    name: "Player",
    classId: 1,
    classSpec: 1,
    className: "Class",
    classSpecName: "Spec",
    abilityScore: 1,
    seasonStrength: 1,
    damage,
    damageBossOnly: stats({ total: damage.total }),
    healing: stats(),
    taken: stats(),
    dmgSkills: {},
    healSkills: {},
    takenSkills: {},
    takenPerSource: [],
  };
}

function payload(damage: RawCombatStats): LiveDataPayload {
  return {
    elapsedMs: "1000",
    activeCombatTimeMs: "1000",
    fightStartTimestampMs: "1",
    totalDmg: damage.total,
    totalDmgBossOnly: damage.total,
    totalHeal: "0",
    totalEffectiveHeal: "0",
    localPlayerUuid: "player-1",
    sceneId: null,
    dungeonDifficulty: null,
    isPaused: false,
    bosses: [],
    entities: [entity(damage)],
  };
}

describe("computePlayerRows", () => {
  it("uses hits when a string zero trigger-hit count has baseline semantics", () => {
    const rows = computePlayerRows(
      payload(
        stats({
          total: "9007199254740993",
          hits: "10",
          luckyHits: "5",
          triggerHits: "0",
        }),
      ),
      "dps",
    );

    expect(rows).toHaveLength(1);
    expect(rows[0]?.luckyRate).toBe(50);
    expect(rows[0]?.dmgPct).toBe(100);
    expect(Number.isFinite(rows[0]?.dps ?? Number.NaN)).toBe(true);
  });
});
