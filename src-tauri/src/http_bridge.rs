// HTTP bridge for development - allows browser to communicate with Tauri backend
use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
struct InvokeRequest {
    cmd: String,
    args: Option<Value>,
}

#[derive(Debug, Serialize)]
struct InvokeResponse {
    success: bool,
    data: Option<Value>,
    error: Option<String>,
}

// Custom rejection type for warp
#[derive(Debug)]
struct StringError(String);
impl warp::reject::Reject for StringError {}

/// Start HTTP bridge server for development
/// This allows browser-based frontend to communicate with Tauri backend running in Docker
pub async fn start_http_bridge(app: AppHandle, port: u16) {
    println!("[HTTP Bridge] Starting on port {}", port);

    let app_filter = warp::any().map(move || app.clone());

    // CORS configuration for development
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    // Main invoke endpoint that mimics Tauri's invoke
    let invoke = warp::path("invoke")
        .and(warp::post())
        .and(warp::body::json())
        .and(app_filter.clone())
        .and_then(handle_invoke);

    let routes = health.or(invoke).with(cors);

    println!("[HTTP Bridge] Server ready at http://0.0.0.0:{}", port);
    println!("[HTTP Bridge] Endpoint: POST /invoke");
    println!("[HTTP Bridge] Health check: GET /health");

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn handle_invoke(
    request: InvokeRequest,
    app: AppHandle,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("[HTTP Bridge] Received command: {}", request.cmd);

    // Route to appropriate command handler
    let result = match request.cmd.as_str() {
        "greet" => {
            let name = request.args
                .as_ref()
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Guest");

            Ok(serde_json::json!(format!("Hello, {}! You've been greeted from Rust!", name)))
        }

        "get_clipboard_content" => {
            use tauri_plugin_clipboard_manager::ClipboardExt;

            match app.clipboard().read_text() {
                Ok(content) => Ok(serde_json::json!(content)),
                Err(e) => Err(format!("Failed to read clipboard: {}", e)),
            }
        }

        "detect_url" => {
            // For now, return a simple validation
            let url = extract_string_param(&request.args, "url")
                .map_err(StringError)?;

            let is_valid = url.starts_with("http://") || url.starts_with("https://");

            let detection_result = serde_json::json!({
                "url": url,
                "isValid": is_valid,
                "detectorName": "builtin",
                "detectionMode": "http_bridge",
            });

            Ok(detection_result)
        }

        "get_settings" => {
            // Return default settings for development
            let settings = serde_json::json!({
                "theme": "Light",
                "defaultDownloadDir": "/tmp/downloads",
                "maxConcurrentDownloads": 3,
                "enableClipboardMonitoring": true,
                "autoStart": false,
                "minimizeToTray": false,
            });

            Ok(settings)
        }

        "get_download_list" => {
            // Return empty list for development
            Ok(serde_json::json!([]))
        }

        "create_download_with_path" | "start_download" | "pause_download" | "cancel_download" => {
            // These commands need full Tauri context
            // For development, just return success
            Ok(serde_json::json!("dev-mode-success"))
        }

        _ => {
            Err(format!("Unknown command: {}", request.cmd))
        }
    };

    let response = match result {
        Ok(data) => InvokeResponse {
            success: true,
            data: Some(data),
            error: None,
        },
        Err(error) => InvokeResponse {
            success: false,
            data: None,
            error: Some(error),
        },
    };

    Ok(warp::reply::json(&response))
}

fn extract_string_param(args: &Option<Value>, key: &str) -> Result<String, String> {
    args.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| format!("Missing '{}' parameter", key))
}
