import { s3CursedTombScene } from "./scenes/s3-cursed-tomb";
import { s3GiantTowerScene } from "./scenes/s3-giant-tower";
import { s3RaidScene } from "./scenes/s3-raid";
import { s3SeaRingedReefScene } from "./scenes/s3-sea-ringed-reef";
import { s3TinaMindrealmScene } from "./scenes/s3-tina-mindrealm";
import type { SceneDefinition } from "./scene-types";

const SCENES: readonly SceneDefinition[] = [
  s3RaidScene,
  s3SeaRingedReefScene,
  s3CursedTombScene,
  s3GiantTowerScene,
  s3TinaMindrealmScene,
];

export function resolveScene(sceneId: number): SceneDefinition | null {
  return SCENES.find((scene) => scene.sceneIds.includes(sceneId)) ?? null;
}
