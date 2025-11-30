// Native Messaging module for Chrome extension integration

pub mod http_server;

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum NativeMessage {
    #[serde(rename = "PING")]
    Ping,
    #[serde(rename = "ADD_DOWNLOAD")]
    AddDownload {
        url: String,
        filename: String,
        referer: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum NativeResponse {
    #[serde(rename = "PONG")]
    Pong,
    #[serde(rename = "DOWNLOAD_STARTED")]
    DownloadStarted { filename: String },
    #[serde(rename = "ERROR")]
    Error { message: String },
}

/// Read a message from stdin using Chrome Native Messaging protocol
/// Message format: 4-byte length (little-endian) + JSON payload
fn read_message() -> io::Result<NativeMessage> {
    let mut length_bytes = [0u8; 4];
    io::stdin().read_exact(&mut length_bytes)?;

    let length = u32::from_le_bytes(length_bytes) as usize;

    let mut buffer = vec![0u8; length];
    io::stdin().read_exact(&mut buffer)?;

    let message = serde_json::from_slice(&buffer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(message)
}

/// Write a message to stdout using Chrome Native Messaging protocol
fn write_message(response: &NativeResponse) -> io::Result<()> {
    let json = serde_json::to_string(response)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let length = json.len() as u32;
    io::stdout().write_all(&length.to_le_bytes())?;
    io::stdout().write_all(json.as_bytes())?;
    io::stdout().flush()?;

    Ok(())
}

/// Handle a native messaging message
async fn handle_message(app: &AppHandle, message: NativeMessage) -> NativeResponse {
    match message {
        NativeMessage::Ping => NativeResponse::Pong,

        NativeMessage::AddDownload {
            url,
            filename,
            referer: _,
        } => {
            // Create download task using existing commands
            match crate::commands::download::create_download(
                url.clone(),
                Some(filename.clone()),
                app.state(),
            )
            .await
            {
                Ok(task_id) => {
                    // Start download immediately
                    match crate::commands::download::start_download(
                        task_id.clone(),
                        app.state(),
                        app.clone(),
                    )
                    .await
                    {
                        Ok(_) => {
                            log::info!(
                                "Download started from Chrome extension: {} ({})",
                                filename,
                                task_id
                            );
                            NativeResponse::DownloadStarted { filename }
                        }
                        Err(e) => {
                            log::error!("Failed to start download: {}", e);
                            NativeResponse::Error {
                                message: format!("Failed to start download: {}", e),
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create download task: {}", e);
                    NativeResponse::Error {
                        message: format!("Failed to create download: {}", e),
                    }
                }
            }
        }
    }
}

/// Start the native messaging host
/// This function blocks and processes messages from stdin
pub async fn run_native_messaging_host(app: AppHandle) -> io::Result<()> {
    log::info!("Native messaging host started");

    loop {
        match read_message() {
            Ok(message) => {
                log::debug!("Received message: {:?}", message);

                let response = handle_message(&app, message).await;

                if let Err(e) = write_message(&response) {
                    log::error!("Failed to write response: {}", e);
                    break;
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    log::info!("Chrome extension disconnected");
                    break;
                }
                log::error!("Failed to read message: {}", e);
                break;
            }
        }
    }

    log::info!("Native messaging host stopped");
    Ok(())
}

/// Check if the app is running as a native messaging host
/// Chrome passes the origin as a command line argument when launching the host
pub fn is_native_messaging_mode() -> bool {
    std::env::args().any(|arg| arg.starts_with("chrome-extension://"))
}
