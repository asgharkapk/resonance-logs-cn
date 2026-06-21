import type { LayoutShapes, MapLine } from "../../scene-types";

export type ArenaPoint = {
  x: number;
  z: number;
};

const ARENA_CENTER = { x: 69, y: 62.2, z: -307 };
const WORLD_HALF_X = 32;
const WORLD_HALF_Z = 30;
const BOSS_AREA_MARGIN = 4;
const Y_HALF_RANGE = 18;
const GRID_SPACING = 13.5;
const GRID_HALF = GRID_SPACING * 1.5;
const GRID_EDGES = [-GRID_HALF, -GRID_SPACING / 2, GRID_SPACING / 2, GRID_HALF];

export type SceneLayout = {
  worldHalfX: number;
  worldHalfZ: number;
  rotationQuarters: number;
  shapes: LayoutShapes;
};

const BOSS_AREA_LINES: MapLine[] = [
  ...rectLines(WORLD_HALF_X, WORLD_HALF_Z),
  ...gridLines(),
];

function rectLines(halfX: number, halfZ: number): MapLine[] {
  return [
    { x1: -halfX, z1: -halfZ, x2: halfX, z2: -halfZ },
    { x1: halfX, z1: -halfZ, x2: halfX, z2: halfZ },
    { x1: halfX, z1: halfZ, x2: -halfX, z2: halfZ },
    { x1: -halfX, z1: halfZ, x2: -halfX, z2: -halfZ },
  ];
}

function gridLines(): MapLine[] {
  const lines: MapLine[] = [];
  for (const x of GRID_EDGES) {
    lines.push({ x1: x, z1: -GRID_HALF, x2: x, z2: GRID_HALF });
  }
  for (const z of GRID_EDGES) {
    lines.push({ x1: -GRID_HALF, z1: z, x2: GRID_HALF, z2: z });
  }
  return lines;
}

export function arenaLayout(): SceneLayout {
  return {
    worldHalfX: WORLD_HALF_X,
    worldHalfZ: WORLD_HALF_Z,
    rotationQuarters: 1,
    shapes: {
      lines: BOSS_AREA_LINES,
      circles: [],
      squares: [],
    },
  };
}

export function arenaWorldRect(): ArenaPoint[] {
  return [
    { x: ARENA_CENTER.x - WORLD_HALF_X, z: ARENA_CENTER.z - WORLD_HALF_Z },
    { x: ARENA_CENTER.x + WORLD_HALF_X, z: ARENA_CENTER.z - WORLD_HALF_Z },
    { x: ARENA_CENTER.x + WORLD_HALF_X, z: ARENA_CENTER.z + WORLD_HALF_Z },
    { x: ARENA_CENTER.x - WORLD_HALF_X, z: ARENA_CENTER.z + WORLD_HALF_Z },
  ];
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
