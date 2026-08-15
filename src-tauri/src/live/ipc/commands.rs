use crate::WINDOW_LIVE_LABEL;
use crate::live::bootstrap_snapshot::{MonitorRuntimeSnapshot, save_monitor_runtime_snapshot};
use crate::live::ipc::models::{
    LiveBuffsPayload, LiveCombatPayload, LiveDeathsPayload, LiveFantasyPayload, LiveMonsterPayload,
    LiveScenePayload, LiveStatusPayload,
};
use crate::live::runtime_handle::LiveRuntimeHandle;
use tauri::Manager;
use window_vibrancy::{apply_blur, clear_blur};

/// Bootstrap for the `live-combat` topic. Topic payloads are sliced out of the
/// runtime snapshot so a window can hydrate before its first pushed event.
#[tauri::command]
#[specta::specta]
pub async fn get_live_combat(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveCombatPayload, String> {
    runtime.combat().await
}

/// Bootstrap for the `live-status` topic.
#[tauri::command]
#[specta::specta]
pub async fn get_live_status(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveStatusPayload, String> {
    runtime.status().await
}

/// Bootstrap for the `live-buffs` topic.
#[tauri::command]
#[specta::specta]
pub async fn get_live_buffs(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveBuffsPayload, String> {
    runtime.buffs().await
}

/// Bootstrap for the `live-monster` topic.
#[tauri::command]
#[specta::specta]
pub async fn get_live_monster(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveMonsterPayload, String> {
    runtime.monster().await
}

/// Bootstrap for the `live-fantasy` topic.
#[tauri::command]
#[specta::specta]
pub async fn get_live_fantasy(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveFantasyPayload, String> {
    runtime.fantasy().await
}

/// Bootstrap for the `live-deaths` topic.
#[tauri::command]
#[specta::specta]
pub async fn get_live_deaths(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveDeathsPayload, String> {
    runtime.deaths().await
}

/// Bootstrap for the `live-scene` topic. `main`-only: drives the daily-scene
/// auto-hide logic for the game/monster/minimap overlay windows without
/// subscribing to the far heavier `live-combat` cadence.
#[tauri::command]
#[specta::specta]
pub async fn get_live_scene(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<LiveScenePayload, String> {
    runtime.scene().await
}

#[tauri::command]
#[specta::specta]
pub fn enable_blur(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        let _ = apply_blur(&window, Some((10, 10, 10, 50)));
    }
}

#[tauri::command]
#[specta::specta]
pub fn disable_blur(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        let _ = clear_blur(&window);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn reset_encounter(runtime: tauri::State<'_, LiveRuntimeHandle>) -> Result<(), String> {
    runtime.manual_reset().await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_pause_encounter(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<(), String> {
    runtime.toggle_pause().await
}

#[tauri::command]
#[specta::specta]
pub async fn start_training_dummy(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<(), String> {
    runtime.start_training().await
}

#[tauri::command]
#[specta::specta]
pub async fn stop_training_dummy(
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<(), String> {
    runtime.stop_training().await
}

#[tauri::command]
#[specta::specta]
pub async fn save_and_apply_monitor_runtime_snapshot(
    snapshot: MonitorRuntimeSnapshot,
    app_handle: tauri::AppHandle,
    runtime: tauri::State<'_, LiveRuntimeHandle>,
) -> Result<(), String> {
    let snapshot = snapshot.normalize()?;
    save_monitor_runtime_snapshot(&app_handle, &snapshot)?;
    runtime.apply_monitor_config(snapshot).await
}
