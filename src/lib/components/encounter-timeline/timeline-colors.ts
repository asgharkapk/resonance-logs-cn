import { getClassColorRaw } from "$lib/utils.svelte";
import { TIMELINE_PALETTE } from "./timeline-palette";
import type { Lane, TimelinePlayerMeta } from "./timeline-types";

export function playerColor(player: TimelinePlayerMeta): string {
  return player.className
    ? getClassColorRaw(player.className, player.classSpecName)
    : TIMELINE_PALETTE.fallbackPlayer;
}

export function laneColor(lane: Lane): string {
  switch (lane.type) {
    case "boss":
      return TIMELINE_PALETTE.boss;
    case "mine":
      return TIMELINE_PALETTE.mine;
    case "teammate":
      return playerColor(lane.player);
  }
}
