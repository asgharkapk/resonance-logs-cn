import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  BuffUpdateState,
  CounterUpdateState,
  ShieldDetailEntry,
  SkillCdState,
} from "$lib/api";
import {
  liveBuffsStore,
  liveStatusStore,
} from "$lib/stores/live-topics.svelte";
import type { BuffDefinition } from "$lib/config/buff-name-table";
import type {
  DragState,
  ResizeState,
  SkillDurationState,
} from "./overlay-types";

function latestBuffsByBaseId(buffs: BuffUpdateState[]) {
  const result = new Map<number, BuffUpdateState>();
  for (const buff of buffs) {
    const current = result.get(buff.baseId);
    if (!current || buff.createTimeMs >= current.createTimeMs) {
      result.set(buff.baseId, buff);
    }
  }
  return result;
}

const _cdMap = $derived.by(() => {
  const result = new Map<number, SkillCdState>();
  for (const cd of liveStatusStore.data?.skillCds ?? []) {
    result.set(Math.floor(cd.skillLevelId / 100), cd);
  }
  return result;
});

const _fightResMap = $derived.by(() => {
  const result = new Map<number, number>();
  for (const entry of liveStatusStore.data?.fightResource?.entries ?? []) {
    result.set(entry.id, entry.value);
  }
  return result;
});

const _buffMap = $derived.by(() =>
  latestBuffsByBaseId(liveBuffsStore.data?.localBuffs ?? []),
);

function countersByRuleId(counters: CounterUpdateState[]) {
  return new Map(counters.map((counter) => [counter.ruleId, counter]));
}

const _counterMap = $derived.by(() =>
  countersByRuleId(liveStatusStore.data?.counters ?? []),
);
const _factorCounterMap = $derived.by(() =>
  countersByRuleId(liveStatusStore.data?.factorCounters ?? []),
);
const _panelAttrMap = $derived.by(
  () =>
    new Map(
      (liveStatusStore.data?.panelAttrs ?? []).map((attr) => [
        attr.attrId,
        attr.value,
      ]),
    ),
);

export const overlayRuntime = $state({
  currentWindow: null as ReturnType<typeof getCurrentWindow> | null,
  cleanup: null as (() => void) | null,
  isInitialized: false,
  isMounted: false,
  skillDurationMap: new Map<number, SkillDurationState>(),
  buffDefinitions: new Map<number, BuffDefinition>(),
  isEditing: false,
  isReferenceMode: false,
  dragState: null as DragState | null,
  resizeState: null as ResizeState | null,
});

export function cdMap() {
  return _cdMap;
}

export function fightResMap() {
  return _fightResMap;
}

export function buffMap() {
  return _buffMap;
}

export function counterMap() {
  return _counterMap;
}

export function factorCounterMap() {
  return _factorCounterMap;
}

export function seasonCultivateFactorSourceItemIds() {
  return liveStatusStore.data?.factorSourceItemIds ?? [];
}

export function seasonCultivateFactorSlotItemIds() {
  return liveStatusStore.data?.factorSlotItemIds ?? [];
}

const _seasonActiveTemplateIds = $derived.by(
  () => new Set(liveStatusStore.data?.seasonActiveTemplateIds ?? []),
);

/** Highest deep-sleep (800522) `seasonId` resolved by the backend; `0`
 * before any season data has been observed. */
export function seasonCultivateSeasonId() {
  return liveStatusStore.data?.seasonId ?? 0;
}

/** Talent template id(s) currently equipped (`cultivateLineAreaList`).
 * Meaningful from S4 on (`seasonCultivateSeasonId() >= 4`). */
export function seasonActiveTemplateIds() {
  return _seasonActiveTemplateIds;
}

export function panelAttrMap() {
  return _panelAttrMap;
}

export function shieldDetailHp() {
  return {
    current: liveStatusStore.data?.shieldCurrentHp ?? 0,
    max: liveStatusStore.data?.shieldMaxHp ?? 0,
  };
}

export function shieldDetailEntries(): ShieldDetailEntry[] {
  return liveStatusStore.data?.shieldEntries ?? [];
}

export function buffDefinitions() {
  return overlayRuntime.buffDefinitions;
}

export function isEditing() {
  return overlayRuntime.isEditing;
}

export function isReferenceMode() {
  return overlayRuntime.isReferenceMode;
}

export function skillDurationMap() {
  return overlayRuntime.skillDurationMap;
}

// Whether to render the full layout scaffold (placeholders for configured-but-
// inactive slots). True both in this overlay's own edit mode AND when it is used
// as a reference layer beneath the monster-overlay, so empty slots stay visible.
export function isLayoutScaffold() {
  return overlayRuntime.isEditing || overlayRuntime.isReferenceMode;
}

export function dragState() {
  return overlayRuntime.dragState;
}

export function resizeState() {
  return overlayRuntime.resizeState;
}
