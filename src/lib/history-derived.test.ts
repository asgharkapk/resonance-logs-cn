import { describe, expect, it } from "vitest";
import type {
  EncounterEntityData,
  EncounterSkillData,
  EncounterStatsData,
  HistoryMetric,
} from "$lib/bindings";
import {
  aggregateMetricStats,
  historyChartSeries,
  historyDeathEntries,
  historyEntityToRaw,
  historySkillRecord,
  historyStatsToCombat,
} from "./history-derived";

function stats(partial: Partial<EncounterStatsData> = {}): EncounterStatsData {
  return {
    total: "0",
    effectiveTotal: "0",
    hits: "0",
    criticalHits: "0",
    criticalTotal: "0",
    luckyHits: "0",
    luckyTotal: "0",
    triggerHits: "0",
    blockedHits: "0",
    luckyBlockHits: "0",
    ...partial,
  };
}

function skill(
  skillId: string,
  metric: HistoryMetric,
  statValues: Partial<EncounterStatsData> = {},
): EncounterSkillData {
  return {
    skillId,
    metric,
    property: null,
    damageMode: null,
    stats: stats(statValues),
  };
}

function entity(
  partial: Partial<EncounterEntityData> = {},
): EncounterEntityData {
  return {
    entityId: "1",
    displayUid: 1001,
    name: "player",
    classId: 1,
    classSpec: 1,
    classSpecName: "Iaido",
    abilityScore: 3200,
    seasonStrength: 15,
    monsterId: null,
    totals: {
      damage: "0",
      bossDamage: "0",
      healing: "0",
      effectiveHealing: "0",
      damageTaken: "0",
    },
    skills: [],
    damageTargets: [],
    healingTargets: [],
    takenSources: [],
    deaths: [],
    ...partial,
  };
}

describe("historyStatsToCombat", () => {
  it("renames every field without dropping precision", () => {
    const combat = historyStatsToCombat(
      stats({
        total: "99999999999999999999",
        criticalHits: "7",
        criticalTotal: "42",
        blockedHits: "3",
        luckyBlockHits: "2",
      }),
    );
    expect(combat.total).toBe("99999999999999999999");
    expect(combat.critHits).toBe("7");
    expect(combat.critTotal).toBe("42");
    expect(combat.blockHits).toBe("3");
    expect(combat.luckyBlockHits).toBe("2");
  });
});

describe("aggregateMetricStats", () => {
  it("sums only the requested metric with bigint precision", () => {
    const skills = [
      skill("1", "damage", { total: "9007199254740993", hits: "2" }),
      skill("2", "damage", { total: "9007199254740993", hits: "3" }),
      skill("3", "healing", { total: "777", hits: "9" }),
    ];
    const damage = aggregateMetricStats(skills, "damage");
    expect(damage.total).toBe("18014398509481986");
    expect(damage.hits).toBe("5");
    const healing = aggregateMetricStats(skills, "healing");
    expect(healing.total).toBe("777");
  });
});

describe("historySkillRecord", () => {
  it("keys by numeric skill id and skips other metrics / bad ids", () => {
    const record = historySkillRecord(
      [
        skill("123", "damage", { total: "10" }),
        skill("456", "healing", { total: "20" }),
        skill("not-a-number", "damage", { total: "30" }),
      ],
      "damage",
    );
    expect(Object.keys(record)).toEqual(["123"]);
    expect(record[123]?.totalValue).toBe("10");
  });
});

describe("historyEntityToRaw", () => {
  it("fills identity, per-metric aggregates, boss-only total, and breakdowns", () => {
    const raw = historyEntityToRaw(
      entity({
        totals: {
          damage: "0",
          bossDamage: "500",
          healing: "0",
          effectiveHealing: "0",
          damageTaken: "0",
        },
        skills: [
          skill("1", "damage", { total: "100", criticalHits: "2" }),
          skill("2", "damage_taken", { total: "40", blockedHits: "1" }),
        ],
        damageTargets: [
          {
            targetEntityId: "boss-1",
            targetDisplayUid: 0,
            targetName: null,
            targetMonsterId: 9001,
            isBoss: true,
            stats: stats({ total: "100" }),
            skills: [skill("1", "damage", { total: "100" })],
          },
        ],
        takenSources: [
          {
            sourceMonsterId: 9001,
            stats: stats({ total: "40", blockedHits: "1" }),
            skills: [skill("2", "damage_taken", { total: "40" })],
          },
        ],
      }),
    );
    expect(raw.entityUuid).toBe("1");
    expect(raw.className).toBe("Stormblade");
    expect(raw.classSpecName).toBe("Iaido");
    expect(raw.damage.total).toBe("100");
    expect(raw.damage.critHits).toBe("2");
    expect(raw.damageBossOnly.total).toBe("500");
    expect(raw.taken.blockHits).toBe("1");
    expect(raw.dmgPerTarget[0]?.targetMonsterId).toBe(9001);
    expect(raw.dmgPerTarget[0]?.isBoss).toBe(true);
    expect(raw.takenPerSource[0]?.taken.total).toBe("40");
    expect(raw.takenPerSource[0]?.skills[2]?.totalValue).toBe("40");
  });

  it("tolerates missing optional fields", () => {
    const raw = historyEntityToRaw(
      entity({
        name: null,
        classId: null,
        classSpec: null,
        classSpecName: null,
        abilityScore: null,
        seasonStrength: null,
      }),
    );
    expect(raw.name).toBe("");
    expect(raw.className).toBe("");
    expect(raw.classSpecName).toBe("");
    expect(raw.abilityScore).toBe(0);
  });
});

describe("historyDeathEntries", () => {
  it("keeps only player deaths that carry a replay", () => {
    const replay = {
      victimEntityUuid: "1",
      deathTimestampMs: "1000",
      recentDamages: [],
      victimBuffs: [],
      participantBuffs: [],
    };
    const entries = historyDeathEntries([
      entity({
        entityId: "1",
        deaths: [
          { offsetMs: 10, sourceEntityId: null, skillId: null, replay: null },
          { offsetMs: 20, sourceEntityId: "2", skillId: "9", replay },
        ],
      }),
      entity({ entityId: "2", monsterId: 9001, deaths: [] }),
    ]);
    expect(entries).toHaveLength(1);
    expect(entries[0]?.entityUuid).toBe("1");
    expect(entries[0]?.deaths).toEqual([replay]);
  });
});

describe("historyChartSeries", () => {
  it("maps metric names to chart indices and parses totals", () => {
    const mapped = historyChartSeries([
      { entityId: "1", metric: "damage", offsetsMs: [0], totals: ["100"] },
      { entityId: "1", metric: "damage_taken", offsetsMs: [0], totals: ["5"] },
    ]);
    expect(mapped[0]).toEqual({
      entityUuid: "1",
      metric: 0,
      offsetsMs: [0],
      totals: [100],
    });
    expect(mapped[1]?.metric).toBe(2);
  });

  it("returns an empty array for missing series", () => {
    expect(historyChartSeries(undefined)).toEqual([]);
  });
});
