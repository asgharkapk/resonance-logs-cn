/**
 * @file Curated built-in loadout presets. Preset payloads use the same
 * versioned schema as user exports so they cannot silently drift from the
 * import/export contract.
 */
import type { AppLocale } from "$lib/i18n/locales";
import { parseLoadoutExport, type LoadoutExport } from "$lib/loadout-import";
import radiantShieldSource from "./loadout-presets/radiant-shield.json";

export type LoadoutPreset = {
  id: string;
  name: string;
  subtitle: string;
  iconPath: string;
  palette: readonly [string, string, string];
  data: LoadoutExport;
};

type PresetLabel = Pick<LoadoutPreset, "name" | "subtitle">;

const RADIANT_SHIELD_LABELS: Record<AppLocale, PresetLabel> = {
  "zh-CN": {
    name: "光盾",
    subtitle: "神盾骑士 · 光盾专精",
  },
  "en-US": {
    name: "Radiant Shield",
    subtitle: "Shield Knight · Radiant Shield",
  },
  "ja-JP": {
    name: "光盾",
    subtitle: "神盾騎士 · 光盾特化",
  },
};

const parsedRadiantShield = parseLoadoutExport(radiantShieldSource);
if (!parsedRadiantShield.success) {
  throw new Error(
    `Invalid built-in radiant shield loadout: ${parsedRadiantShield.issues.join(", ")}`,
  );
}

const RADIANT_SHIELD_DATA = parsedRadiantShield.output;

export function buildLoadoutPresets(locale: AppLocale): LoadoutPreset[] {
  return [
    {
      id: "radiant-shield",
      ...RADIANT_SHIELD_LABELS[locale],
      iconPath: "/images/class_specs/Shield.png",
      palette: ["#fde68a", "#67e8f9", "#4ade80"],
      data: RADIANT_SHIELD_DATA,
    },
  ];
}
