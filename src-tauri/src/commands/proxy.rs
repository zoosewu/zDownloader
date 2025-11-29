// Proxy command handlers - Tauri commands for proxy management
use crate::commands::download::AppState;
use crate::models::ProxyInfo;
use crate::proxy::cache::import_proxies_from_file;
use tauri::State;
use uuid::Uuid;

/// Import proxy list from a file
/// Expected file format: one proxy per line in format "host:port"
/// Lines starting with # are treated as comments
#[tauri::command]
pub async fn import_proxy_list(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    // Import proxies from file
    let proxies = import_proxies_from_file(&file_path)?;
    let count = proxies.len();

    // Add to proxy pool
    state.proxy_pool.add_proxies(proxies).await;

    Ok(count)
}

/// Get all proxies with their current status and health scores
#[tauri::command]
pub async fn get_proxy_list(state: State<'_, AppState>) -> Result<Vec<ProxyInfo>, String> {
    let proxies = state.proxy_pool.get_all_proxies().await;
    Ok(proxies)
}

/// Remove a specific proxy from the pool by its ID
#[tauri::command]
pub async fn remove_proxy(
    proxy_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&proxy_id).map_err(|e| e.to_string())?;
    state.proxy_pool.remove_proxy(uuid).await?;
    Ok(())
}

/// Clear all proxies from the pool
#[tauri::command]
pub async fn clear_all_proxies(state: State<'_, AppState>) -> Result<(), String> {
    state.proxy_pool.clear().await;
    Ok(())
}

/// Get count of active proxies
#[tauri::command]
pub async fn get_active_proxy_count(state: State<'_, AppState>) -> Result<usize, String> {
    let count = state.proxy_pool.count_active().await;
    Ok(count)
}

/// Reset all failed proxies to active status
#[tauri::command]
pub async fn reset_failed_proxies(state: State<'_, AppState>) -> Result<(), String> {
    state.proxy_pool.reset_failed_proxies().await;
    Ok(())
}
