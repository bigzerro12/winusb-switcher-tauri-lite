//! WinUSB Switcher Lite–only commands.

use tauri::AppHandle;

/// Extract bundled J-Link to the user profile (if needed) and prepend it to PATH.
/// Runs blocking work on the blocking thread pool so the UI can load first.
#[tauri::command]
pub async fn prepare_bundled_jlink(app: AppHandle) -> Result<String, String> {
    let app = app.clone();
    tokio::task::spawn_blocking(move || crate::bundled_jlink::ensure_extracted_and_on_path(&app))
        .await
        .map_err(|e| e.to_string())?
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}
