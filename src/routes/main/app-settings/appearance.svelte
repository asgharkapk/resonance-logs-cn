<script lang="ts">
  /**
   * @file Main-window appearance: theme colors, background image and custom
   * fonts. These are global app settings (kept on `SETTINGS.accessibility`)
   * and never travel with an exported loadout — the live-overlay equivalents
   * (theme colors, class/spec colors) live under DPS → 主题 instead, scoped
   * to the active loadout.
   */
  import SettingsSwitch from "../dps/settings/settings-switch.svelte";
  import SettingsSlider from "../dps/settings/settings-slider.svelte";
  import SettingsSelect from "../dps/settings/settings-select.svelte";
  import SettingsColorAlpha from "../dps/settings/settings-color-alpha.svelte";
  import SettingsFilePicker from "../dps/settings/settings-file-picker.svelte";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import {
    SETTINGS,
    DEFAULT_CUSTOM_THEME_COLORS,
    CUSTOM_THEME_COLOR_LABELS,
    type CustomThemeColors,
  } from "$lib/settings-store";
  import { COLOR_PRESETS } from "$lib/theme-color-presets";
  import ChevronDown from "virtual:icons/lucide/chevron-down";

  // This tab only edits the main window's global palette — `backgroundLive`
  // belongs to the active loadout's live appearance, edited instead under
  // DPS → 主题.
  const MAIN_IRRELEVANT_COLOR_KEYS = new Set<keyof CustomThemeColors>([
    "backgroundLive",
  ]);

  let expandedSections = $state({
    colorThemes: false,
    backgroundImage: false,
    customFonts: false,
  });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  $effect(() => {
    if (
      typeof SETTINGS.accessibility.state.backgroundImageOpacity !== "number"
    ) {
      SETTINGS.accessibility.state.backgroundImageOpacity = 100;
    }
  });

  const colorCategories = $derived.by(() => {
    const categories: Record<
      string,
      Array<keyof typeof DEFAULT_CUSTOM_THEME_COLORS>
    > = {};
    for (const [key, info] of Object.entries(CUSTOM_THEME_COLOR_LABELS)) {
      if (MAIN_IRRELEVANT_COLOR_KEYS.has(key as keyof CustomThemeColors))
        continue;
      if (!categories[info.category]) {
        categories[info.category] = [];
      }
      categories[info.category]!.push(
        key as keyof typeof DEFAULT_CUSTOM_THEME_COLORS,
      );
    }
    return categories;
  });

  const categoryOrder = [
    "Base",
    "Surfaces",
    "Tooltip",
    "Accents",
    "Tables",
    "Utility",
  ];

  function colorCategoryLabel(category: string) {
    return t(`themes.colors.category.${category}` as MessageKey);
  }

  function customThemeColorLabel(
    colorKey: keyof typeof DEFAULT_CUSTOM_THEME_COLORS,
  ) {
    return t(`themes.colors.${colorKey}.label` as MessageKey);
  }

  function customThemeColorDescription(
    colorKey: keyof typeof DEFAULT_CUSTOM_THEME_COLORS,
  ) {
    return t(`themes.colors.${colorKey}.description` as MessageKey);
  }

  function updateCustomThemeColor(
    key: keyof typeof DEFAULT_CUSTOM_THEME_COLORS,
    value: string,
  ) {
    SETTINGS.accessibility.state.customThemeColors = {
      ...SETTINGS.accessibility.state.customThemeColors,
      [key]: value,
    };
  }

  function resetCustomThemeColors() {
    SETTINGS.accessibility.state.customThemeColors = {
      ...DEFAULT_CUSTOM_THEME_COLORS,
    };
  }

  function applyColorPreset(presetKey: string) {
    const preset = COLOR_PRESETS[presetKey];
    if (preset) {
      SETTINGS.accessibility.state.customThemeColors = {
        ...SETTINGS.accessibility.state.customThemeColors,
        ...preset.vars,
      };
    }
  }
</script>

<p class="text-muted-foreground text-xs">
  {t("appSettings.appearance.scope")}
</p>

<div class="space-y-3">
  <!-- Color Theme Presets -->
  <div class="space-y-2">
    <div>
      <h2 class="text-sm font-semibold text-foreground">
        {t("themes.presets.colorTitle")}
      </h2>
      <p class="text-xs text-muted-foreground mt-0.5">
        {t("appSettings.appearance.colorScopeMain")}
      </p>
    </div>
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
      {#each Object.entries(COLOR_PRESETS) as [key, preset] (key)}
        <button
          type="button"
          class="group relative flex flex-col items-start p-3 rounded-lg border border-border/60 bg-card/40 hover:bg-card/60 hover:border-primary/50 transition-all text-left"
          onclick={() => applyColorPreset(key)}
        >
          <div class="flex gap-1.5 mb-2">
            <span
              class="w-3.5 h-3.5 rounded-full border border-black/20"
              style="background: {preset.preview.bg}"
            ></span>
            <span
              class="w-3.5 h-3.5 rounded-full border border-black/20"
              style="background: {preset.preview.primary}"
            ></span>
            <span
              class="w-3.5 h-3.5 rounded-full border border-black/20"
              style="background: {preset.preview.accent}"
            ></span>
            <span
              class="w-3.5 h-3.5 rounded-full border border-black/20"
              style="background: {preset.preview.fg}"
            ></span>
          </div>
          <span class="text-sm font-medium text-foreground"
            >{t(preset.nameKey)}</span
          >
        </button>
      {/each}
    </div>
  </div>

  <!-- Color Themes Section -->
  <div
    class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
  >
    <button
      type="button"
      class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
      onclick={() => toggleSection("colorThemes")}
    >
      <h2 class="text-base font-semibold text-foreground">
        {t("themes.section.colorThemes")}
      </h2>
      <ChevronDown
        class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.colorThemes
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSections.colorThemes}
      <div class="px-4 pb-4 space-y-3">
        <div class="mt-3 pt-3 border-t border-border/50">
          <div class="flex items-center justify-between mb-3">
            <div>
              <h3 class="text-sm font-semibold text-foreground">
                {t("themes.section.customColorTheme")}
              </h3>
              <p class="text-xs text-muted-foreground mt-0.5">
                {t("themes.section.customColorThemeDescription")}
              </p>
            </div>
            <button
              onclick={resetCustomThemeColors}
              class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors"
              >{t("themes.action.reset")}</button
            >
          </div>

          {#each categoryOrder as category (category)}
            {#if colorCategories[category]}
              <div class="mb-4">
                <h4
                  class="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2 px-1"
                >
                  {colorCategoryLabel(category)}
                </h4>
                <div class="space-y-1">
                  {#each colorCategories[category] ?? [] as colorKey (colorKey)}
                    {@const colorInfo = CUSTOM_THEME_COLOR_LABELS[colorKey]}
                    {#if colorInfo}
                      <SettingsColorAlpha
                        label={customThemeColorLabel(colorKey)}
                        description={customThemeColorDescription(colorKey)}
                        value={SETTINGS.accessibility.state
                          .customThemeColors?.[colorKey] ??
                          DEFAULT_CUSTOM_THEME_COLORS[colorKey] ??
                          "rgba(128, 128, 128, 1)"}
                        oninput={(value: string) =>
                          updateCustomThemeColor(colorKey, value)}
                      />
                    {/if}
                  {/each}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Background Image Section -->
  <div
    class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
  >
    <button
      type="button"
      class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
      onclick={() => toggleSection("backgroundImage")}
    >
      <h2 class="text-base font-semibold text-foreground">
        {t("themes.background.title")}
      </h2>
      <ChevronDown
        class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.backgroundImage
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSections.backgroundImage}
      <div class="px-4 pb-4 space-y-2">
        <p class="text-xs text-muted-foreground">
          {t("themes.background.description")}
        </p>
        <SettingsSwitch
          bind:checked={SETTINGS.accessibility.state.backgroundImageEnabled}
          label={t("themes.background.enable")}
          description={t("themes.background.enableDescription")}
        />
        {#if SETTINGS.accessibility.state.backgroundImageEnabled}
          <div class="mt-2 space-y-2">
            <SettingsFilePicker
              label={t("themes.background.selectImage")}
              description={t("themes.background.selectImageDescription")}
              accept="image/*"
              value={SETTINGS.accessibility.state.backgroundImage}
              onchange={(dataUrl) => {
                SETTINGS.accessibility.state.backgroundImage = dataUrl;
              }}
              onclear={() => {
                SETTINGS.accessibility.state.backgroundImage = "";
              }}
            />
            <SettingsSlider
              bind:value={SETTINGS.accessibility.state.backgroundImageOpacity}
              min={0}
              max={100}
              step={1}
              unit="%"
              label={t("themes.background.opacity")}
              description={t("themes.background.opacityDescription")}
            />
            <SettingsSelect
              label={t("themes.background.imageMode")}
              description={t("themes.background.imageModeDescription")}
              bind:selected={SETTINGS.accessibility.state.backgroundImageMode}
              values={[
                {
                  label: t("themes.background.imageMode.cover"),
                  value: "cover",
                },
                {
                  label: t("themes.background.imageMode.contain"),
                  value: "contain",
                },
              ]}
            />
            {#if SETTINGS.accessibility.state.backgroundImageMode === "contain"}
              <SettingsColorAlpha
                label={t("themes.background.fillColor")}
                description={t("themes.background.fillColorDescription")}
                value={SETTINGS.accessibility.state
                  .backgroundImageContainColor}
                oninput={(value: string) => {
                  SETTINGS.accessibility.state.backgroundImageContainColor =
                    value;
                }}
              />
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Custom Fonts Section -->
  <div
    class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
  >
    <button
      type="button"
      class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
      onclick={() => toggleSection("customFonts")}
    >
      <h2 class="text-base font-semibold text-foreground">
        {t("themes.fonts.title")}
      </h2>
      <ChevronDown
        class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.customFonts
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSections.customFonts}
      <div class="px-4 pb-4 space-y-4">
        <p class="text-xs text-muted-foreground">
          {t("themes.fonts.description")}
        </p>

        <SettingsSwitch
          bind:checked={SETTINGS.accessibility.state.customFontApplyToOverlay}
          label={t("themes.fonts.applyToOverlay")}
          description={t("themes.fonts.applyToOverlayDescription")}
        />

        <!-- Sans-serif Font -->
        <div class="space-y-2 pt-2 border-t border-border/30">
          <h3 class="text-sm font-semibold text-foreground">
            {t("themes.fonts.sansTitle")}
          </h3>
          <p class="text-xs text-muted-foreground">
            {t("themes.fonts.defaultSans")}
          </p>
          <SettingsSwitch
            bind:checked={SETTINGS.accessibility.state.customFontSansEnabled}
            label={t("themes.fonts.enableSans")}
            description={t("themes.fonts.enableSansDescription")}
          />
          {#if SETTINGS.accessibility.state.customFontSansEnabled}
            <SettingsFilePicker
              label={t("themes.fonts.selectFont")}
              description={t("themes.fonts.selectFontDescription")}
              accept=".woff2,.woff,.ttf,.otf"
              value={SETTINGS.accessibility.state.customFontSansUrl}
              onchange={(dataUrl, fileName) => {
                SETTINGS.accessibility.state.customFontSansUrl = dataUrl;
                const fontName = fileName.replace(/\.(woff2?|ttf|otf)$/i, "");
                SETTINGS.accessibility.state.customFontSansName = fontName;
                const fontFace = new FontFace(fontName, `url(${dataUrl})`);
                fontFace
                  .load()
                  .then((loadedFace) => {
                    document.fonts.add(loadedFace);
                  })
                  .catch((e) => console.error("Failed to load font:", e));
              }}
              onclear={() => {
                SETTINGS.accessibility.state.customFontSansUrl = "";
                SETTINGS.accessibility.state.customFontSansName = "";
              }}
            />
            {#if SETTINGS.accessibility.state.customFontSansName}
              <p class="text-xs text-muted-foreground pl-3">
                {t("themes.fonts.loaded", {
                  name: SETTINGS.accessibility.state.customFontSansName,
                })}
              </p>
            {/if}
          {/if}
        </div>

        <!-- Monospace Font -->
        <div class="space-y-2 pt-3 border-t border-border/30">
          <h3 class="text-sm font-semibold text-foreground">
            {t("themes.fonts.monoTitle")}
          </h3>
          <p class="text-xs text-muted-foreground">
            {t("themes.fonts.defaultMono")}
          </p>
          <SettingsSwitch
            bind:checked={SETTINGS.accessibility.state.customFontMonoEnabled}
            label={t("themes.fonts.enableMono")}
            description={t("themes.fonts.enableMonoDescription")}
          />
          {#if SETTINGS.accessibility.state.customFontMonoEnabled}
            <SettingsFilePicker
              label={t("themes.fonts.selectFont")}
              description={t("themes.fonts.selectFontDescription")}
              accept=".woff2,.woff,.ttf,.otf"
              value={SETTINGS.accessibility.state.customFontMonoUrl}
              onchange={(dataUrl, fileName) => {
                SETTINGS.accessibility.state.customFontMonoUrl = dataUrl;
                const fontName = fileName.replace(/\.(woff2?|ttf|otf)$/i, "");
                SETTINGS.accessibility.state.customFontMonoName = fontName;
                const fontFace = new FontFace(fontName, `url(${dataUrl})`);
                fontFace
                  .load()
                  .then((loadedFace) => {
                    document.fonts.add(loadedFace);
                  })
                  .catch((e) => console.error("Failed to load font:", e));
              }}
              onclear={() => {
                SETTINGS.accessibility.state.customFontMonoUrl = "";
                SETTINGS.accessibility.state.customFontMonoName = "";
              }}
            />
            {#if SETTINGS.accessibility.state.customFontMonoName}
              <p class="text-xs text-muted-foreground pl-3">
                {t("themes.fonts.loaded", {
                  name: SETTINGS.accessibility.state.customFontMonoName,
                })}
              </p>
            {/if}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
