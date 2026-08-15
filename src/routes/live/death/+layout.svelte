<script lang="ts">
  /**
   * @file Scopes the `live-deaths` topic connection to the death route
   * subtree. `connectTopics` refcounts consumers, so leaving `/live/death/*`
   * automatically disconnects the topic instead of keeping it live for the
   * whole `live` window.
   */
  import { onMount } from "svelte";
  import { liveDeathsStore } from "$lib/stores/live-topics.svelte";
  import { connectTopics } from "$lib/stores/live-topic-store.svelte";

  let { children } = $props();

  onMount(() => {
    const disconnectTopics = connectTopics(liveDeathsStore);
    return () => {
      disconnectTopics();
    };
  });
</script>

{@render children()}
