//! Player-customized buff icon files.
//!
//! Icons live under `app_local_data_dir()/buff-icons/` and are served to the
//! webview through Tauri's built-in asset protocol (see `tauri.conf.json`).
//! Settings persist only the file name keyed by buff base id; the directory
//! itself is resolved here so path handling stays in one place.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::Manager;

const BUFF_ICON_DIR_NAME: &str = "buff-icons";
const MAX_ICON_BYTES: u64 = 4 * 1024 * 1024;
const ALLOWED_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];

fn buff_icon_dir_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app_local_data_dir: {e}"))?;
    let dir = base.join(BUFF_ICON_DIR_NAME);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("create directory {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Normalizes a directory for webview consumption: forward slashes and a
/// trailing `/`, so the frontend can build `dir + file_name` without an async
/// `path.join`.
fn normalize_dir_prefix(dir: &Path) -> String {
    let mut text = dir.to_string_lossy().replace('\\', "/");
    if !text.ends_with('/') {
        text.push('/');
    }
    text
}

fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.contains(&ext)
}

/// `{base_id}_{8 hex of sha256}.{ext}` — content-addressed, so re-importing
/// the same image is idempotent and a changed image gets a fresh name (which
/// also busts the webview cache).
fn icon_file_name(base_id: u32, contents: &[u8], ext: &str) -> String {
    let hash = Sha256::digest(contents);
    let short = u32::from_be_bytes(hash[..4].try_into().expect("sha256 is 32 bytes"));
    format!("{base_id}_{short:08x}.{ext}")
}

/// Whitelist for delete targets: exactly what `icon_file_name` produces.
/// Rejects directory traversal and any name we did not generate.
fn is_valid_icon_file_name(file_name: &str) -> bool {
    let Some((base, rest)) = file_name.split_once('_') else {
        return false;
    };
    let Some((hash, ext)) = rest.split_once('.') else {
        return false;
    };
    !base.is_empty()
        && base.bytes().all(|b| b.is_ascii_digit())
        && hash.len() == 8
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && is_allowed_extension(ext)
}

/// Copies `source_path` into `dir` under its content-addressed name and
/// removes superseded icons for the same buff. Returns the stored file name.
fn import_buff_icon_into(dir: &Path, base_id: u32, source_path: &Path) -> Result<String, String> {
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| is_allowed_extension(e))
        .ok_or_else(|| format!("unsupported image extension: {}", source_path.display()))?;

    let metadata = std::fs::metadata(source_path)
        .map_err(|e| format!("read metadata {}: {e}", source_path.display()))?;
    if !metadata.is_file() {
        return Err(format!("not a file: {}", source_path.display()));
    }
    if metadata.len() > MAX_ICON_BYTES {
        return Err(format!(
            "image exceeds {} bytes: {}",
            MAX_ICON_BYTES,
            source_path.display()
        ));
    }

    let contents =
        std::fs::read(source_path).map_err(|e| format!("read {}: {e}", source_path.display()))?;
    let file_name = icon_file_name(base_id, &contents, &ext);
    let destination = dir.join(&file_name);
    if !destination.exists() {
        std::fs::write(&destination, &contents)
            .map_err(|e| format!("write {}: {e}", destination.display()))?;
    }

    // Best-effort cleanup of superseded icons for this buff (same base id,
    // different content hash). Files we did not generate are left alone.
    let prefix = format!("{base_id}_");
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(&prefix) && name != file_name && is_valid_icon_file_name(name) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(file_name)
}

fn delete_buff_icon_from(dir: &Path, file_name: &str) -> Result<(), String> {
    if !is_valid_icon_file_name(file_name) {
        return Err(format!("invalid icon file name: {file_name}"));
    }
    let path = dir.join(file_name);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove {}: {e}", path.display())),
    }
}

pub mod commands {
    use super::*;

    /// Returns the buff-icon directory as a webview-ready prefix (forward
    /// slashes, trailing `/`) for `convertFileSrc` concatenation.
    #[tauri::command]
    #[specta::specta]
    pub fn buff_icon_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
        Ok(normalize_dir_prefix(&buff_icon_dir_path(&app_handle)?))
    }

    /// Copies `source_path` into the buff-icon directory under a
    /// content-addressed name, removing superseded icons for the same buff.
    /// Returns the stored file name for persistence in settings.
    #[tauri::command]
    #[specta::specta]
    pub fn import_buff_icon(
        app_handle: tauri::AppHandle,
        base_id: u32,
        source_path: String,
    ) -> Result<String, String> {
        let dir = buff_icon_dir_path(&app_handle)?;
        import_buff_icon_into(&dir, base_id, Path::new(&source_path))
    }

    /// Deletes a previously imported icon. Missing files are treated as
    /// already deleted.
    #[tauri::command]
    #[specta::specta]
    pub fn delete_buff_icon(app_handle: tauri::AppHandle, file_name: String) -> Result<(), String> {
        let dir = buff_icon_dir_path(&app_handle)?;
        delete_buff_icon_from(&dir, &file_name)
    }
}
