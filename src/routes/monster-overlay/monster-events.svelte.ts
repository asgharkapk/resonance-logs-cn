import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  liveFantasyStore,
  liveMonsterStore,
} from "$lib/stores/live-topics.svelte";
import { connectTopics } from "$lib/stores/live-topic-store.svelte";
import {
  onGlobalPointerMove,
  onGlobalPointerUp,
  setMonsterEditMode,
  setMonsterOverlayWindow,
  setMonsterReferenceMode,
} from "./monster-layout.svelte.js";
import { updateMonsterDisplay } from "./monster-display.svelte.js";
import { monsterRuntime } from "./monster-runtime.svelte.js";

export function initMonsterOverlay() {
  if (monsterRuntime.cleanup) return monsterRuntime.cleanup;
  if (typeof window === "undefined") {
    return () => {};
  }

  monsterRuntime.isMounted = true;
  monsterRuntime.isInitialized = true;
  setMonsterOverlayWindow(getCurrentWindow());

  document.documentElement.style.setProperty(
    "background",
    "transparent",
    "important",
  );
  document.body.style.setProperty("background", "transparent", "important");

  void setMonsterEditMode(false);

  const unlistenEditToggle = listen("monster-overlay-edit-toggle", () => {
    void setMonsterEditMode(!monsterRuntime.isEditing);
  });
  const unlistenReferenceToggle = listen<boolean>(
    "monster-overlay-reference-toggle",
    (event) => {
      setMonsterReferenceMode(event.payload);
    },
  );
  const disconnectTopics = connectTopics(liveMonsterStore, liveFantasyStore);

  window.addEventListener("pointermove", onGlobalPointerMove);
  window.addEventListener("pointerup", onGlobalPointerUp);
  monsterRuntime.rafId = requestAnimationFrame(updateMonsterDisplay);

  monsterRuntime.cleanup = () => {
    monsterRuntime.isMounted = false;
    monsterRuntime.isInitialized = false;
    monsterRuntime.dragState = null;
    monsterRuntime.resizeState = null;
    monsterRuntime.bossSections = [];
    monsterRuntime.teammateColumns = [];
    monsterRuntime.teammateRows = [];
    monsterRuntime.hateSections = [];
    monsterRuntime.stunSections = [];
    monsterRuntime.fantasyRows = [];
    monsterRuntime.dbmRows = [];
    unlistenEditToggle.then((fn) => fn());
    unlistenReferenceToggle.then((fn) => fn());
    disconnectTopics();
    window.removeEventListener("pointermove", onGlobalPointerMove);
    window.removeEventListener("pointerup", onGlobalPointerUp);
    if (monsterRuntime.rafId) {
      cancelAnimationFrame(monsterRuntime.rafId);
      monsterRuntime.rafId = null;
    }
    setMonsterOverlayWindow(null);
    monsterRuntime.cleanup = null;
  };

  return monsterRuntime.cleanup;
}
