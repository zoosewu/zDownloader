// Download engine core - single-threaded download for MVP
use crate::models::{DownloadTask, DownloadStatus};
use crate::download::HttpClient;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct DownloadEngine {
    http_client: HttpClient,
}

impl DownloadEngine {
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::new(),
        }
    }

    pub async fn download_file<F>(
        &self,
        task: &mut DownloadTask,
        mut progress_callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(&DownloadTask) + Send,
    {
        // Update status to downloading
        task.status = DownloadStatus::Downloading;
        progress_callback(task);

        // Get the file
        let response = self.http_client.get_client()
            .get(&task.url)
            .send()
            .await?;

        if !response.status().is_success() {
            task.status = DownloadStatus::Failed;
            progress_callback(task);
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        // Get total size
        if let Some(content_length) = response.content_length() {
            task.filesize = content_length;
        }

        // Create file
        let file_path = Path::new(&task.filepath);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(file_path)?;

        // Download with progress tracking
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        let mut last_progress_update = 0.0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            // Update progress
            if task.filesize > 0 {
                task.progress = (downloaded as f64 / task.filesize as f64) * 100.0;

                // Emit progress event every 1% change to avoid flooding
                if task.progress - last_progress_update >= 1.0 {
                    progress_callback(task);
                    last_progress_update = task.progress;
                }
            }
        }

        task.status = DownloadStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());
        progress_callback(task);

        Ok(())
    }
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new()
    }
}
