// HTTP server for Chrome extension integration
// Provides a REST API for the extension to communicate with the app

use crate::commands::download::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use warp::Filter;

#[derive(Debug, Deserialize)]
struct AddDownloadRequest {
    url: String,
    filename: Option<String>,
    referer: Option<String>,
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

/// Start HTTP server for Chrome extension communication
pub async fn start_http_server(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let port = 45677; // Fixed port for Chrome extension to connect to

    // CORS configuration - allow requests from Chrome extension
    let cors = warp::cors()
        .allow_any_origin() // In production, restrict to chrome-extension:// origins
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

    let routes = health
        .or(add_download)
        .with(cors)
        .with(warp::log("zoo_downloader::http_server"));

    log::info!("Starting HTTP server for Chrome extension on port {}", port);

    // Start server in background
    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    });

    Ok(())
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
    log::info!("Received add-download request: {:?}", req);

    // Get app state
    let state = app.state::<AppState>();

    // Create download task
    match crate::commands::download::create_download(req.url.clone(), req.filename, state.clone())
        .await
    {
        Ok(task_id) => {
            // Start download immediately
            match crate::commands::download::start_download(task_id.clone(), state, app.as_ref().clone())
                .await
            {
                Ok(_) => {
                    log::info!("Download started: {}", task_id);
                    Ok(warp::reply::json(&AddDownloadResponse {
                        success: true,
                        task_id: Some(task_id),
                        message: "Download started successfully".to_string(),
                    }))
                }
                Err(e) => {
                    log::error!("Failed to start download: {}", e);
                    Ok(warp::reply::json(&AddDownloadResponse {
                        success: false,
                        task_id: None,
                        message: format!("Failed to start download: {}", e),
                    }))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create download task: {}", e);
            Ok(warp::reply::json(&AddDownloadResponse {
                success: false,
                task_id: None,
                message: format!("Failed to create download: {}", e),
            }))
        }
    }
}
