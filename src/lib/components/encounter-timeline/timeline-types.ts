import type { EncounterTimelineEvent } from "./timeline-data";

/** Player metadata needed to build cast lanes and DPS curves. */
export type TimelinePlayerMeta = {
  entityUuid: string;
  /** Display name (already privacy-filtered by the parent). */
  name: string;
  className: string;
  classSpecName: string;
  isLocalPlayer: boolean;
};

/** Display strings/icon for one lane marker, resolved by the parent. */
export type TimelineEventDisplay = {
  name: string;
  /** null when the marker has no artwork (boss skills): rendered as a text pill. */
  iconPath: string | null;
  casterName: string;
};

/** Boss caster identity for labelling one lane per boss. */
export type TimelineBossMeta = {
  entityUuid: string;
  name: string;
};

export type LanePoint = {
  timeMs: number;
  event: EncounterTimelineEvent;
};

export type Lane =
  | { key: string; type: "boss"; name: string; points: LanePoint[] }
  | {
      key: string;
      type: "mine";
      player: TimelinePlayerMeta;
      points: LanePoint[];
    }
  | {
      key: string;
      type: "teammate";
      player: TimelinePlayerMeta;
      points: LanePoint[];
    };

/** A pointer's location while hovering the interactive plot surface, in the
 * plot's own local coordinate space (y=0 at the first lane row's top edge). */
export type TimelineHoverPoint = {
  timeMs: number;
  y: number;
};
