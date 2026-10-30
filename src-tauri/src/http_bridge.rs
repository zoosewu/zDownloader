// HTTP bridge for development - allows browser to communicate with Tauri backend
use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

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
struct StringError(#[allow(dead_code)] String); // read via Debug in warp rejection logs
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

    let routes = dev_origin_only().and(health.or(invoke)).with(cors);

    // Loopback by default. Docker needs 0.0.0.0 inside the container; opt in explicitly
    // and publish the port as 127.0.0.1:3030:3030 so it stays off the LAN.
    let host: std::net::IpAddr = std::env::var("TAURI_HTTP_BRIDGE_HOST")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or(std::net::IpAddr::from([127, 0, 0, 1]));

    println!("[HTTP Bridge] Server ready at http://{}:{}", host, port);
    println!("[HTTP Bridge] Endpoint: POST /invoke");
    println!("[HTTP Bridge] Health check: GET /health");

    warp::serve(routes).run((host, port)).await;
}

/// Only the local Vite dev page (or non-browser clients) may drive the bridge;
/// any other web page could otherwise invoke commands and read the clipboard.
fn dev_origin_only() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("origin")
        .and_then(|origin: Option<String>| async move {
            match origin {
                Some(o)
                    if !(o.starts_with("http://localhost:")
                        || o.starts_with("http://127.0.0.1:")) =>
                {
                    Err(warp::reject::custom(StringError(format!(
                        "Origin not allowed: {}",
                        o
                    ))))
                }
                _ => Ok(()),
            }
        })
        .untuple_one()
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

        "settings_get" => {
            let settings = app.state::<crate::commands::settings::SettingsState>().settings.lock().await.clone();
            serde_json::to_value(settings).map_err(|e| e.to_string())
        }

        "downloads_list" => {
            let tasks = app.state::<crate::downloads::Downloads>().manager.list().await;
            tasks.map_err(|e| e.to_string()).and_then(|t| serde_json::to_value(t).map_err(|e| e.to_string()))
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
