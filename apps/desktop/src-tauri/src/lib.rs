mod commands;
mod dto;
mod error;
mod state;

use state::AppState;
use tauri::Manager;
use tauri::Wry;

pub fn run() {
    tauri::Builder::<Wry>::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("reqwest", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init::<Wry>())
        .manage(AppState::new())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            log::info!("PkgSeal started: {}", app.package_info().name);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_health,
            commands::app::source_availability,
            commands::search::search_packages,
            commands::search::get_package_details,
            commands::search::get_installed_packages,
            commands::resolver::resolve_applications_command,
            commands::resolver::get_resolver_config,
            commands::policy::evaluate_policy,
            commands::policy::list_policy_presets,
            commands::transaction::preview_transaction,
            commands::transaction::validate_transaction_request,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| panic!("error while running tauri application: {e}"));
}
