//! Proxy pool commands.

use crate::commands::error::{CmdResult, CommandError};
use crate::downloads::Downloads;
use tauri::State;
use zoo_core::proxy_pool::{ImportResult, ProxyInfo};

/// Import `host:port` (or `http://host:port`) lines from a text file; `#` comments and blank lines are ignored.
#[tauri::command]
pub async fn proxies_import(file_path: String, downloads: State<'_, Downloads>) -> CmdResult<ImportResult> {
    let text = std::fs::read_to_string(&file_path)
        .map_err(|e| CommandError::new("io", format!("cannot read {file_path}: {e}")))?;
    Ok(downloads.pool.import(&text))
}

#[tauri::command]
pub async fn proxies_list(downloads: State<'_, Downloads>) -> CmdResult<Vec<ProxyInfo>> {
    Ok(downloads.pool.list())
}

#[tauri::command]
pub async fn proxies_remove(id: String, downloads: State<'_, Downloads>) -> CmdResult<()> {
    let id = uuid::Uuid::parse_str(&id).map_err(|_| CommandError::invalid(format!("not a proxy id: {id}")))?;
    Ok(downloads.pool.remove(id)?)
}

#[tauri::command]
pub async fn proxies_clear(downloads: State<'_, Downloads>) -> CmdResult<()> {
    downloads.pool.clear();
    Ok(())
}

/// Give failed and blacklisted proxies another chance.
#[tauri::command]
pub async fn proxies_reset_failed(downloads: State<'_, Downloads>) -> CmdResult<()> {
    downloads.pool.reset_failed();
    Ok(())
}
