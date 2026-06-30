use crate::logger;
use crate::models::{WslInstance, WslVersion};
use crate::wsl;

#[tauri::command]
pub fn log_message(msg: String) {
    logger::log(&msg);
}

#[tauri::command]
pub async fn list_instances() -> Result<Vec<WslInstance>, String> {
    wsl::list_instances().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wsl_version() -> Result<WslVersion, String> {
    wsl::get_wsl_version().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_online_distributions() -> Result<Vec<String>, String> {
    wsl::list_online_distributions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_distribution(
    distro: String,
    install_name: String,
) -> Result<(), String> {
    wsl::install_distribution(&distro, &install_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_instance(old_name: String, new_name: String) -> Result<(), String> {
    wsl::rename_instance(&old_name, &new_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_instance(name: String) -> Result<(), String> {
    wsl::start_instance(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_instance(name: String) -> Result<(), String> {
    wsl::stop_instance(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn shutdown() -> Result<(), String> {
    wsl::shutdown().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_terminal(name: String) -> Result<(), String> {
    wsl::open_terminal(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_default_instance(name: String) -> Result<(), String> {
    wsl::set_default_instance(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_instance(name: String) -> Result<(), String> {
    wsl::delete_instance(&name).await.map_err(|e| e.to_string())
}
