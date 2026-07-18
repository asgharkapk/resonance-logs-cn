/**
 * Player-customized buff icons.
 *
 * Overrides map a buff base id (as a string) to a file name inside the app's
 * buff-icon directory (`app_local_data_dir()/buff-icons/`, served via Tauri's
 * asset protocol). This module holds pure helpers only — the runtime directory
 * handle lives in `buff-icon-dir.svelte.ts`.
 */

/** Buff base id (string) -> icon file name, e.g. `"997110_1a2b3c4d.png"`. */
export type BuffIconOverrideMap = Record<string, string>;

/** Mirrors the rust-side whitelist (`is_valid_icon_file_name`). */
const ICON_FILE_NAME_PATTERN = /^\d+_[0-9a-f]{8}\.(?:png|jpe?g|webp|gif)$/;

/** Drops malformed entries (bad key or unsafe/unknown file name). */
export function ensureBuffIconOverrides(
  overrides: BuffIconOverrideMap | null | undefined,
): BuffIconOverrideMap {
  const next: BuffIconOverrideMap = {};
  for (const [baseId, fileName] of Object.entries(overrides ?? {})) {
    if (!/^\d+$/.test(baseId)) continue;
    if (!ICON_FILE_NAME_PATTERN.test(fileName)) continue;
    next[baseId] = fileName;
  }
  return next;
}

/**
 * Resolves the image src for a buff:
 * player override (asset URL) > game sprite (static path) > null (no icon).
 *
 * `iconDirUrlPrefix` is the `convertFileSrc`-converted buff-icon directory
 * (trailing slash included); while it is unavailable the override silently
 * falls back to the sprite so rendering never breaks.
 */
export function resolveBuffIconSrc(
  baseId: number,
  spriteFile: string | null | undefined,
  overrides: BuffIconOverrideMap,
  iconDirUrlPrefix: string | null,
): string | null {
  const fileName = overrides[String(baseId)];
  if (fileName && iconDirUrlPrefix) {
    return `${iconDirUrlPrefix}${fileName}`;
  }
  if (spriteFile) {
    return `/images/buff/${spriteFile}`;
  }
  return null;
}
