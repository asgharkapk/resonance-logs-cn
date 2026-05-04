<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { save } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n/index.svelte";
  import { toast } from "svelte-sonner";

  async function openLogDir() {
    try {
      await invoke("open_log_dir");
    } catch (e) {
      console.error(e);
      toast.error(
        t("settings.debug.toast.openLogDirFailed", {
          error: String(e),
        }),
      );
    }
  }

  async function createDiagnosticsBundle() {
    try {
      const ts = new Date();
      const pad = (n: number) => n.toString().padStart(2, "0");
      const defaultName = `debug_${ts.getFullYear()}-${pad(ts.getMonth() + 1)}-${pad(ts.getDate())}_${pad(ts.getHours())}-${pad(ts.getMinutes())}-${pad(ts.getSeconds())}.zip`;

      const destinationPath = await save({
        title: t("settings.debug.dialog.saveDiagnosticsTitle"),
        defaultPath: defaultName,
        filters: [
          { name: t("settings.debug.dialog.zipFilter"), extensions: ["zip"] },
        ],
      });

      if (!destinationPath) {
        return;
      }

      const path = await invoke<string>("create_diagnostics_bundle", {
        destination_path: destinationPath,
      });
      try {
        await navigator.clipboard.writeText(path);
        toast.success(
          t("settings.debug.toast.diagnosticsCreatedCopied", {
            path,
          }),
        );
      } catch {
        toast.success(
          t("settings.debug.toast.diagnosticsCreated", {
            path,
          }),
        );
      }
    } catch (e) {
      console.error(e);
      toast.error(
        t("settings.debug.toast.diagnosticsCreateFailed", {
          error: String(e),
        }),
      );
    }
  }
</script>

<div class="space-y-3">
  <div
    class="border-border/60 bg-card/40 overflow-hidden rounded-lg border shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
  >
    <div class="px-4 py-3">
      <h2 class="text-foreground mb-4 text-base font-semibold">
        {t("settings.debug.title")}
      </h2>

      <div class="flex items-center justify-between">
        <div class="text-muted-foreground text-sm">
          <div class="text-foreground font-medium">
            {t("settings.debug.logFiles")}
          </div>
          {t("settings.debug.logFilesDescription")}
        </div>
        <Button variant="outline" onclick={openLogDir}>
          {t("settings.debug.openLogs")}
        </Button>
      </div>

      <div class="mt-4 flex items-center justify-between">
        <div class="text-muted-foreground text-sm">
          <div class="text-foreground font-medium">
            {t("settings.debug.diagnosticsBundle")}
          </div>
          {t("settings.debug.diagnosticsBundleDescription")}
        </div>
        <Button variant="outline" onclick={createDiagnosticsBundle}>
          {t("settings.debug.createDiagnosticsBundle")}
        </Button>
      </div>
    </div>
  </div>
</div>
