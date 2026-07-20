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

## Website

### New HTML Inspired Homepage

A brand-new landing page has been created to showcase Resonance Logs as a modern Windows desktop application instead of a traditional documentation page.

#### Added

* Responsive single-page application homepage.
* Acrylic (glassmorphism) navigation bar.
* Hero section with application branding.
* Primary call-to-action buttons:

  * Download for Windows
  * View Documentation
* Feature showcase section highlighting:

  * DPS Meter
  * Dungeon & Raid DBM
  * Interactive Minimap
  * Voice Callouts
  * Combat Logging
  * Theme Customization
  * Multi-language Support
* Screenshot gallery for application previews.
* Documentation overview cards for quick navigation.
* Modern footer with application branding.

---

## User Interface

### Fluent Design

The homepage has been redesigned to match the look and feel of modern Windows applications.

#### Added

* Acrylic glass background effects.
* Blur and transparency effects.
* Animated gradient background.
* Rounded Fluent Design cards.
* Modern typography using Segoe UI.
* Hover animations for buttons, cards, and screenshots.
* Responsive navigation bar.
* Large hero section optimized for desktop displays.

### Improved

* Cleaner visual hierarchy.
* Better spacing and readability.
* Improved responsiveness across different screen sizes.
* More desktop application–oriented presentation compared to a documentation-focused layout.

---

## Homepage Content

### Application Overview

The landing page now introduces Resonance Logs as a complete companion application for Star Resonance.

#### Added

* Hero headline describing the application's purpose.
* Short feature summary explaining:

  * DPS analysis
  * Combat logging
  * Dungeon & Raid DBM
  * Interactive Minimap
  * Voice Callouts
  * Buff & Skill Tracking
* Download section for Windows users.
* Documentation entry point.

---

## Feature Showcase

### Interactive Feature Cards

Dedicated feature cards have been added to present the application's major capabilities.

#### Added

* DPS Meter
* Interactive Minimap
* Voice Callouts
* Combat Logging
* Theme Customization
* Multi-language Support

Each feature includes a concise description to help new users quickly understand the application's functionality.

---

## Screenshots

### Application Gallery

A preview gallery has been added to visually showcase the application's interface.

#### Added

* Compact Theme preview.
* Theme Customization preview.
* Accuracy Test preview.
* DPS Overlay preview.
* Dungeon Mechanics Minimap preview.
* Additional overlay screenshots.

---

## Documentation

### Documentation Navigation

Documentation links have been reorganized into a dedicated homepage section.

#### Added

* README
* English Documentation
* Simplified Chinese Documentation
* CHANGELOG
* Fork Changelog
* TODO List

### Improved

* Easier navigation to project documentation.
* Cleaner organization of project resources.
* More user-friendly presentation for first-time visitors.

---

## Responsive Design

### Cross-Device Compatibility

The homepage now adapts automatically to different screen sizes.

#### Added

* Responsive navigation menu.
* Responsive feature grid.
* Responsive screenshot gallery.
* Mobile-friendly hero section.
* Flexible layouts for tablets and desktop displays.

---

## Visual Improvements

### Modern Appearance

The website has been redesigned to better represent a premium Windows desktop application.

#### Added

* Soft shadows.
* Smooth transitions.
* Hover animations.
* Rounded interface elements.
* Glass-effect cards.
* High-contrast dark theme.
* Large application preview.
* Consistent spacing throughout the page.

> This update introduces a completely new HTML homepage, transforming the project from a documentation-centric landing page into a modern desktop application website while preserving access to all existing documentation.
