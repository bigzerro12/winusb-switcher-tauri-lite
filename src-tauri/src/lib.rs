//! WinUSB Switcher Lite — Tauri application entry point.

mod commands;
mod bundled_jlink;
mod error;
mod jlink;
mod platform;
mod process;
mod state;

use commands::probe;
use state::JLinkState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_bin = platform::config().jlink_bin;

    tauri::Builder::default()
        .manage(JLinkState::new(default_bin))
        .setup(|app| {
            // Lite: ship a pinned J-Link distribution and make it available immediately.
            // This runs before the frontend starts invoking commands.
            let _ = crate::bundled_jlink::ensure_extracted_and_on_path(&app.handle());
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("winusb_switcher_lite_lib", log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Probe
            probe::detect_and_scan,
            probe::scan_probes,
            probe::switch_usb_driver,
            probe::get_arch_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}