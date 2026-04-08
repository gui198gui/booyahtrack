// src-tauri/src/main.rs

mod bungie;
mod tracker;

use tauri::{LogicalPosition, LogicalSize, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(tracker::AppState::default())
        .setup(|app| {
            let overlay = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("/overlay".into()),
            )
            .title("overlay")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .resizable(false)
            .visible(true)
            .focused(false)
            .skip_taskbar(true)
            .accept_first_mouse(false)
            .content_protected(false)
            .inner_size(220.0, 70.0)
            .position(20.0, 20.0)
            .build()?;

            let _ = overlay.set_shadow(false);
            let _ = overlay.set_size(LogicalSize::new(220.0, 70.0));
            let _ = overlay.set_position(LogicalPosition::new(20.0, 20.0));

            let toast = WebviewWindowBuilder::new(
                app,
                "toast",
                WebviewUrl::App("/toast".into()),
            )
            .title("toast")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .resizable(false)
            .visible(true)
            .focused(false)
            .skip_taskbar(true)
            .accept_first_mouse(false)
            .content_protected(false)
            .inner_size(360.0, 120.0)
            .position(1500.0, 900.0)
            .build()?;

            let _ = toast.set_shadow(false);
            let _ = toast.set_size(LogicalSize::new(360.0, 120.0));
            let _ = toast.set_position(LogicalPosition::new(1500.0, 900.0));

            tracker::start_tracker_poller(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bungie::greet,
            bungie::bungie_status,
            bungie::search_player,
            bungie::get_profile,
            bungie::get_current_activity,
            tracker::get_tracker_snapshot,
            tracker::get_todays_raid_history,
            tracker::set_active_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}