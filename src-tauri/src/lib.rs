mod commands;

use commands::{
    call_ai, call_ai_chat, create_file, fetch_report_from_url, list_ai_models, save_export_file,
    test_ai_connection,
};
use log::LevelFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_file,
            fetch_report_from_url,
            call_ai,
            call_ai_chat,
            test_ai_connection,
            list_ai_models,
            save_export_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
