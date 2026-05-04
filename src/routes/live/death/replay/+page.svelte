<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import {
    getDeathRecords,
    getLiveData,
  } from "$lib/stores/live-meter-store.svelte";
  import DeathReplayDetail from "$lib/components/death-replay/death-replay-detail.svelte";
  import { t } from "$lib/i18n/index.svelte";

  const playerUid = $derived(Number(page.url.searchParams.get("playerUid") ?? "-1"));
  const deathTs = $derived(Number(page.url.searchParams.get("deathTs") ?? "-1"));

  const liveData = $derived(getLiveData());
  const deathRecords = $derived(getDeathRecords());

  const record = $derived(
    deathRecords.find(
      (r) =>
        Number(r.victimUid) === playerUid &&
        Number(r.deathTimestampMs) === deathTs,
    ) ?? null,
  );
  const entity = $derived(
    liveData?.entities.find((e) => e.uid === playerUid) ?? null,
  );

  function handleFallback() {
    if (window.history.length > 1) {
      window.history.back();
    } else {
      goto("/live/death");
    }
  }
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

{#if record}
  <DeathReplayDetail
    playerName=""
    className={entity?.className ?? ""}
    classSpecName={entity?.classSpecName ?? ""}
    {record}
  />
{:else}
  <div
    class="flex h-40 items-center justify-center rounded-lg border border-dashed border-border/60 text-muted-foreground text-xs"
  >
    {t("live.death.replay.notFound")}
    <button class="ml-2 underline" onclick={handleFallback}
      >{t("live.death.replay.backToList")}</button
    >
  </div>
{/if}
