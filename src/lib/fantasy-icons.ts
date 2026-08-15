/**
 * @file Resolves a fantasy (resonance echo) cast's normalized resonance skill
 * id to a display icon and name.
 *
 * The backend normalizes the summon marker's source config id (or the
 * summoned monster id, via the curated meter-data table) into a resonance
 * skill id before publishing `TeammateFantasyState.resonanceSkillId` or
 * recording a history timeline marker, so the frontend never has to guess
 * which id space a value belongs to. Unresolvable casts fall back to
 * `FANTASY_PLACEHOLDER_ICON_PATH`, since showing a wrong icon is worse than
 * showing a generic one.
 */
import { resolveMonsterName } from "$lib/config/game-names";
import { findResonanceSkill } from "$lib/skill-mappings";

/** Generic stand-in used whenever a cast has no resolved resonance skill. */
export const FANTASY_PLACEHOLDER_ICON_PATH =
  "/images/resonance_skill/skill_aoyi_skill_icon_053.png";

export type FantasyIconInfo = {
  /** The resonance skill id this cast was resolved to, if known. */
  skillId: number | null;
  iconPath: string;
  isPlaceholder: boolean;
};

export function resolveFantasyIcon(
  resonanceSkillId: number | null,
): FantasyIconInfo {
  const skill =
    resonanceSkillId === null
      ? undefined
      : findResonanceSkill(resonanceSkillId);
  if (skill) {
    return {
      skillId: skill.skillId,
      iconPath: skill.imagePath,
      isPlaceholder: false,
    };
  }
  return {
    skillId: null,
    iconPath: FANTASY_PLACEHOLDER_ICON_PATH,
    isPlaceholder: true,
  };
}

function stripFantasySuffix(name: string): string {
  const separatorIndex = name.indexOf("-");
  return (
    (separatorIndex >= 0 ? name.slice(0, separatorIndex) : name).trim() || name
  );
}

/** A short display name for a fantasy cast, for tooltips/labels. */
export function resolveFantasyDisplayName(
  resonanceSkillId: number | null,
  monsterId: number,
): string {
  const skillName =
    resonanceSkillId === null
      ? undefined
      : findResonanceSkill(resonanceSkillId)?.name;
  return skillName ?? stripFantasySuffix(resolveMonsterName(monsterId));
}
