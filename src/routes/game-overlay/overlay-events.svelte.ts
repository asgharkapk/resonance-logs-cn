import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { untrack } from "svelte";
import {
  liveBuffsStore,
  liveStatusStore,
} from "$lib/stores/live-topics.svelte";
import { connectTopics } from "$lib/stores/live-topic-store.svelte";
import {
  getAvailableBuffDefinitions,
  type BuffDefinition,
} from "$lib/config/buff-name-table";
import {
  ensureBuffGroups,
  ensureCustomPanelGroups,
  ensureIndividualMonitorAllGroup,
  ensureOverlayPositions,
  ensureOverlaySizes,
  ensureOverlayVisibility,
  ensureTextBuffPanelStyle,
} from "./overlay-utils";
import {
  activeProfile,
  monitoredSkillDurationIds,
  selectedClassKey,
  updateActiveProfile,
} from "./overlay-profile.svelte.js";
import { overlayRuntime } from "./overlay-runtime.svelte.js";
import { findAnySkillByBaseId } from "$lib/skill-mappings";
import { reconcileSkillDurationStates } from "./skill-duration-state";
import {
  onGlobalPointerMove,
  onGlobalPointerUp,
  setEditMode,
  setOverlayWindow,
  setReferenceMode,
} from "./overlay-layout.svelte.js";
import { initOverlayClock } from "./overlay-clock.svelte.js";

export function initOverlay() {
  if (overlayRuntime.cleanup) return overlayRuntime.cleanup;
  if (typeof window === "undefined") {
    return () => {};
  }

  overlayRuntime.isMounted = true;
  overlayRuntime.isInitialized = true;
  setOverlayWindow(getCurrentWindow());

  document.documentElement.style.setProperty(
    "background",
    "transparent",
    "important",
  );
  document.body.style.setProperty("background", "transparent", "important");

  ensureActiveProfileDefaults();
  void setEditMode(false);
  loadAvailableBuffs();

  const unlistenEditToggle = listen("overlay-edit-toggle", () => {
    void setEditMode(!overlayRuntime.isEditing);
  });
  const unlistenReferenceToggle = listen<boolean>(
    "game-overlay-reference-toggle",
    (event) => {
      setReferenceMode(event.payload);
    },
  );
  const stopSkillDurationEffect = $effect.root(() => {
    $effect(() => {
      const status = liveStatusStore.data;
      if (!status) return;
      const classKey = selectedClassKey();
      // The previous map is this effect's own output; reading it tracked would
      // make every write re-run the effect forever.
      overlayRuntime.skillDurationMap = reconcileSkillDurationStates({
        current: untrack(() => overlayRuntime.skillDurationMap),
        skillCds: status.skillCds,
        monitoredSkillIds: new Set(monitoredSkillDurationIds()),
        durationMsForSkill: (skillId) =>
          findAnySkillByBaseId(classKey, skillId)?.effectDurationMs,
        fallbackStartedAtMs: Date.now(),
      });
    });
  });
  const disconnectTopics = connectTopics(liveStatusStore, liveBuffsStore);

  window.addEventListener("pointermove", onGlobalPointerMove);
  window.addEventListener("pointerup", onGlobalPointerUp);
  const cleanupClock = initOverlayClock();

  overlayRuntime.cleanup = () => {
    overlayRuntime.isMounted = false;
    overlayRuntime.isInitialized = false;
    overlayRuntime.dragState = null;
    overlayRuntime.resizeState = null;
    unlistenEditToggle.then((fn) => fn());
    unlistenReferenceToggle.then((fn) => fn());
    stopSkillDurationEffect();
    disconnectTopics();
    window.removeEventListener("pointermove", onGlobalPointerMove);
    window.removeEventListener("pointerup", onGlobalPointerUp);
    cleanupClock();
    setOverlayWindow(null);
    overlayRuntime.cleanup = null;
  };

  return overlayRuntime.cleanup;
}

function loadAvailableBuffs() {
  const next = new Map<number, BuffDefinition>();
  for (const buff of getAvailableBuffDefinitions()) {
    next.set(buff.baseId, buff);
  }
  overlayRuntime.buffDefinitions = next;
}

function ensureActiveProfileDefaults() {
  const profile = activeProfile();
  if (
    profile &&
    (!profile.overlayPositions ||
      profile.overlayPositions.skillDurationPositions === undefined ||
      !profile.overlaySizes ||
      profile.overlaySizes.skillDurationSizes === undefined ||
      !profile.overlayVisibility ||
      profile.overlayVisibility.showSkillDurationGroup === undefined ||
      !profile.buffDisplayMode ||
      !profile.buffGroups ||
      !profile.customPanelGroups ||
      profile.customPanelGroups.some((group) => !group.style || !group.kind) ||
      !profile.textBuffPanelStyle ||
      !profile.textBuffMaxVisible ||
      profile.monitoredSkillDurationIds === undefined)
  ) {
    updateActiveProfile((profile) => ({
      ...profile,
      monitoredSkillDurationIds: profile.monitoredSkillDurationIds ?? [],
      overlayPositions: ensureOverlayPositions(profile),
      overlaySizes: ensureOverlaySizes(profile),
      overlayVisibility: ensureOverlayVisibility(profile),
      buffDisplayMode: profile.buffDisplayMode ?? "individual",
      buffGroups: ensureBuffGroups(profile),
      individualMonitorAllGroup: ensureIndividualMonitorAllGroup(profile),
      customPanelGroups: ensureCustomPanelGroups(profile),
      inlineBuffEntries: [],
      textBuffPanelStyle: ensureTextBuffPanelStyle(profile),
      textBuffMaxVisible: Math.max(
        1,
        Math.min(20, profile.textBuffMaxVisible ?? 10),
      ),
    }));
  }
}
