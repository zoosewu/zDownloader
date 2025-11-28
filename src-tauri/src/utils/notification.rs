use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Send download complete notification
/// Uses tauri-plugin-notification to show system notification
pub fn send_download_complete_notification(
    app_handle: &AppHandle,
    _filename: &str,
    file_size: u64,
) -> Result<(), String> {
    let notification = app_handle.notification();

    let size_mb = file_size as f64 / (1024.0 * 1024.0);
    let body = format!("文件大小: {:.2} MB", size_mb);

    notification
        .builder()
        .title("下載完成")
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to show notification: {}", e))?;

    Ok(())
}

/// Send download error notification
pub fn send_download_error_notification(
    app_handle: &AppHandle,
    filename: &str,
    error_message: &str,
) -> Result<(), String> {
    let notification = app_handle.notification();

    notification
        .builder()
        .title(format!("下載失敗: {}", filename))
        .body(error_message)
        .show()
        .map_err(|e| format!("Failed to show notification: {}", e))?;

    Ok(())
}

/// Send batch download complete notification
pub fn send_batch_complete_notification(
    app_handle: &AppHandle,
    total_count: usize,
    success_count: usize,
) -> Result<(), String> {
    let notification = app_handle.notification();

    let body = format!("成功: {} / 總計: {}", success_count, total_count);

    notification
        .builder()
        .title("批次下載完成")
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to show notification: {}", e))?;

    Ok(())
}
