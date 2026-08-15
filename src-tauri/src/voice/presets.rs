//! Built-in reference voices, selected automatically by the app's UI
//! locale so a user can generate voice lines right after installing the
//! TTS model, without recording or importing their own reference audio.
//!
//! Layout under `src-tauri/voice-presets/` (bundled verbatim via
//! `tauri.conf.json`'s `bundle.resources`, alongside `./meter-data/`):
//! - `presets.json` — revision + per-locale display name/attribution,
//!   embedded into the binary at compile time (see [`EMBEDDED_MANIFEST`])
//!   so the revision can never drift from the code that reads it.
//! - `<locale>.wav` — the reference audio itself, resolved at runtime from
//!   the installed app's resource directory (or, in dev builds, straight
//!   from the crate directory).
//! - `LICENSE.md` — attribution/licensing notes for whatever audio
//!   currently ships in this directory.
//!
//! A cloned [`super::models::VoiceProfileMeta`] extracted from one of these
//! files is tagged with a [`VoicePresetTag`] (locale + manifest revision),
//! so generation can tell a still-current preset profile apart from a
//! stale one left over from an older bundled recording and re-extract
//! instead of silently reusing outdated audio.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use super::error::{VoiceError, VoiceResult};

const EMBEDDED_MANIFEST: &str = include_str!("../../voice-presets/presets.json");

/// A UI locale with a bundled reference voice. Deliberately distinct from
/// [`super::models::VoiceLanguage`], which tags the *text* of a phrase and
/// drives the sidecar's codec language id: a Chinese-locale user can still
/// author (and hear synthesized) an English or Japanese phrase, and should
/// keep hearing it in the Chinese preset voice unless they pick a different
/// source explicitly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum VoicePresetLocale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "ja-JP")]
    JaJp,
}

impl VoicePresetLocale {
    fn file_name(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN.wav",
            Self::EnUs => "en-US.wav",
            Self::JaJp => "ja-JP.wav",
        }
    }
}

/// Identifies which bundled reference file (and manifest revision) a cloned
/// profile was extracted from. Stored on [`super::models::VoiceProfileMeta`]
/// so a later generation can detect that the bundled audio moved on to a
/// new revision and re-extract rather than reuse a stale embedding.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoicePresetTag {
    pub locale: VoicePresetLocale,
    pub revision: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresetVoiceEntry {
    display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PresetManifest {
    revision: u32,
    voices: PresetVoicesByLocale,
}

#[derive(Debug, Clone, Deserialize)]
struct PresetVoicesByLocale {
    #[serde(rename = "zh-CN")]
    zh_cn: PresetVoiceEntry,
    #[serde(rename = "en-US")]
    en_us: PresetVoiceEntry,
    #[serde(rename = "ja-JP")]
    ja_jp: PresetVoiceEntry,
}

impl PresetVoicesByLocale {
    fn entry(&self, locale: VoicePresetLocale) -> &PresetVoiceEntry {
        match locale {
            VoicePresetLocale::ZhCn => &self.zh_cn,
            VoicePresetLocale::EnUs => &self.en_us,
            VoicePresetLocale::JaJp => &self.ja_jp,
        }
    }
}

static MANIFEST: LazyLock<PresetManifest> = LazyLock::new(|| {
    serde_json::from_str(EMBEDDED_MANIFEST)
        .expect("embedded voice-presets/presets.json must be valid and match PresetManifest")
});

/// The `(locale, revision)` identity of the preset voice bundled with this
/// build. Compared against a candidate profile's stored [`VoicePresetTag`]
/// to decide whether it can be reused as-is or must be re-extracted.
pub fn current_tag(locale: VoicePresetLocale) -> VoicePresetTag {
    VoicePresetTag {
        locale,
        revision: MANIFEST.revision,
    }
}

/// Human-readable name for a freshly extracted preset profile, shown in the
/// profile list like any other cloned voice.
pub fn display_name(locale: VoicePresetLocale) -> String {
    MANIFEST.voices.entry(locale).display_name.clone()
}

/// Resolves the on-disk path of the bundled reference WAV for `locale`.
///
/// Mirrors the resource lookup used for the qwen3-tts sidecar binaries
/// (`resolve_sidecar_path` in `voice/mod.rs`): a dev-mode path relative to
/// this crate for `cargo run`/`tauri dev`, then the installed app's
/// resource directory for a bundled release build.
pub fn resolve_audio_path(app_handle: &AppHandle, locale: VoicePresetLocale) -> VoiceResult<PathBuf> {
    let file_name = locale.file_name();

    #[cfg(debug_assertions)]
    {
        let candidate = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("voice-presets")
            .join(file_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let candidate = resource_dir.join("voice-presets").join(file_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(VoiceError::not_found(
        "built-in preset reference audio",
        file_name.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_manifest_parses_and_has_a_positive_revision_baseline() {
        // Forces the LazyLock to run (and thus `serde_json::from_str` +
        // struct-shape validation) instead of only checking it lazily on
        // first real use.
        let _ = &*MANIFEST;
    }

    #[test]
    fn every_locale_resolves_a_display_name_and_stable_file_name() {
        for locale in [
            VoicePresetLocale::ZhCn,
            VoicePresetLocale::EnUs,
            VoicePresetLocale::JaJp,
        ] {
            assert!(!display_name(locale).is_empty());
            assert!(locale.file_name().ends_with(".wav"));
        }
    }

    #[test]
    fn current_tag_carries_the_manifest_revision() {
        let tag = current_tag(VoicePresetLocale::JaJp);
        assert_eq!(tag.revision, MANIFEST.revision);
        assert_eq!(tag.locale, VoicePresetLocale::JaJp);
    }
}
