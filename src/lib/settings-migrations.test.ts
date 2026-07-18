import { describe, expect, it } from "vitest";
import {
  createDefaultLoadoutsState,
  createDefaultMonitoringSettingsState,
  createDefaultMonsterMonitorProfile,
  createDefaultSkillMonitorProfile,
} from "./settings-store";
import {
  CURRENT_MONITORING_SCHEMA_VERSION,
  canAttemptMonitoringRecovery,
  isMonitoringRecoveryAuthority,
  migrateLegacyMonitoringState,
  reconcileMonitoringState,
} from "./settings-migrations";

describe("monitoring recovery coordination", () => {
  it("allows only the main window to coordinate recovery", () => {
    expect(isMonitoringRecoveryAuthority("main")).toBe(true);
    expect(isMonitoringRecoveryAuthority("skill-monitor-overlay")).toBe(false);
    expect(isMonitoringRecoveryAuthority("monster-monitor-overlay")).toBe(
      false,
    );
  });

  it("allows at most one automatic recovery attempt per session", () => {
    expect(canAttemptMonitoringRecovery(null)).toBe(true);
    expect(canAttemptMonitoringRecovery("0")).toBe(true);
    expect(canAttemptMonitoringRecovery("1")).toBe(false);
  });
});

describe("monitoring settings reconciliation", () => {
  it("repairs invalid ids and loadout references", () => {
    const state = createDefaultMonitoringSettingsState();
    const duplicate = createDefaultSkillMonitorProfile("Duplicate");
    duplicate.id = state.skillMonitor.profiles[0]!.id;
    state.skillMonitor.profiles.push(duplicate);
    state.loadouts.items = [
      {
        id: "loadout",
        name: "Broken",
        skillProfileId: "missing",
        monsterProfileId: "missing",
        starterPlaceholder: false,
      },
    ];
    state.loadouts.activeId = "missing";

    const repaired = reconcileMonitoringState(state);
    expect(
      new Set(repaired.skillMonitor.profiles.map((item) => item.id)).size,
    ).toBe(2);
    expect(repaired.loadouts.activeId).toBe("loadout");
    expect(repaired.loadouts.items[0]!.skillProfileId).toBe(
      repaired.skillMonitor.profiles[0]!.id,
    );
    expect(repaired.loadouts.items[0]!.monsterProfileId).toBe(
      repaired.monsterMonitor.profiles[0]!.id,
    );
  });

  it("materializes the active loadout monster profile into the mirror", () => {
    const state = createDefaultMonitoringSettingsState();
    const first = state.monsterMonitor.profiles[0]!;
    const second = createDefaultMonsterMonitorProfile("Second");
    second.hateListEnabled = true;
    state.monsterMonitor.profiles.push(second);
    state.loadouts.items = [
      {
        id: "loadout",
        name: "Second",
        skillProfileId: state.skillMonitor.profiles[0]!.id,
        monsterProfileId: second.id,
        starterPlaceholder: false,
      },
    ];
    state.loadouts.activeId = "loadout";
    state.monsterMonitor.mirroredProfileId = first.id;

    const repaired = reconcileMonitoringState(state);
    expect(repaired.monsterMonitor.mirroredProfileId).toBe(second.id);
    expect(repaired.monsterMonitor.hateListEnabled).toBe(true);
  });
});

describe("legacy monitoring migration", () => {
  it("creates one loadout per legacy skill profile and preserves active index", () => {
    const base = createDefaultMonitoringSettingsState();
    const second = createDefaultSkillMonitorProfile("Second");
    base.skillMonitor.profiles.push(second);

    const migrated = migrateLegacyMonitoringState({
      skillMonitor: { ...base.skillMonitor, activeProfileIndex: 1 },
      monsterMonitor: base.monsterMonitor,
      loadouts: { ...createDefaultLoadoutsState(), schemaVersion: 0 },
    });

    expect(migrated.schemaVersion).toBe(CURRENT_MONITORING_SCHEMA_VERSION);
    expect(migrated.loadouts.items).toHaveLength(2);
    expect(migrated.loadouts.activeId).toBe(migrated.loadouts.items[1]!.id);
    expect(migrated.loadouts.firstRunPromptDismissed).toBe(true);
  });

  it("keeps the first-run prompt for untouched fresh defaults", () => {
    const base = createDefaultMonitoringSettingsState();
    const migrated = migrateLegacyMonitoringState({
      skillMonitor: { ...base.skillMonitor, activeProfileIndex: 0 },
      monsterMonitor: base.monsterMonitor,
      loadouts: { ...createDefaultLoadoutsState(), schemaVersion: 0 },
    });

    expect(migrated.loadouts.firstRunPromptDismissed).toBe(false);
    expect(migrated.loadouts.items[0]!.starterPlaceholder).toBe(true);
  });

  it.each([
    [
      "duration skill",
      (base: ReturnType<typeof createDefaultMonitoringSettingsState>) => {
        base.skillMonitor.profiles[0]!.monitoredSkillDurationIds = [101];
      },
    ],
    [
      "custom panel",
      (base: ReturnType<typeof createDefaultMonitoringSettingsState>) => {
        base.skillMonitor.profiles[0]!.customPanelGroups = [
          {
            id: "panel",
            name: "Panel",
            kind: "manual",
            entries: [],
            position: { x: 0, y: 0 },
            scale: 1,
            style: {
              gap: 6,
              columnGap: 12,
              fontSize: 14,
              nameColor: "#fff",
              valueColor: "#fff",
              progressColor: "#fff",
              progressOpacity: 0.4,
              textShadowEnabled: true,
              backgroundEnabled: false,
              backgroundOpacity: 0.76,
            },
          },
        ];
      },
    ],
    [
      "skill voice",
      (base: ReturnType<typeof createDefaultMonitoringSettingsState>) => {
        base.skillMonitor.profiles[0]!.buffVoiceConfigs = {
          "101": { gained: { enabled: true, phrase: { source: "auto" } } },
        };
      },
    ],
    [
      "skill layout",
      (base: ReturnType<typeof createDefaultMonitoringSettingsState>) => {
        base.skillMonitor.profiles[0]!.overlayPositions.skillCdGroup.x = 999;
      },
    ],
    [
      "monster config",
      (base: ReturnType<typeof createDefaultMonitoringSettingsState>) => {
        base.monsterMonitor.hateListEnabled = true;
      },
    ],
  ])("does not mark configured legacy %s as a starter", (_name, mutate) => {
    const base = createDefaultMonitoringSettingsState();
    mutate(base);
    const migrated = migrateLegacyMonitoringState({
      skillMonitor: { ...base.skillMonitor, activeProfileIndex: 0 },
      monsterMonitor: base.monsterMonitor,
      loadouts: { ...createDefaultLoadoutsState(), schemaVersion: 0 },
    });

    expect(migrated.loadouts.firstRunPromptDismissed).toBe(true);
    expect(migrated.loadouts.items[0]!.starterPlaceholder).toBe(false);
  });

  it("preserves existing unreferenced profiles during intermediate migration", () => {
    const base = createDefaultMonitoringSettingsState();
    const orphanSkill = createDefaultSkillMonitorProfile("Orphan");
    const orphanMonster = createDefaultMonsterMonitorProfile("Orphan");
    base.skillMonitor.profiles.push(orphanSkill);
    base.monsterMonitor.profiles.push(orphanMonster);
    const firstSkill = base.skillMonitor.profiles[0]!;
    const firstMonster = base.monsterMonitor.profiles[0]!;
    base.loadouts.items = [
      {
        id: "loadout",
        name: "Only",
        skillProfileId: firstSkill.id,
        monsterProfileId: firstMonster.id,
        starterPlaceholder: false,
      },
    ];
    base.loadouts.activeId = "loadout";

    const migrated = migrateLegacyMonitoringState({
      skillMonitor: base.skillMonitor,
      monsterMonitor: base.monsterMonitor,
      loadouts: { ...base.loadouts, schemaVersion: 1 },
    });

    expect(
      migrated.skillMonitor.profiles.some((item) => item.id === orphanSkill.id),
    ).toBe(true);
    expect(
      migrated.monsterMonitor.profiles.some(
        (item) => item.id === orphanMonster.id,
      ),
    ).toBe(true);
  });

  it("preserves all monster profiles when loadouts have not been created yet", () => {
    const base = createDefaultMonitoringSettingsState();
    const configured = createDefaultMonsterMonitorProfile("Configured");
    configured.dbmAliases = { "101": "Mechanic" };
    base.monsterMonitor.profiles.push(configured);

    const migrated = migrateLegacyMonitoringState({
      skillMonitor: { ...base.skillMonitor, activeProfileIndex: 0 },
      monsterMonitor: base.monsterMonitor,
      loadouts: { ...createDefaultLoadoutsState(), schemaVersion: 0 },
    });

    expect(migrated.monsterMonitor.profiles).toHaveLength(2);
    expect(
      migrated.monsterMonitor.profiles.find(
        (profile) => profile.id === configured.id,
      )?.dbmAliases,
    ).toEqual({ "101": "Mechanic" });
    expect(migrated.loadouts.items[0]!.starterPlaceholder).toBe(false);
  });
});
