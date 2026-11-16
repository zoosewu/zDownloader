//! App-level commands.

use tauri::State;

use crate::downloads::Downloads;

/// Called after an update has been downloaded, right before it is installed: the installer
/// ends this process without the usual exit, so stop the downloads and save their state now.
/// Running downloads continue after the restart.
#[tauri::command]
pub async fn app_prepare_update(downloads: State<'_, Downloads>) -> Result<(), ()> {
    log::info!("stopping downloads for the update");
    downloads.shutdown().await;
    Ok(())
}
