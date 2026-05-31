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

const CLASS_LABELS_JA: Record<string, string> = {
  "Heavy Guardian": "ヘビーガーディアン",
  "Shield Knight": "シールドナイト",
  Stormblade: "ストームブレイド",
  "Wind Knight": "ウインドナイト",
  Marksman: "マークスマン",
  "Frost Mage": "フロストメイジ",
  "Flame Berserker": "フレイムバーサーカー",
  "Verdant Oracle": "ヴァーダントオラクル",
  "Beat Performer": "ビートパフォーマー",
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

const SPEC_LABELS_JA: Record<string, string> = {
  Earthfort: "アースフォート",
  Block: "ブロック",
  Iaido: "居合",
  "Iaido Slash": "居合",
  Moonstrike: "月撃",
  Vanguard: "ヴァンガード",
  Skyward: "スカイワード",
  Wildpack: "ワイルドパック",
  Falconry: "ファルコナリー",
  Icicle: "アイシクル",
  Frostbeam: "フロストビーム",
  Voidflame: "ヴォイドフレイム",
  Blazecrimson: "ブレイズクリムゾン",
  Smite: "スマイト",
  Lifebind: "ライフバインド",
  Recovery: "リカバリー",
  Shield: "シールド",
  "Light Shield": "ライトシールド",
  Concerto: "コンチェルト",
  Dissonance: "ディソナンス",
};

function getClassLabelMap() {
  const locale = getLocale();
  if (locale === "zh-CN") return CLASS_LABELS_ZH;
  if (locale === "ja-JP") return CLASS_LABELS_JA;
  return null;
}

function getSpecLabelMap() {
  const locale = getLocale();
  if (locale === "zh-CN") return SPEC_LABELS_ZH;
  if (locale === "ja-JP") return SPEC_LABELS_JA;
  return null;
}

export function toClassLabel(className: string): string {
  const labels = getClassLabelMap();
  if (!labels) return className;
  return labels[className] ?? className;
}

export function toSpecLabel(specName: string): string {
  const labels = getSpecLabelMap();
  if (!labels) return specName;
  return labels[specName] ?? specName;
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
