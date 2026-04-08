// src-tauri/src/lib.rs

mod bungie;
mod tracker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            bungie::greet,
            bungie::bungie_status,
            bungie::search_player,
            bungie::get_profile,
            bungie::get_current_activity,
            tracker::get_tracker_snapshot
        ])
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}