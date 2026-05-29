import type {
  HistoryEntityData,
  PerSourceStats,
  RawCombatStats,
} from "$lib/bindings";
import { resolveMonsterName } from "$lib/config/game-names";
import { t } from "$lib/i18n/index.svelte";

/** URL/key token for the "unknown source" bucket (backend sentinel monster id 0). */
export const UNKNOWN_SOURCE_KEY = "unknown";

/** Stable string key for a source monster, round-tripping through URL params. */
export function sourceMonsterKey(sourceMonsterId: number | null): string {
  return sourceMonsterId == null ? UNKNOWN_SOURCE_KEY : String(sourceMonsterId);
}

/** Display name for a source monster, or a localized label for the unknown bucket. */
export function resolveSourceName(sourceMonsterId: number | null): string {
  return sourceMonsterId == null
    ? t("tanked.monster.unknownSource")
    : resolveMonsterName(sourceMonsterId);
}

/** Find the per-source entry matching a monster key produced by {@link sourceMonsterKey}. */
export function findSourceByKey(
  perSource: PerSourceStats[] | null | undefined,
  monsterKey: string | null,
): PerSourceStats | null {
  if (!perSource || monsterKey == null) return null;
  return (
    perSource.find((src) =>
      monsterKey === UNKNOWN_SOURCE_KEY
        ? src.sourceMonsterId == null
        : src.sourceMonsterId === Number(monsterKey),
    ) ?? null
  );
}

function zeroCombatStats(): RawCombatStats {
  return {
    total: 0,
    effectiveTotal: 0,
    hits: 0,
    critHits: 0,
    critTotal: 0,
    luckyHits: 0,
    luckyTotal: 0,
    triggerHits: 0,
    blockHits: 0,
    luckyBlockHits: 0,
  };
}

/**
 * Build synthetic per-monster "entities" from a defender's per-source taken data,
 * so the existing player-row builders (`computePlayerRowsFromEntities` /
 * `buildHistoryPlayers`) can reuse the overview's tanked metric and column config.
 *
 * Each synthetic entity carries the source monster as its identity (name + key
 * encoded in `entityUuid`) and its `taken` stats; damage/healing are zeroed.
 */
export function buildSourceEntities(
  base: HistoryEntityData,
  perSource: PerSourceStats[] | null | undefined,
): HistoryEntityData[] {
  return (perSource ?? []).map((src) => ({
    ...base,
    entityUuid: sourceMonsterKey(src.sourceMonsterId),
    displayUid: src.sourceMonsterId ?? 0,
    name: resolveSourceName(src.sourceMonsterId),
    className: "",
    classSpecName: "",
    abilityScore: 0,
    seasonStrength: 0,
    damage: zeroCombatStats(),
    damageBossOnly: zeroCombatStats(),
    healing: zeroCombatStats(),
    taken: src.taken,
    dmgSkills: {},
    healSkills: {},
    takenSkills: src.skills,
    takenPerSource: [],
    dmgPerTarget: [],
    healPerTarget: [],
    deaths: [],
  }));
}
