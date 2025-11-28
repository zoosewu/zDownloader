// HTTP client wrapper using reqwest
use crate::models::ProxyInfo;
use reqwest::{header, Client, Proxy};
use std::time::Duration;

pub struct HttpClient {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    pub supports_range: bool,
    pub content_type: Option<String>,
}

impl HttpClient {
    pub fn new() -> Self {
        Self::with_proxy(None)
    }

    /// Create HTTP client with optional proxy
    pub fn with_proxy(proxy_info: Option<&ProxyInfo>) -> Self {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Zoo-Downloader/0.1.0");

        // Add proxy if provided
        if let Some(proxy) = proxy_info {
            let proxy_url = format!("http://{}:{}", proxy.host, proxy.port);
            if let Ok(reqwest_proxy) = Proxy::http(&proxy_url) {
                builder = builder.proxy(reqwest_proxy);
            } else {
                eprintln!("Warning: Failed to configure proxy {}", proxy_url);
            }
        }

        let client = builder
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Check if server supports range requests by sending HEAD request
    /// Returns file info including size and whether range requests are supported
    pub async fn check_range_support(&self, url: &str) -> Result<FileInfo, String> {
        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| format!("HEAD request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "HEAD request failed with status: {}",
                response.status()
            ));
        }

        // Check Content-Length header
        let size = response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or("Missing or invalid Content-Length header")?;

        // Check Accept-Ranges header
        let supports_range = response
            .headers()
            .get(header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v != "none")
            .unwrap_or(false);

        // Get Content-Type
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        Ok(FileInfo {
            size,
            supports_range,
            content_type,
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
