import type { LayoutShapes } from "../../scene-types";

export type ArenaPoint = {
  x: number;
  z: number;
};

// Central boss arena: world center (0, 110, 0). The X/Z center is the origin,
// so toArenaLocal is the identity for the ground plane; only Y is used to
// filter the central layer (exclude layer-1 sub-bosses at Y ~= 79).
const ARENA_CENTER = { x: 0, y: 110, z: 0 };
// Players operate only inside the cabinet ring (radius 16). The view is tightened
// to that inner circle plus a small buffer; the empty 16..40 ring is not shown.
const WORLD_HALF_X = 20;
const WORLD_HALF_Z = 20;
const CABINET_RING = 16;
const Y_HALF_RANGE = 15;
const BOSS_AREA_MARGIN = 6;

export type SceneLayout = {
  worldHalfX: number;
  worldHalfZ: number;
  rotationQuarters: number;
  shapes: LayoutShapes;
};

export function arenaLayout(): SceneLayout {
  return {
    worldHalfX: WORLD_HALF_X,
    worldHalfZ: WORLD_HALF_Z,
    rotationQuarters: 1,
    shapes: {
      lines: [],
      circles: [CABINET_RING],
      squares: [],
    },
  };
}

export function toArenaLocal(x: number, z: number): ArenaPoint {
  return {
    x: x - ARENA_CENTER.x,
    z: z - ARENA_CENTER.z,
  };
}

export function yInArena(y: number): boolean {
  return Math.abs(y - ARENA_CENTER.y) <= Y_HALF_RANGE;
}

export function inBossArea(x: number, z: number): boolean {
  const local = toArenaLocal(x, z);
  return (
    Math.abs(local.x) <= WORLD_HALF_X + BOSS_AREA_MARGIN &&
    Math.abs(local.z) <= WORLD_HALF_Z + BOSS_AREA_MARGIN
  );
}

// Pizza danger sector radii (static table missing; tunable constants). The
// sector is drawn only out to the cabinet ring — the players' play area — so it
// no longer extends into the unused outer ring.
export const PIZZA_INNER_RADIUS = CABINET_RING;
export const PIZZA_OUTER_RADIUS = CABINET_RING;
