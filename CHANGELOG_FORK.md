# Changelog

## CI / Build Automation

### Automated Build & Release Pipeline

A complete GitHub Actions CI/CD pipeline has been added to automate application builds and releases.

#### Added

* Automatic application builds on every push.
* Automatic version tag generation using the format:

  * `vYYYY.MM.DD-r<run_number>`
* Automatic GitHub Release creation.
* Automatic upload of release artifacts:

  * NSIS installer
  * Auto-updater packages
  * Portable ZIP package
  * Standalone executable
* Dependency caching for Node.js and Rust to significantly reduce build times.
* Automatic tag creation and release publishing using the default GitHub Actions token.

---

## Documentation

The project documentation has been extensively reorganized and translated to improve accessibility for both English and Chinese users.

### Added

* English documentation index.
* Quick navigation buttons for:

  * `README.md`
  * `README_EN.md`
  * `README_CN.md`
  * `CHANGELOG.md`
  * `CHANGELOG_FORK.md`
* Language selection table covering all available documentation.
* Preview screenshots for major features:

  * Compact Theme
  * Theme Settings
  * Accuracy Test
  * Dungeon Mechanics Minimap
  * DPS Overlay UI
  * Language Settings
* Expanded documentation build instructions.
* Documentation maintenance guidelines for shared images and localized UI placeholders.
* GitHub callout blocks:

  * **NOTE**
  * **TIP**
  * **IMPORTANT**
  * **WARNING**
  * **CAUTION**

### Improved

* Translated the documentation index from Simplified Chinese to English.
* Reorganized the README for easier navigation.
* Improved explanations for HTML generation and single-language documentation builds.
* Enhanced formatting, readability, and overall documentation structure.

> This update contains documentation improvements only and does not include application code changes.

---

## Localization

### Updated Translations

Updated multiple localization and database files to improve translation coverage and keep game data synchronized.

#### Updated

* `SceneName.json`
* `DbmTable.json`
* `MonsterIdNameType.json`
* `skill_aoyi_icons.json`
* `README.md`

These updates include new translations, terminology improvements, and synchronization with the latest game content.
