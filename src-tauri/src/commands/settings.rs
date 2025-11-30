// Settings command handlers
use crate::models::settings::AppSettings;
use crate::storage::paths::get_config_file;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// Settings state wrapper for Tauri state management
pub struct SettingsState {
    pub settings: Arc<Mutex<AppSettings>>,
}

/// Load settings from config file or return default
///
/// Automatically creates config file with defaults if it doesn't exist
pub fn load_settings_from_file() -> AppSettings {
    let config_path = get_config_file();

    // Try to load existing settings
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    println!("Settings loaded from: {:?}", config_path);
                    return settings;
                }
                Err(e) => {
                    eprintln!("Failed to parse settings file: {}. Using defaults.", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to read settings file: {}. Using defaults.", e);
            }
        }
    }

    // Return default settings
    let default_settings = AppSettings::default();

    // Try to save default settings to file
    if let Err(e) = save_settings_to_file(&default_settings) {
        eprintln!("Warning: Failed to save default settings: {}", e);
    }

    default_settings
}

/// Save settings to config file
fn save_settings_to_file(settings: &AppSettings) -> Result<(), String> {
    let config_path = get_config_file();

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Serialize settings to JSON
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to file
    fs::write(&config_path, json).map_err(|e| format!("Failed to write settings file: {}", e))?;

    println!("Settings saved to: {:?}", config_path);
    Ok(())
}

/// Get current settings
#[tauri::command]
pub async fn get_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

/// Update settings
///
/// Merges the provided settings with current settings and saves to file
#[tauri::command]
pub async fn update_settings(
    state: State<'_, SettingsState>,
    new_settings: AppSettings,
) -> Result<AppSettings, String> {
    // Update settings in state
    {
        let mut settings = state.settings.lock().await;
        *settings = new_settings.clone();
    }

    // Save to file
    save_settings_to_file(&new_settings)?;

    Ok(new_settings)
}

/// Export settings to a specified file path
#[tauri::command]
pub async fn export_settings(
    state: State<'_, SettingsState>,
    export_path: String,
) -> Result<(), String> {
    let settings = state.settings.lock().await;
    let path = PathBuf::from(export_path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create export directory: {}", e))?;
    }

    // Serialize settings to JSON
    let json = serde_json::to_string_pretty(&*settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to file
    fs::write(&path, json).map_err(|e| format!("Failed to write export file: {}", e))?;

    println!("Settings exported to: {:?}", path);
    Ok(())
}

/// Import settings from a specified file path
///
/// Validates the imported settings before applying them
#[tauri::command]
pub async fn import_settings(
    state: State<'_, SettingsState>,
    import_path: String,
) -> Result<AppSettings, String> {
    let path = PathBuf::from(import_path);

    // Check if file exists
    if !path.exists() {
        return Err(format!("Import file not found: {:?}", path));
    }

    // Read file content
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;

    // Parse settings
    let imported_settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    // Update settings in state
    {
        let mut settings = state.settings.lock().await;
        *settings = imported_settings.clone();
    }

    // Save to config file
    save_settings_to_file(&imported_settings)?;

    println!("Settings imported from: {:?}", path);
    Ok(imported_settings)
}

/// Reset settings to default values
#[tauri::command]
pub async fn reset_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let default_settings = AppSettings::default();

    // Update settings in state
    {
        let mut settings = state.settings.lock().await;
        *settings = default_settings.clone();
    }

    // Save to file
    save_settings_to_file(&default_settings)?;

    println!("Settings reset to defaults");
    Ok(default_settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "zh-TW");
        assert_eq!(settings.segment_size_mb, 10);
        assert_eq!(settings.thread_count, 8);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.language, settings.language);
    }
}
