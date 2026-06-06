import { emit } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { Window } from "@tauri-apps/api/window";

/**
 * Cross-overlay reference orchestration.
 *
 * When one overlay window enters its edit mode, it shows the *sibling* overlay
 * aligned beneath itself as a live, 1:1, full-opacity reference layer (the
 * sibling renders its layout scaffold so even empty/inactive slots are visible).
 * The editing window stays on top; geometry is mirrored while editing; on exit
 * the sibling's prior visibility is restored (its geometry is intentionally left
 * aligned so the two windows stay coincident afterwards).
 *
 * This module is symmetric: both overlays use it to drive the other.
 */

export type ReferenceSession = {
  /** True while this overlay is actively driving its sibling as a reference. */
  active: boolean;
  /** Sibling visibility snapshot taken on enable, restored on disable. */
  siblingWasVisible: boolean;
  /** Unlisten handles for the geometry mirror (self move/resize -> sibling). */
  geometryUnlisten: (() => void)[] | null;
};

export function createReferenceSession(): ReferenceSession {
  return { active: false, siblingWasVisible: false, geometryUnlisten: null };
}

/** Align the sibling window onto this window (same screen rect). */
async function alignSiblingToSelf(self: Window, sibling: WebviewWindow) {
  const [position, size] = await Promise.all([
    self.outerPosition(),
    self.innerSize(),
  ]);
  await sibling.setPosition(position);
  await sibling.setSize(size);
}

export async function enableSiblingReference(params: {
  self: Window | null;
  siblingLabel: string;
  referenceEvent: string;
  session: ReferenceSession;
}) {
  const { self, siblingLabel, referenceEvent, session } = params;
  if (!self || session.active) return;
  try {
    const sibling = await WebviewWindow.getByLabel(siblingLabel);
    if (!sibling) return;
    // Snapshot the REAL window visibility — overlays can be toggled directly via
    // buttons (see skill-monitor / monster-monitor +layout), independent of the
    // monitor-enabled setting — so we can faithfully restore it on disable.
    session.siblingWasVisible = await sibling.isVisible();
    session.active = true;
    await alignSiblingToSelf(self, sibling);
    await sibling.show();
    await sibling.unminimize();
    await emit(referenceEvent, true);
    // Both overlays are alwaysOnTop; re-assert ours so it stays above the reference.
    await self.setAlwaysOnTop(true);
    await self.setFocus();
    const unlistenMoved = await self.onMoved(() => {
      void alignSiblingToSelf(self, sibling);
    });
    const unlistenResized = await self.onResized(() => {
      void alignSiblingToSelf(self, sibling);
    });
    session.geometryUnlisten = [unlistenMoved, unlistenResized];
  } catch (error) {
    console.error(
      `[overlay-reference] failed to enable reference for ${siblingLabel}`,
      error,
    );
  }
}

export async function disableSiblingReference(params: {
  siblingLabel: string;
  referenceEvent: string;
  session: ReferenceSession;
}) {
  const { siblingLabel, referenceEvent, session } = params;
  if (session.geometryUnlisten) {
    for (const unlisten of session.geometryUnlisten) unlisten();
    session.geometryUnlisten = null;
  }
  if (!session.active) return;
  session.active = false;
  try {
    await emit(referenceEvent, false);
    if (!session.siblingWasVisible) {
      const sibling = await WebviewWindow.getByLabel(siblingLabel);
      if (sibling) await sibling.hide();
    }
  } catch (error) {
    console.error(
      `[overlay-reference] failed to disable reference for ${siblingLabel}`,
      error,
    );
  }
}
