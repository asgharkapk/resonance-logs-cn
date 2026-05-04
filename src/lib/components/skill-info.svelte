<script lang="ts">
  /**
   * @file This component displays the name of a skill.
   */
  import { t } from "$lib/i18n/index.svelte";

  let {
    skillName,
  }: {
    skillName?: string;
  } = $props();

  const displaySkillName = $derived(
    skillName || t("components.skillInfo.unknownName"),
  );
</script>

<span class="ml-2 flex"><span class="truncate">{displaySkillName}</span></span>
