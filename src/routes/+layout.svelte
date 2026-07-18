<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { setLocale, t } from "$lib/i18n/index.svelte";
  import { SETTINGS } from "$lib/settings-store";
  import {
    initializeMonitoringSettings,
    RECOVERY_NOTICE_STORAGE_KEY,
  } from "$lib/settings-migrations";
  import { commands } from "$lib/bindings";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import AlertTriangleIcon from "virtual:icons/lucide/triangle-alert";
  import FolderOpenIcon from "virtual:icons/lucide/folder-open";
  import LoaderCircleIcon from "virtual:icons/lucide/loader-circle";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import XIcon from "virtual:icons/lucide/x";
  // Only allow warnings and errors to be printed to console in production builds
  if (typeof window !== "undefined" && import.meta.env.PROD) {
    // Keep warn and error; disable verbose logging
    console.log = () => {};
    console.debug = () => {};
    console.info = () => {};
  }

  let { children } = $props();
  type BootstrapState =
    | { status: "loading" }
    | { status: "recovering" }
    | { status: "ready" }
    | { status: "error"; message: string };
  let bootstrapState = $state<BootstrapState>({ status: "loading" });
  let windowLabel = $state("");
  let recoveryNotice = $state<string | null>(null);

  onMount(() => {
    windowLabel = getCurrentWebviewWindow().label;
    void initializeMonitoringSettings()
      .then((result) => {
        if (result.status === "reload-required") {
          bootstrapState = { status: "recovering" };
          setTimeout(() => window.location.reload(), 100);
          return;
        }
        bootstrapState = { status: "ready" };
        if (windowLabel === "main") {
          recoveryNotice = localStorage.getItem(RECOVERY_NOTICE_STORAGE_KEY);
        }
      })
      .catch((error) => {
        console.error("[monitoring-settings] initialization failed", error);
        bootstrapState = {
          status: "error",
          message: error instanceof Error ? error.message : String(error),
        };
      });
  });

  function reloadWindow() {
    window.location.reload();
  }

  async function openLogDirectory() {
    try {
      const result = await commands.openLogDir();
      if (result.status === "error") throw new Error(String(result.error));
    } catch (error) {
      bootstrapState = {
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      };
    }
  }

  function dismissRecoveryNotice() {
    localStorage.removeItem(RECOVERY_NOTICE_STORAGE_KEY);
    recoveryNotice = null;
  }

  const customThemeKeyToCssVar: Record<string, string | string[]> = {
    backgroundMain: "--background-main",
    backgroundLive: "--background-live",
    foreground: "--foreground",
    surface: ["--card", "--popover"],
    surfaceForeground: ["--card-foreground", "--popover-foreground"],
    primary: ["--primary", "--ring"],
    primaryForeground: "--primary-foreground",
    secondary: "--secondary",
    secondaryForeground: "--secondary-foreground",
    muted: "--muted",
    mutedForeground: "--muted-foreground",
    accent: "--accent",
    accentForeground: "--accent-foreground",
    destructive: "--destructive",
    destructiveForeground: "--destructive-foreground",
    border: "--border",
    input: "--input",
    tooltipBg: "--tooltip-bg",
    tooltipBorder: "--tooltip-border",
    tooltipFg: "--tooltip-fg",
    tableTextColor: ["--player-text-color", "--skill-text-color"],
    tableAbbreviatedColor: ["--abbreviated-color", "--skill-abbreviated-color"],
  };

  // Apply custom theme colors to CSS variables
  function applyCustomThemeColors(colors: Record<string, string>) {
    const root = document.documentElement;
    for (const [key, cssVars] of Object.entries(customThemeKeyToCssVar)) {
      const colorValue = colors[key];
      if (colorValue) {
        if (Array.isArray(cssVars)) {
          cssVars.forEach((v) => root.style.setProperty(v, colorValue));
        } else {
          root.style.setProperty(cssVars, colorValue);
        }
      }
    }
  }

  // Remove custom theme inline styles
  function clearCustomThemeColors() {
    const root = document.documentElement;
    for (const cssVars of Object.values(customThemeKeyToCssVar)) {
      if (Array.isArray(cssVars)) {
        cssVars.forEach((v) => root.style.removeProperty(v));
      } else {
        root.style.removeProperty(cssVars);
      }
    }
  }
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<!-- Apply theme on the document element -->
{(() => {
  $effect(() => {
    if (typeof document !== "undefined") {
      const locale = setLocale(SETTINGS.i18n.state.locale);
      document.documentElement.lang = locale;

      // The live overlay renders its own palette (from the active loadout's
      // live profile) so it can differ from the main window's; every other
      // window (main, overlays) uses the global main-window palette.
      const customThemeColors =
        windowLabel === "live"
          ? SETTINGS.live.appearance.state.themeColors
          : SETTINGS.accessibility.state.customThemeColors;

      // Always operate in 'custom' theme mode. Apply any custom colors if present.
      document.documentElement.setAttribute("data-theme", "custom");

      if (customThemeColors) {
        applyCustomThemeColors(customThemeColors);
      } else {
        clearCustomThemeColors();
      }
    }
  });
})()}

{(() => {})()}

{#if bootstrapState.status === "ready"}
  {@render children()}
  {#if windowLabel === "main" && recoveryNotice}
    <div
      class="border-border bg-popover text-popover-foreground fixed right-4 bottom-4 z-[100] flex max-w-lg items-start gap-3 rounded-lg border p-3 shadow-xl"
      role="status"
    >
      <AlertTriangleIcon class="text-primary mt-0.5 h-4 w-4 shrink-0" />
      <div class="min-w-0 flex-1">
        <p class="text-sm font-medium">
          {t("monitoring.recovery.noticeTitle")}
        </p>
        <p class="text-muted-foreground mt-1 text-xs">
          {t("monitoring.recovery.noticeDescription")}
        </p>
        <p
          class="text-muted-foreground mt-1 truncate font-mono text-[11px]"
          title={recoveryNotice}
        >
          {recoveryNotice}
        </p>
      </div>
      <button
        type="button"
        class="text-muted-foreground hover:text-foreground flex h-7 w-7 shrink-0 items-center justify-center rounded transition-colors"
        aria-label={t("monitoring.recovery.dismiss")}
        onclick={dismissRecoveryNotice}
      >
        <XIcon class="h-4 w-4" />
      </button>
    </div>
  {/if}
{:else if bootstrapState.status === "error"}
  <main
    class="bg-background-main text-foreground flex min-h-screen items-center justify-center p-6"
  >
    <div class="w-full max-w-xl">
      <AlertTriangleIcon class="text-destructive h-8 w-8" />
      <h1 class="mt-4 text-xl font-semibold">
        {t("monitoring.recovery.errorTitle")}
      </h1>
      <p class="text-muted-foreground mt-2 text-sm">
        {t("monitoring.recovery.errorDescription")}
      </p>
      <pre
        class="border-border bg-muted/30 mt-4 max-h-40 overflow-auto rounded border p-3 text-xs whitespace-pre-wrap">{bootstrapState.message}</pre>
      <div class="mt-4 flex flex-wrap gap-2">
        <button
          type="button"
          class="bg-primary text-primary-foreground flex items-center gap-2 rounded px-3 py-2 text-sm"
          onclick={reloadWindow}
        >
          <RefreshCwIcon class="h-4 w-4" />
          {t("monitoring.recovery.reload")}
        </button>
        <button
          type="button"
          class="border-border hover:bg-muted/40 flex items-center gap-2 rounded border px-3 py-2 text-sm"
          onclick={() => void openLogDirectory()}
        >
          <FolderOpenIcon class="h-4 w-4" />
          {t("monitoring.recovery.openLogs")}
        </button>
      </div>
    </div>
  </main>
{:else}
  <main
    class="bg-background-main text-muted-foreground flex min-h-screen items-center justify-center gap-3 text-sm"
  >
    <LoaderCircleIcon class="h-5 w-5 animate-spin" />
    {bootstrapState.status === "recovering"
      ? t("monitoring.recovery.recovering")
      : t("monitoring.recovery.loading")}
  </main>
{/if}
