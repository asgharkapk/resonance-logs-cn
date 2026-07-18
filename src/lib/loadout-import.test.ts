import { describe, expect, it } from "vitest";
import {
  createDefaultLiveMeterProfileData,
  createDefaultMonsterMonitorProfile,
  createDefaultSkillMonitorProfile,
  omitProfileId,
} from "./settings-store";
import { parseLoadoutExport } from "./loadout-import";

function validExport(): Record<string, unknown> {
  return {
    kind: "resonance-logs-loadout",
    version: 1,
    name: "Test",
    skillProfile: omitProfileId(createDefaultSkillMonitorProfile("Test")),
    monsterProfile: omitProfileId(createDefaultMonsterMonitorProfile("Test")),
  };
}

describe("parseLoadoutExport", () => {
  it("accepts and normalizes a valid version 1 export", () => {
    const result = parseLoadoutExport(validExport());
    expect(result.success).toBe(true);
    if (!result.success) return;
    expect(result.output.version).toBe(1);
    expect(result.output.skillProfile.selectedClass).toBe("wind_knight");
  });

  it("rejects missing and future versions", () => {
    const missing = validExport();
    delete missing["version"];
    expect(parseLoadoutExport(missing).success).toBe(false);

    const future = { ...validExport(), version: 2 };
    expect(parseLoadoutExport(future).success).toBe(false);
  });

  it("rejects missing required profile fields", () => {
    const data = validExport();
    const skill = data["skillProfile"] as Record<string, unknown>;
    delete skill["monitoredSkillIds"];
    expect(parseLoadoutExport(data).success).toBe(false);
  });

  it("rejects invalid nested values and non-finite ids", () => {
    const nested = validExport();
    const monster = nested["monsterProfile"] as Record<string, unknown>;
    monster["overlayPositions"] = { monsterBuffPanel: { x: "bad", y: 0 } };
    expect(parseLoadoutExport(nested).success).toBe(false);

    const ids = validExport();
    const skill = ids["skillProfile"] as Record<string, unknown>;
    skill["monitoredSkillIds"] = [Number.NaN];
    expect(parseLoadoutExport(ids).success).toBe(false);

    const infinite = validExport();
    const infiniteMonster = infinite["monsterProfile"] as Record<
      string,
      unknown
    >;
    infiniteMonster["hateListMaxDisplay"] = Number.POSITIVE_INFINITY;
    expect(parseLoadoutExport(infinite).success).toBe(false);
  });

  it("strips unknown fields and fills optional defaults", () => {
    const data = validExport();
    const skill = data["skillProfile"] as Record<string, unknown>;
    delete skill["buffVoiceConfigs"];
    skill["untrusted"] = "discard me";
    const overlayPositions = skill["overlayPositions"] as Record<
      string,
      unknown
    >;
    overlayPositions["untrusted"] = { x: 1, y: 2 };

    const result = parseLoadoutExport(data);
    expect(result.success).toBe(true);
    if (!result.success) return;
    expect(result.output.skillProfile.buffVoiceConfigs).toEqual({});
    expect("untrusted" in result.output.skillProfile).toBe(false);
    expect("untrusted" in result.output.skillProfile.overlayPositions).toBe(
      false,
    );
  });

  it("rejects invalid enums and dynamic record keys", () => {
    const invalidEnum = validExport();
    const enumSkill = invalidEnum["skillProfile"] as Record<string, unknown>;
    enumSkill["buffDisplayMode"] = "automatic";
    expect(parseLoadoutExport(invalidEnum).success).toBe(false);

    const invalidRecord = validExport();
    const recordSkill = invalidRecord["skillProfile"] as Record<
      string,
      unknown
    >;
    const overlaySizes = recordSkill["overlaySizes"] as Record<string, unknown>;
    overlaySizes["iconBuffSizes"] = { invalid: 1 };
    expect(parseLoadoutExport(invalidRecord).success).toBe(false);
  });

  it("rejects invalid custom-panel and monitor-all group entries", () => {
    const customPanel = validExport();
    const skill = customPanel["skillProfile"] as Record<string, unknown>;
    skill["customPanelGroups"] = [
      {
        id: "panel",
        name: "Panel",
        kind: "manual",
        entries: [
          {
            id: "entry",
            sourceType: "buff",
            sourceId: "not-a-number",
            label: "Broken",
            format: "timer",
          },
        ],
        position: { x: 0, y: 0 },
        scale: 1,
        style: {
          gap: 1,
          columnGap: 1,
          fontSize: 12,
          nameColor: "#fff",
          valueColor: "#fff",
          progressColor: "#fff",
          progressOpacity: 1,
          textShadowEnabled: true,
          backgroundEnabled: false,
          backgroundOpacity: 0,
        },
      },
    ];
    expect(parseLoadoutExport(customPanel).success).toBe(false);

    const monitorAll = validExport();
    const monitorSkill = monitorAll["skillProfile"] as Record<string, unknown>;
    monitorSkill["individualMonitorAllGroup"] = { monitorAll: true };
    expect(parseLoadoutExport(monitorAll).success).toBe(false);
  });

  it("rejects malformed nested voice bindings and record values", () => {
    const voice = validExport();
    const skill = voice["skillProfile"] as Record<string, unknown>;
    skill["buffVoiceConfigs"] = {
      "101": { gained: { enabled: true, phrase: { source: "custom" } } },
    };
    expect(parseLoadoutExport(voice).success).toBe(false);

    const aliases = validExport();
    const monster = aliases["monsterProfile"] as Record<string, unknown>;
    monster["fantasyMonsterAliases"] = { "101": 123 };
    expect(parseLoadoutExport(aliases).success).toBe(false);
  });

  it("preserves supported optional nested fields", () => {
    const data = validExport();
    const skill = data["skillProfile"] as Record<string, unknown>;
    skill["inlineBuffEntries"] = [
      {
        id: "counter",
        sourceType: "counter",
        sourceId: 101,
        counterSlotId: 2,
        hideWhenZero: true,
        label: "Counter",
        format: "timer",
      },
    ];
    skill["userCounterRules"] = [
      {
        ruleId: 101,
        name: "Rule",
        sourceRefs: [],
        slotRefs: [],
        voice: {
          "2": {
            threshold: { enabled: true, phrase: { source: "auto" } },
          },
        },
      },
    ];

    const result = parseLoadoutExport(data);
    expect(result.success).toBe(true);
    if (!result.success) return;
    expect(result.output.skillProfile.inlineBuffEntries?.[0]).toMatchObject({
      counterSlotId: 2,
      hideWhenZero: true,
    });
    expect(
      result.output.skillProfile.userCounterRules?.[0]?.voice?.["2"]?.threshold
        ?.enabled,
    ).toBe(true);
    expect(parseLoadoutExport(result.output).success).toBe(true);
  });

  it("round-trips valid buff, counter, monster, and DBM voice bindings", () => {
    const data = validExport();
    const skill = data["skillProfile"] as Record<string, unknown>;
    skill["buffVoiceConfigs"] = {
      "101": {
        gained: {
          enabled: true,
          phrase: { source: "custom", text: "Ready" },
        },
        expiring: {
          enabled: true,
          phrase: { source: "phrase", phraseId: "phrase-1" },
          secondsBefore: 2.5,
        },
      },
    };
    skill["presetCounterVoiceConfigs"] = {
      "201": {
        "1": {
          expiring: {
            enabled: true,
            phrase: { source: "auto" },
            secondsBefore: 3,
          },
        },
      },
    };
    const monster = data["monsterProfile"] as Record<string, unknown>;
    monster["dbmVoiceConfigs"] = {
      "301": {
        onCast: {
          enabled: true,
          phrase: { source: "phrase", phraseId: "phrase-2" },
        },
      },
    };
    monster["monsterBuffVoiceConfigs"] = {
      "401": {
        lost: {
          enabled: true,
          phrase: { source: "custom", text: "Gone" },
        },
      },
    };

    const parsed = parseLoadoutExport(data);
    expect(parsed.success).toBe(true);
    if (!parsed.success) return;
    const roundTrip = parseLoadoutExport(
      JSON.parse(JSON.stringify(parsed.output)),
    );
    expect(roundTrip).toEqual(parsed);
  });

  it("defaults liveProfile.challengeWatch/appearance for pre-existing exports", () => {
    // Older exports predate the challengeWatch/appearance fields entirely —
    // `liveProfile` itself may even be absent (falls back to defaults).
    const result = parseLoadoutExport(validExport());
    expect(result.success).toBe(true);
    if (!result.success) return;
    const defaults = createDefaultLiveMeterProfileData();
    expect(result.output.liveProfile.challengeWatch).toEqual(
      defaults.challengeWatch,
    );
    expect(result.output.liveProfile.appearance).toEqual(defaults.appearance);
  });

  it("preserves an explicit liveProfile.challengeWatch/appearance payload", () => {
    const data = validExport();
    data["liveProfile"] = {
      ...omitProfileId({
        ...createDefaultLiveMeterProfileData(),
        id: "unused",
        name: "Live",
      }),
      challengeWatch: { forbiddenDamageIds: [123, 456] },
      appearance: {
        ...createDefaultLiveMeterProfileData().appearance,
        classColors: { Warrior: "#abcdef" },
        useClassSpecColors: true,
      },
    };

    const result = parseLoadoutExport(data);
    expect(result.success).toBe(true);
    if (!result.success) return;
    expect(result.output.liveProfile.challengeWatch.forbiddenDamageIds).toEqual(
      [123, 456],
    );
    expect(result.output.liveProfile.appearance.classColors).toEqual({
      Warrior: "#abcdef",
    });
    expect(result.output.liveProfile.appearance.useClassSpecColors).toBe(true);
    expect(parseLoadoutExport(result.output).success).toBe(true);
  });
});
