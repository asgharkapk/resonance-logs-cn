# <img src="app-logo.png" width="64" align="center"> Star Resonance DPS meter & Dungeon/Raid DBM & Minimap & voice callout & Skill/Buff Trackers Overlay

<div align="center">

# [<img src="src-tauri/icons/icon.ico" width="64" align="center"> Resonance Logs Documentation](https://asgharkapk.github.io/resonance-logs-cn/index_old.html)

A feature-rich companion application for **Star Resonance** with DPS analysis, overlays, combat logging, minimap support, dungeon mechanic guides, and more.

[![English](https://img.shields.io/badge/README-English-blue?style=for-the-badge)](./README_EN.md)
[![简体中文](https://img.shields.io/badge/README-简体中文-red?style=for-the-badge)](./README_CN.md)
[![日本語](https://img.shields.io/badge/README-日本語-purple?style=for-the-badge)](./README_JP.md)

[![DPS Meter](https://img.shields.io/badge/DPS%20Meter-✓-blue?style=for-the-badge)](#compact-theme)
[![Raid DBM](https://img.shields.io/badge/Raid%20DBM-✓-purple?style=for-the-badge)](#theme-customization)
[![Minimap](https://img.shields.io/badge/Minimap-✓-green?style=for-the-badge)](#dungeon-mechanics-guide-minimap)
[![Voice Callouts](https://img.shields.io/badge/Voice%20Callouts-✓-red?style=for-the-badge)](#-changing-the-application-language)
[![Buff Tracker](https://img.shields.io/badge/Buff%20Tracker-✓-orange?style=for-the-badge)](#-changing-the-application-language)
[![Overlay](https://img.shields.io/badge/Overlay-✓-teal?style=for-the-badge)](#dps-overlay-as-your-game-ui)

[![Windows x64](https://img.shields.io/badge/Download-Windows%20x64-0078D6?style=for-the-badge)](../../releases)
[![Portable](https://img.shields.io/badge/Portable-Available-success?style=for-the-badge)](../../releases)
[![Installer](https://img.shields.io/badge/Installer-MSI-blue?style=for-the-badge)](../../releases)

[![Windows](https://img.shields.io/badge/Platform-Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
](https://www.microsoft.com/en-us/windows)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust)
](https://rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
](https://v2.tauri.app/)

[![Open Source](https://img.shields.io/badge/Open%20Source-Yes-success?style=for-the-badge)](#)
[![Actively Developed](https://img.shields.io/badge/Status-Actively%20Developed-brightgreen?style=for-the-badge)](#)
[![Stable](https://img.shields.io/badge/Status-Stable-success?style=for-the-badge)](#)
[![MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](https://www.mit.edu/)
[![GitHub Issues](https://img.shields.io/badge/Issues-Welcome-success?style=for-the-badge)](../../issues)
[![Pull Requests](https://img.shields.io/badge/PRs-Welcome-brightgreen?style=for-the-badge)](../../pulls)
[![Contributions](https://img.shields.io/badge/Contributions-Welcome-blue?style=for-the-badge)](../../pulls)
[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?style=for-the-badge&logo=discord&logoColor=white)
](https://discord.gg/RHeX47wvDU)

---

### 📖 Documentation

| Document                                                                                            | Description                      |
| --------------------------------------------------------------------------------------------------- | -------------------------------- |
| 📘 [README](README.md)                    | Main documentation               |
| 🇺🇸 [README_EN.md](README_EN.md)         | English documentation            |
| 🇨🇳 [README_CN.md](README_CN.md)         | 文档 |
| 🇨🇳 [README_JP.md](README_JP.md)         | ドキュメント |
| 📝 [CHANGELOG_CN.md](CHANGELOG.md)           | 项目变更日志                |
| 📝 [CHANGELOG_EN.md](src/lib/changelog/en-US.md)           | Project changelog                |
| 📝 [CHANGELOG_JP.md](src/lib/changelog/ja-JP.md)           | プロジェクト変更履歴                |
| 🍴 [CHANGELOG_FORK.md](CHANGELOG_FORK.md) | Fork-specific changes            |
| 🍴 [TODO.md](TODO.md) | ToDo List            |

---

## 🌍 Available Documentation Languages

The documentation is separated by language.

| Language                | Documentation                        | Description            |
| ----------------------- | ------------------------------------ | ---------------------- |
| 🇨🇳 Simplified Chinese | [doc/zh-CN](doc/zh-CN/README.md) | 默认文档  |
| 🇺🇸 English            | [doc/en-US](doc/en-US/README.md) | English documentation  |
| 🇯🇵 Japanese           | [doc/ja-JP](doc/ja-JP/README.md) | 日本語のドキュメント |

</div>

---

> [!IMPORTANT]
> This repository contains the documentation source files only. If you want to contribute translations or improve existing pages, please edit the corresponding language directory.

> [!NOTE]
> Images are shared across all languages. Every language uses the same image assets to keep documentation consistent and reduce repository size.

> [!TIP]
> If you're new to Resonance Logs, start with the English or Chinese README before exploring individual feature documentation.

---

# ✨ Features Preview

### Compact Theme

<img src="doc/shared/img/dps/dps_5.png">

A minimal interface designed to keep important combat information visible while occupying as little screen space as possible.

---

### Theme Customization

<img src="doc/shared/img/dps/dps_4.png">

Customize colors, layouts, fonts, and appearance to match your preferences.

---

### Accuracy Test

<img src="/doc/shared/img/dps/dps_1.png">

Verify combat log accuracy and ensure the parser is correctly reading game data.

---

### Dungeon Mechanics Guide Minimap

<table>
  <tr>
    <td><img src="doc/shared/img/minimap/s3_minimap_1.png" width="250"></td>
    <td><img src="doc/shared/img/minimap/s3_minimap_2.png" width="250"></td>
    <td><img src="doc/shared/img/minimap/s3_minimap_3.png" width="250"></td>
  </tr>
</table>
Interactive minimap overlays help players quickly learn dungeon mechanics and positioning.

---

### DPS Overlay as Your Game UI

<img src="/doc/shared/img/monitor/buff_4.png">

You can disable the in-game UI and rely on the Resonance Logs overlay for combat information while maintaining a clean gameplay experience.

---

## 🌐 Changing the Application Language

* Complete translations powered by the official game client localization, with AI translations filling in missing or untranslated text.

<img src="doc/shared/img/faq/faq_5.png">

The application supports multiple languages. Open **Settings → Language** to switch the interface language.

> [!TIP]
> After changing the language, restart the application if some interface elements do not immediately update.

---

# 🛠 Building the HTML Documentation

Generate the complete HTML documentation with:

```bash
npm run doc:html
```

This command expands placeholders such as:

```text
{{ui:key}}
```

into their corresponding localized application menu names before generating the documentation.

The generated HTML files are placed in:

```text
doc/html_doc/
```

Open:

```text
doc/html_doc/index.html
```

in your browser to view the complete documentation.

> [!NOTE]
> The generated HTML reflects the application's current localization, making it ideal for publishing or offline browsing.

---

# 🔧 Building a Single Language

For faster development and debugging, generate documentation for only one locale:

```bash
node scripts/build-doc-html.cjs --locale=en-US
```

Replace `en-US` with another supported locale if needed.

---

# 📂 Documentation Maintenance

## Images

All images have a **single source of truth** located in:

```text
shared/img/
```

Do **not** duplicate images into language folders.

Use the following relative paths:

| Document Type     | Image Path                |
| ----------------- | ------------------------- |
| `faq/*.md`        | `../../shared/img/...`    |
| `features/*/*.md` | `../../../shared/img/...` |

> [!IMPORTANT]
> Keeping images centralized prevents inconsistencies and significantly reduces repository size.

---

## UI Menu Placeholders

While writing documentation, use placeholders such as:

```text
{{ui:routes.tools.dps}}
```

During HTML generation, these placeholders are automatically replaced with the correct localized menu names from the application's i18n resources.

> [!TIP]
> This approach allows a single documentation source to remain synchronized with interface translations without manually updating menu names for every language.

---

> [!WARNING]
> Do not manually replace `{{ui:...}}` placeholders with translated text inside Markdown files. Doing so can cause documentation to become outdated when the application's UI translations change.

> [!CAUTION]
> Avoid copying images into language-specific directories. Multiple copies increase maintenance effort and can easily become inconsistent over time.
