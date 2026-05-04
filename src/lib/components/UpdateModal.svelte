<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t } from "$lib/i18n/index.svelte";
  import MarkdownContent from "./MarkdownContent.svelte";

  export interface UpdateInfo {
    version: string;
    body: string;
    downloadUrl: string;
  }

  let {
    info,
    currentVersion,
    onclose,
  }: {
    info: UpdateInfo;
    currentVersion: string;
    onclose?: () => void;
  } = $props();

  function close() {
    onclose?.();
  }

  async function openDownloadPage() {
    try {
      await openUrl(info.downloadUrl);
    } catch (err) {
      console.error("Failed to open update URL:", info.downloadUrl, err);
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center">
  <button
    class="absolute inset-0 bg-black/60 backdrop-blur-[2px]"
    onclick={close}
    type="button"
    aria-label={t("components.updateModal.closeAria")}
  >
  </button>

  <div
    class="border-border bg-card relative z-10 flex h-[85vh] w-[90vw] max-w-3xl flex-col overflow-hidden rounded-xl border shadow-2xl"
  >
    <div
      class="border-border flex items-center justify-between border-b px-6 py-4"
    >
      <div>
        <h2 class="text-xl font-semibold">
          {t("components.updateModal.title")}
        </h2>
        <p class="text-muted-foreground mt-1 text-sm">
          {t("components.updateModal.versionLine", {
            currentVersion,
            latestVersion: info.version,
          })}
        </p>
      </div>
      <button
        class="text-muted-foreground hover:bg-muted hover:text-foreground rounded-md p-2 transition-colors"
        type="button"
        onclick={close}
        aria-label={t("components.updateModal.closeAria")}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <div class="flex-1 overflow-auto px-6 py-6">
      <MarkdownContent
        source={info.body}
        emptyText={t("components.updateModal.empty")}
      />
    </div>

    <div class="border-border space-y-3 border-t px-6 py-4">
      <p class="text-muted-foreground text-sm">
        {t("components.updateModal.footer")}
      </p>
      <div class="flex items-center justify-end gap-2">
        <button
          type="button"
          class="border-border hover:bg-muted rounded-md border px-3 py-2 text-sm transition-colors"
          onclick={close}
        >
          {t("components.dialog.close")}
        </button>
        <button
          type="button"
          class="bg-primary text-primary-foreground rounded-md px-3 py-2 text-sm font-medium transition-opacity hover:opacity-90"
          onclick={openDownloadPage}
        >
          {t("components.updateModal.download")}
        </button>
      </div>
    </div>
  </div>
</div>
