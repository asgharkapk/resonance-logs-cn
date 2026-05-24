<script lang="ts">
  import { SETTINGS } from "$lib/settings-store";
  import { t } from "$lib/i18n/index.svelte";
  import type { GhostArea } from "./monster-types";
  import {
    getMonsterOverlayPositions,
    getMonsterOverlaySizes,
    getMonsterOverlayVisibility,
  } from "./monster-state.svelte.js";

  const ghostAreas = $derived.by(() => {
    const positions = getMonsterOverlayPositions();
    const sizes = getMonsterOverlaySizes();
    const visibility = getMonsterOverlayVisibility();
    const next: GhostArea[] = [];

    const pushArea = (
      id: string,
      label: string,
      x: number,
      y: number,
      width: number,
      height: number,
      scale = 1,
    ) => {
      next.push({ id, label, x, y, width, height, scale });
    };

    if (visibility.showMonsterBuffPanel) {
      pushArea(
        "monsterBuffPanel",
        t("monsterOverlay.buffGroupTag"),
        positions.monsterBuffPanel.x,
        positions.monsterBuffPanel.y,
        280,
        130,
        sizes.monsterBuffPanelScale,
      );
    }

    if (visibility.showTeammateBuffPanel) {
      pushArea(
        "teammateBuffPanel",
        t("monsterOverlay.teammateGroupTag"),
        positions.teammateBuffPanel.x,
        positions.teammateBuffPanel.y,
        420,
        180,
        sizes.teammateBuffPanelScale,
      );
    }

    if (
      SETTINGS.monsterMonitor.state.hateListEnabled &&
      visibility.showHatePanel
    ) {
      pushArea(
        "hatePanel",
        t("monsterOverlay.hateGroupTag"),
        positions.hatePanel.x,
        positions.hatePanel.y,
        260,
        120,
        sizes.hatePanelScale,
      );
    }

    return next;
  });
</script>

<div class="ghost-reference" style:pointer-events="none">
  {#each ghostAreas as area (area.id)}
    <div
      class="ghost-area"
      style:left={`${area.x}px`}
      style:top={`${area.y}px`}
      style:width={`${area.width}px`}
      style:height={`${area.height}px`}
      style:transform={`scale(${area.scale})`}
      style:transform-origin="top left"
    >
      <span class="ghost-label">{area.label}</span>
    </div>
  {/each}
</div>

<style>
  .ghost-reference {
    position: absolute;
    inset: 0;
    z-index: 10;
  }

  .ghost-area {
    position: absolute;
    border: 2px dashed rgba(148, 163, 184, 0.75);
    border-radius: 10px;
    background: rgba(148, 163, 184, 0.14);
    box-shadow: inset 0 0 0 1px rgba(226, 232, 240, 0.15);
  }

  .ghost-label {
    position: absolute;
    top: -20px;
    left: 0;
    padding: 2px 6px;
    border-radius: 6px;
    background: rgba(51, 65, 85, 0.8);
    color: rgba(241, 245, 249, 0.95);
    font-size: 11px;
    font-weight: 700;
    white-space: nowrap;
  }
</style>
