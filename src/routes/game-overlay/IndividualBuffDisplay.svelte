<script lang="ts">
  import BuffGroupGrid from "./BuffGroupGrid.svelte";
  import IconBuffCell from "./IconBuffCell.svelte";
  import {
    getDisplayIconPosition,
    getDisplayIconSize,
    individualAllGroupBuffs,
    individualModeIconBuffs,
    individualMonitorAllGroup,
    isEditing,
    isLayoutScaffold,
    startDrag,
    startResize,
  } from "./overlay-state.svelte.js";
  import { t } from "$lib/i18n/index.svelte";

  const editing = $derived(isEditing());
  const scaffold = $derived(isLayoutScaffold());
  const individualBuffs = $derived(individualModeIconBuffs());
  const allGroup = $derived(individualMonitorAllGroup());
  const allGroupBuffs = $derived(individualAllGroupBuffs());

  function getAllGroupName(group: { name: string }): string {
    return group.name.trim() || t("skillMonitor.defaults.allBuffGroupName");
  }
</script>

{#each individualBuffs as buff, idx (buff.layoutKey ?? `buff:${buff.baseId}`)}
  {@const iconPos = getDisplayIconPosition(buff, idx)}
  {@const iconSize = getDisplayIconSize(buff)}
  <IconBuffCell
    {buff}
    {iconSize}
    standalone={true}
    editable={editing}
    left={iconPos.x}
    top={iconPos.y}
    onPointerDown={(e) =>
      startDrag(
        e,
        buff.categoryKey
          ? { kind: "categoryIcon", categoryKey: buff.categoryKey }
          : { kind: "iconBuff", baseId: buff.baseId },
        iconPos,
      )}
    onResizePointerDown={(e) =>
      startResize(
        e,
        buff.categoryKey
          ? { kind: "categoryIcon", categoryKey: buff.categoryKey }
          : { kind: "iconBuff", baseId: buff.baseId },
        iconSize,
      )}
  />
{/each}

{#if allGroup && (allGroupBuffs.length > 0 || scaffold)}
  {@const group = allGroup}
  {@const maxVisible = Math.max(1, group.columns * group.rows)}
  <BuffGroupGrid
    {group}
    buffs={allGroupBuffs.slice(0, maxVisible)}
    editable={editing}
    tagText={`${getAllGroupName(group)}${t("skillMonitor.buff.group.allSuffix")}`}
    onPointerDown={(e) => startDrag(e, { kind: "individualAllGroup" }, group.position)}
    onResizePointerDown={(e) =>
      startResize(e, { kind: "individualAllGroup" }, group.iconSize)}
  />
{/if}
