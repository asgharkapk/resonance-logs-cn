/** Formatting helpers shared by the curve, lanes, minimap and tooltip. Pure
 * functions only so they stay trivially testable and framework-agnostic -
 * callers read the user's abbreviation preference from `SETTINGS` and pass it
 * in rather than this module importing the store. */

import { abbreviateNumber, type AbbreviationStyle } from "$lib/number-format";

export function formatTimeMs(valueMs: number, includeMillis = false): string {
  const totalMs = Math.max(0, Math.round(valueMs));
  const totalSeconds = Math.floor(totalMs / 1_000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = (totalSeconds % 60).toString().padStart(2, "0");
  if (!includeMillis) return `${minutes}:${seconds}`;
  const millis = (totalMs % 1_000).toString().padStart(3, "0");
  return `${minutes}:${seconds}.${millis}`;
}

/** Abbreviates a DPS/damage value for an axis label or tooltip, honouring the
 * user's global abbreviation preference so the timeline agrees with the
 * history tables. */
export function formatValue(
  value: number,
  style: AbbreviationStyle = "western",
  decimals = 1,
): string {
  return abbreviateNumber(value, decimals, style);
}

/** Converts a `#rrggbb` hex color into an `rgba(...)` string at `alpha`.
 * Returns the input unchanged if it isn't a 6-digit hex color. */
export function hexToRgba(hex: string, alpha: number): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return hex;
  const n = parseInt(m[1] ?? "0", 16);
  return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${alpha})`;
}
