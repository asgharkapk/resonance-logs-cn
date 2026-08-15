/**
 * @file Maps the app's UI locale to a preset voice generation source, so
 * "preset" generation always uses the bundled reference voice that matches
 * the language the user is reading the app in, regardless of which
 * language the phrase text itself is written in (that's `phrase.language`,
 * a separate concept - see `voice::presets` in the backend).
 */
import type { VoiceGenerateRequestDto } from "$lib/bindings";
import type { AppLocale } from "$lib/i18n/locales";

/**
 * `VoicePresetLocale` (Rust) and `AppLocale` (frontend) both serialize to
 * the same three string literals ("zh-CN" | "en-US" | "ja-JP"), so this is
 * a type-checked identity rather than a lookup table. Kept as a named
 * function so call sites read intentionally and the mapping has one place
 * to unit test if either union ever diverges.
 */
export function presetSourceForLocale(
  locale: AppLocale,
): VoiceGenerateRequestDto["source"] {
  return { mode: "preset", locale };
}
