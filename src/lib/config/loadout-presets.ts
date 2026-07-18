/**
 * @file Built-in "starter" loadout presets, one per class. Each preset
 * bundles a skill-monitor profile (pre-selected class, otherwise blank) and
 * a default monster-monitor profile, so a brand-new user can pick their
 * class and get a reasonable starting point instead of an empty monitor.
 *
 * The skill selections themselves are intentionally left at the defaults
 * (no monitored skills/buffs pre-picked) - fine-tuning which skills/buffs
 * matter most per class is expected to happen by hand later.
 */
import { getClassConfigs } from "$lib/skill-mappings";
import type { AppLocale } from "$lib/i18n/locales";
import {
  createDefaultMonsterMonitorProfile,
  createDefaultSkillMonitorProfile,
  omitProfileId,
  type MonsterMonitorProfile,
  type SkillMonitorProfile,
} from "$lib/settings-store";

export type LoadoutPreset = {
  classKey: string;
  className: string;
  skillProfile: Omit<SkillMonitorProfile, "id">;
  monsterProfile: Omit<MonsterMonitorProfile, "id">;
};

export function buildLoadoutPresets(locale: AppLocale): LoadoutPreset[] {
  return getClassConfigs(locale).map((config) => {
    const skillProfile = omitProfileId(
      createDefaultSkillMonitorProfile(config.className, config.classKey),
    );
    const monsterProfile = omitProfileId(
      createDefaultMonsterMonitorProfile(config.className),
    );
    return {
      classKey: config.classKey,
      className: config.className,
      skillProfile,
      monsterProfile,
    };
  });
}
