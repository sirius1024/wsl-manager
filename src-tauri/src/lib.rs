mod commands;
mod logger;
mod models;
mod wsl;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::log("WSL Manager starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::log_message,
            commands::list_instances,
            commands::get_wsl_version,
            commands::list_online_distributions,
            commands::install_distribution,
            commands::rename_instance,
            commands::start_instance,
            commands::stop_instance,
            commands::shutdown,
            commands::open_terminal,
            commands::set_default_instance,
            commands::delete_instance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
