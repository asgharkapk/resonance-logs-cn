# Dungeon Mechanics

Corresponds to **Dungeon Mechanics** in the toolbox—a standalone in-game overlay for real-time positioning, callouts, and mechanic warnings in supported dungeons.

Independent from **Monster Monitor** and **Live Monitor**: uses the **minimap overlay** window and activates automatically only in adapted dungeon scenes.

## Overview

The overlay has two independently toggled panels:

| Panel | Description |
|------|------|
| **Map Panel** | Top-down minimap: you, teammates, Boss, and mechanic-related points; colors, regions, and lines mark stacks, safe zones, danger halves, etc. |
| **Mechanic Calls** | Text list: active mechanic names, callout targets, and countdown timers |

When you enter a supported dungeon, the app loads the matching parser by scene ID. Leaving the dungeon or entering an unsupported scene clears mechanic content.

## Enable & Top Bar

On the **Dungeon Mechanics** page top bar:

- **Toggle Dungeon Mechanics Window**: Show or hide the in-game overlay.
- **Edit Layout**: Drag and resize the map panel and mechanic calls panel.

## Display Options

| Option | Description |
|------|------|
| **Auto-hide in daily scenes** | Hides the overlay in daily-scene blacklist entries; restores prior visibility when leaving |
| **Show only self and mechanic-affected teammates** | Hides normal teammate dots; callouts and mechanic buff colors reveal affected players |
| **Show Boss position** | Marks Boss on the minimap (off by default to reduce clutter) |

## Player Colors & Self Marker

- **Player colors**: Default dot colors for you and teammates when no mechanic override applies.
- **Self ring**: Concentric ring around your dot so you stay easy to find even when recolored by a mechanic (color and width configurable).

## Panel Visibility

Toggle **Map Panel** and **Mechanic Calls** separately in settings. Position and width are adjusted in overlay **Edit Layout** mode.

## Supported Dungeons

| Dungeon | Scene IDs | Mechanics (summary) |
|------|---------|----------|
| **Cursed Tomb** | 6513 / 6514 / 6515 | Blue tower activation, energy pillar callouts, half-arena charges, charge clones, etc. |
| **Sea-Ringed Reef** | 6563 / 6564 / 6565 | Matrix rune traps (mid-run), Boss dual-tone colors, ice/wave safe zones and cross lines, etc. |
| **Forgotten Dreamwild** | 13021 / 13022 / 13023 | Phase tiles, stack/decay/spread, causal jump, kill marks, electromagnetic ring sequence, pinball, etc. |

> Mechanic labels match in-overlay text. Normal / Hard / Master difficulties share the same parser with different scene IDs.

## Examples

### Cursed Tomb

Map panel marks tower progress, energy pillars, and charge-related points; mechanic calls show phase and timers.

![Cursed Tomb mechanics example](../../../shared/img/minimap/s3_minimap_1.png)

### Sea-Ringed Reef · Matrix (mid-run)

Rune colors, trap callouts, and teammate positions during the matrix phase.

![Sea-Ringed Reef mid-run example](../../../shared/img/minimap/s3_minimap_2.png)

### Sea-Ringed Reef · Boss

Dual-tone colors, ice/wave safe zones, and cross intersection markers during the Boss fight.

![Sea-Ringed Reef Boss example](../../../shared/img/minimap/s3_minimap_3.png)

## vs Monster Monitor Boss DBM

| Item | Dungeon Mechanics | Monster Monitor · Boss DBM |
|------|----------|---------------------|
| Overlay | Minimap overlay | Monster overlay |
| Main info | Position map + mechanic calls | Boss skill warning bars (name + countdown) |
| Scope | Adapted dungeons | Any fight with Boss DBM events |
| Settings | Dungeon Mechanics page | Monster Monitor → Boss DBM |

Both can run together in different screen areas.
