import { getLocale } from "$lib/i18n/index.svelte";

const CLASS_LABELS_ZH: Record<string, string> = {
  "Heavy Guardian": "巨刃守护者",
  "Shield Knight": "神盾骑士",
  Stormblade: "雷影剑士",
  "Wind Knight": "青岚骑士",
  Marksman: "神射手",
  "Frost Mage": "冰魔导师",
  "Flame Berserker": "赤炎狂战士",
  "Verdant Oracle": "森语者",
  "Beat Performer": "灵魂乐手",
};

const SPEC_LABELS_ZH: Record<string, string> = {
  Earthfort: "岩盾",
  Block: "格挡",
  Iaido: "太刀",
  "Iaido Slash": "太刀",
  Moonstrike: "月刃",
  Vanguard: "重装",
  Skyward: "空枪",
  Wildpack: "狼弓",
  Falconry: "鹰弓",
  Icicle: "冰矛",
  Frostbeam: "射线",
  Voidflame: "无相流",
  Blazecrimson: "赤红流",
  Smite: "惩击",
  Lifebind: "愈合",
  Recovery: "防盾",
  Shield: "光盾",
  "Light Shield": "光盾",
  Concerto: "协奏",
  Dissonance: "狂音",
};

export function toClassLabel(className: string): string {
  if (getLocale() !== "zh-CN") return className;
  return CLASS_LABELS_ZH[className] ?? className;
}

export function toSpecLabel(specName: string): string {
  if (getLocale() !== "zh-CN") return specName;
  return SPEC_LABELS_ZH[specName] ?? specName;
}

export function formatClassSpecLabel(
  className: string,
  specName?: string,
): string {
  const classLabel = toClassLabel(className);
  const specLabel = specName ? toSpecLabel(specName) : "";
  if (!classLabel && !specLabel) return "";
  if (!classLabel) return specLabel;
  if (!specLabel) return classLabel;
  return `${classLabel} - ${specLabel}`;
}
