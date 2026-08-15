import { commands, type LiveBuffsPayload, type LiveCombatPayload, type LiveDeathsPayload, type LiveFantasyPayload, type LiveMonsterPayload, type LiveScenePayload, type LiveStatusPayload } from "$lib/bindings";
import { LiveTopicStore } from "$lib/stores/live-topic-store.svelte";

export const liveCombatStore = new LiveTopicStore<LiveCombatPayload>(
  "live-combat",
  () => commands.getLiveCombat(),
);

export const liveStatusStore = new LiveTopicStore<LiveStatusPayload>(
  "live-status",
  () => commands.getLiveStatus(),
);

export const liveBuffsStore = new LiveTopicStore<LiveBuffsPayload>(
  "live-buffs",
  () => commands.getLiveBuffs(),
);

export const liveMonsterStore = new LiveTopicStore<LiveMonsterPayload>(
  "live-monster",
  () => commands.getLiveMonster(),
);

export const liveFantasyStore = new LiveTopicStore<LiveFantasyPayload>(
  "live-fantasy",
  () => commands.getLiveFantasy(),
);

export const liveDeathsStore = new LiveTopicStore<LiveDeathsPayload>(
  "live-deaths",
  () => commands.getLiveDeaths(),
);

// `main`-only: drives the daily-scene auto-hide logic for the
// game/monster/minimap overlay windows without subscribing to the far
// heavier `live-combat` cadence.
export const liveSceneStore = new LiveTopicStore<LiveScenePayload>(
  "live-scene",
  () => commands.getLiveScene(),
);
