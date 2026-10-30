// HTTP server for Chrome extension integration
// Provides a REST API for the extension to communicate with the app

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use warp::Filter;

#[derive(Debug, Deserialize)]
struct AddDownloadRequest {
    url: String,
    filename: Option<String>,
}

#[derive(Debug, Serialize)]
struct AddDownloadResponse {
    success: bool,
    task_id: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    version: String,
}

/// Check if a port is available for binding
fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Start HTTP server for Chrome extension communication
pub async fn start_http_server(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let port = 45677; // Fixed port for Chrome extension to connect to

    // Check if port is available before starting server
    if !is_port_available(port) {
        let error_msg = format!(
            "Cannot start Native Messaging HTTP server: port {} is already in use.\n\
             This is normal if:\n\
             - Another instance of Zoo Downloader is running\n\
             - Chrome extension is still connected\n\
             \n\
             The application will continue without Native Messaging support.\n\
             To disable this check, set DISABLE_NATIVE_MESSAGING_SERVER=1",
            port
        );
        log::warn!("{}", error_msg);
        return Err(error_msg.into());
    }

    // CORS only shapes what a browser may read back; extension_origin_only() is the actual gate.
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    let app_handle = Arc::new(app);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&StatusResponse {
                status: "ok".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
        });

    // Add download endpoint
    let add_download = warp::path("add-download")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app_handle.clone()))
        .and_then(handle_add_download);

    let routes = extension_origin_only()
        .and(health.or(add_download))
        .with(cors)
        .recover(handle_rejection)
        .with(warp::log("zoo_downloader::http_server"));

    log::info!("Starting HTTP server for Chrome extension on port {}", port);

    // Start server in background
    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    });

    Ok(())
}

#[derive(Debug)]
struct ForbiddenOrigin;
impl warp::reject::Reject for ForbiddenOrigin {}

/// Reject requests coming from web pages.
///
/// Browsers always attach `Origin` to cross-origin POSTs, including CORS "simple"
/// requests that run the handler even though the page can't read the response.
/// The extension sends `chrome-extension://<id>`; non-browser local clients send none.
fn extension_origin_only() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("origin")
        .and_then(|origin: Option<String>| async move {
            match origin {
                Some(o) if !o.starts_with("chrome-extension://") => {
                    log::warn!("Rejected request from origin {}", o);
                    Err(warp::reject::custom(ForbiddenOrigin))
                }
                _ => Ok(()),
            }
        })
        .untuple_one()
}

async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    use warp::http::StatusCode;
    let status = if err.find::<ForbiddenOrigin>().is_some() {
        StatusCode::FORBIDDEN
    } else if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_REQUEST
    };
    Ok(warp::reply::with_status(
        status.canonical_reason().unwrap_or("error"),
        status,
    ))
}

fn with_app(
    app: Arc<AppHandle>,
) -> impl Filter<Extract = (Arc<AppHandle>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}

async fn handle_add_download(
    req: AddDownloadRequest,
    app: Arc<AppHandle>,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("Received add-download request for {}", req.url);

    let dir = app
        .state::<crate::commands::settings::SettingsState>()
        .settings
        .lock()
        .await
        .default_download_dir
        .clone();
    let request = zoo_core::AddRequest {
        url: req.url,
        filename: req.filename,
        dir,
        start: zoo_core::StartMode::Now,
    };
    let reply = match app.state::<crate::downloads::Downloads>().manager.add(request).await {
        Ok(task) => AddDownloadResponse {
            success: true,
            task_id: Some(task.id.to_string()),
            message: "Download queued".to_string(),
        },
        Err(e) => {
            log::warn!("Rejected add-download request: {e}");
            AddDownloadResponse { success: false, task_id: None, message: e.to_string() }
        }
    };
    Ok(warp::reply::json(&reply))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn status_for(origin: Option<&str>) -> u16 {
        let route = extension_origin_only()
            .map(|| "ok")
            .recover(handle_rejection);
        let mut req = warp::test::request().method("POST").path("/add-download");
        if let Some(o) = origin {
            req = req.header("origin", o);
        }
        req.reply(&route).await.status().as_u16()
    }

    #[tokio::test]
    async fn rejects_web_page_origins() {
        assert_eq!(status_for(Some("https://evil.example")).await, 403);
        assert_eq!(status_for(Some("null")).await, 403);
        assert_eq!(status_for(Some("http://127.0.0.1:45677")).await, 403);
    }

    #[tokio::test]
    async fn allows_extension_and_non_browser_clients() {
        assert_eq!(status_for(Some("chrome-extension://abcdefghijklmnop")).await, 200);
        assert_eq!(status_for(None).await, 200);
    }
}
