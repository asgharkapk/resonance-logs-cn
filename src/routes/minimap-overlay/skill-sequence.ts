import type { MinimapSkillCast } from "$lib/api";
import { t, type MessageKey } from "$lib/i18n/index.svelte";
import type { MechanicRow } from "./scene-types";

type SkillSequenceEntry = {
  labelKey: MessageKey;
  colorSlot: number;
};

export type SkillSequenceRule = {
  key: string;
  groupKey: MessageKey;
  slots: number;
  separator?: string;
  skills: Record<number, SkillSequenceEntry>;
  resetOnForeignSkill?: boolean;
};

type MatchedSkill = SkillSequenceEntry & {
  skillId: number;
};

export function evaluateSkillSequence(
  rule: SkillSequenceRule,
  casts: MinimapSkillCast[],
): MechanicRow[] {
  if (rule.slots <= 0) return [];

  const resetOnForeignSkill = rule.resetOnForeignSkill ?? true;
  let sequence: MatchedSkill[] = [];

  for (const cast of casts) {
    const matched = rule.skills[cast.skillId];
    if (matched) {
      sequence = [
        ...sequence,
        {
          ...matched,
          skillId: cast.skillId,
        },
      ].slice(-rule.slots);
      continue;
    }

    if (resetOnForeignSkill) {
      sequence = [];
    }
  }

  const latest = sequence.at(-1);
  if (!latest) return [];

  const label = sequence
    .map((skill) => t(skill.labelKey))
    .join(rule.separator ?? " → ");

  return [
    {
      key: `${rule.key}:${sequence.map((skill) => skill.skillId).join("-")}`,
      group: t(rule.groupKey),
      label,
      colorSlot: latest.colorSlot,
      createTimeMs: 0,
      durationMs: 0,
      targets: [],
      hideTimer: true,
    },
  ];
}
