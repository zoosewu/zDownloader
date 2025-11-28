// Download engine core - supports both single-threaded and multi-threaded segmented downloads
use crate::download::{
    calculate_segments, can_resume_download, get_resumable_segments, merge_segments,
    save_checkpoint, HttpClient, SegmentConfig,
};
use crate::models::{DownloadStatus, DownloadTask, SegmentStatus};
use crate::proxy::pool::ProxyPool;
use futures::StreamExt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

// Custom error type that implements Send
#[derive(Debug)]
pub struct DownloadError(String);

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DownloadError {}

impl From<String> for DownloadError {
    fn from(s: String) -> Self {
        DownloadError(s)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError(format!("IO error: {}", e))
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError(format!("HTTP error: {}", e))
    }
}

impl From<tokio::task::JoinError> for DownloadError {
    fn from(e: tokio::task::JoinError) -> Self {
        DownloadError(format!("Task join error: {}", e))
    }
}

pub struct DownloadEngine {
    http_client: HttpClient,
    segment_config: SegmentConfig,
    proxy_pool: Option<Arc<ProxyPool>>,
    use_proxy: bool,
}

impl DownloadEngine {
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::new(),
            segment_config: SegmentConfig::default(),
            proxy_pool: None,
            use_proxy: false,
        }
    }

    pub fn with_segment_config(segment_config: SegmentConfig) -> Self {
        Self {
            http_client: HttpClient::new(),
            segment_config,
            proxy_pool: None,
            use_proxy: false,
        }
    }

    /// Enable proxy support with a proxy pool
    pub fn with_proxy_pool(mut self, proxy_pool: Arc<ProxyPool>) -> Self {
        self.proxy_pool = Some(proxy_pool);
        self.use_proxy = true;
        self
    }

    /// Set whether to use proxy for downloads
    pub fn set_use_proxy(&mut self, use_proxy: bool) {
        self.use_proxy = use_proxy;
    }

    /// Get the best available proxy
    async fn get_proxy(&self) -> Option<crate::models::ProxyInfo> {
        if !self.use_proxy {
            return None;
        }

        if let Some(pool) = &self.proxy_pool {
            pool.get_best_proxy().await
        } else {
            None
        }
    }

    /// Download a file with automatic segmentation decision
    ///
    /// This method checks if the server supports range requests and if the file
    /// is large enough to benefit from segmented download. It then automatically
    /// chooses between single-threaded and multi-threaded download.
    ///
    /// If proxy pool is enabled, automatically retries with different proxies on failure.
    pub async fn download_file<F>(
        &self,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static + Clone,
    {
        // Update status to downloading
        task.status = DownloadStatus::Downloading;
        progress_callback(task);

        // Check if we can resume
        let temp_dir = self.get_temp_dir(task);
        let can_resume = can_resume_download(task, &temp_dir).unwrap_or(false);

        if can_resume {
            // Resume download
            return self.resume_download(task, progress_callback).await;
        }

        // If proxy is enabled, use retry logic
        if self.use_proxy && self.proxy_pool.is_some() {
            self.download_with_proxy_retry(task, progress_callback).await
        } else {
            // Direct download without proxy
            self.download_without_proxy(task, progress_callback).await
        }
    }

    /// Download with proxy retry logic
    async fn download_with_proxy_retry<F>(
        &self,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static + Clone,
    {
        const MAX_RETRIES: usize = 3;
        let mut retry_count = 0;
        let mut last_error = None;

        loop {
            // Get a proxy from the pool
            let proxy = match &self.proxy_pool {
                Some(pool) => pool.get_best_proxy().await,
                None => None,
            };

            if proxy.is_none() && retry_count > 0 {
                // No proxies available
                return Err("All proxies failed, no proxies available".to_string().into());
            }

            let proxy_id = proxy.as_ref().map(|p| p.id);
            let start_time = std::time::Instant::now();

            // Create HTTP client with this proxy
            let client = HttpClient::with_proxy(proxy.as_ref());

            // Attempt download
            let result = self.download_with_client(&client, task, progress_callback.clone()).await;

            let elapsed = start_time.elapsed();
            let latency_ms = elapsed.as_millis() as f64;

            match result {
                Ok(_) => {
                    // Success - mark proxy as successful
                    if let (Some(id), Some(pool)) = (proxy_id, &self.proxy_pool) {
                        let _ = pool.mark_success(id, latency_ms).await;
                    }
                    return Ok(());
                }
                Err(e) => {
                    // Failure - mark proxy as failed
                    if let (Some(id), Some(pool)) = (proxy_id, &self.proxy_pool) {
                        let _ = pool.mark_failure(id).await;
                    }

                    last_error = Some(e);
                    retry_count += 1;

                    if retry_count >= MAX_RETRIES {
                        break;
                    }

                    // Small delay before retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| "Download failed after retries".to_string().into()))
    }

    /// Download without proxy (direct connection)
    async fn download_without_proxy<F>(
        &self,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static + Clone,
    {
        self.download_with_client(&self.http_client, task, progress_callback).await
    }

    /// Download with a specific HTTP client
    async fn download_with_client<F>(
        &self,
        client: &HttpClient,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static + Clone,
    {
        // Check server capabilities
        let file_info = client
            .check_range_support(&task.url)
            .await
            .map_err(|e| format!("Failed to check server capabilities: {}", e))?;

        task.filesize = file_info.size;

        // Decide download strategy
        if file_info.supports_range
            && file_info.size >= crate::download::segment::MIN_FILE_SIZE_FOR_SEGMENTS
        {
            // Use multi-threaded segmented download
            let progress_arc = Arc::new(progress_callback);
            self.download_segmented_with_client(client, task, progress_arc).await
        } else {
            // Use single-threaded download
            self.download_single_thread_with_client(client, task, &progress_callback).await
        }
    }

    /// Single-threaded download with specific HTTP client
    async fn download_single_thread_with_client<F>(
        &self,
        client: &HttpClient,
        task: &mut DownloadTask,
        progress_callback: &F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync,
    {
        // Get the file
        let response = client.get_client().get(&task.url).send().await?;

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

    /// Multi-threaded segmented download with specific HTTP client
    async fn download_segmented_with_client<F>(
        &self,
        client: &HttpClient,
        task: &mut DownloadTask,
        progress_callback: Arc<F>,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static,
    {
        // Calculate segments
        let segments = calculate_segments(task.filesize, &self.segment_config);

        // Attach segments to task
        task.segments = segments
            .iter()
            .map(|s| s.clone().with_task_id(task.id))
            .collect();

        // Create temp directory for segments
        let temp_dir = self.get_temp_dir(task);
        std::fs::create_dir_all(&temp_dir)?;

        // Save checkpoint
        let checkpoint_path = temp_dir.join("checkpoint.json");
        save_checkpoint(task, &checkpoint_path)?;

        // Clone client for concurrent segments
        let client_arc = Arc::new(client.clone());

        // Download all segments concurrently
        self.download_all_segments_with_client(client_arc, task, &temp_dir, progress_callback.clone())
            .await?;

        // Merge segments
        let final_path = Path::new(&task.filepath);
        merge_segments(&task.segments, final_path, &temp_dir)
            .await
            .map_err(|e| format!("Failed to merge segments: {}", e))?;

        // Cleanup temp files
        crate::download::merger::cleanup_segment_files(&task.segments, &temp_dir)?;

        task.status = DownloadStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());
        progress_callback(task);

        Ok(())
    }

    /// Multi-threaded segmented download (legacy method using default client)
    async fn download_segmented<F>(
        &self,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static,
    {
        // Calculate segments
        let segments = calculate_segments(task.filesize, &self.segment_config);

        // Attach segments to task
        task.segments = segments
            .iter()
            .map(|s| s.clone().with_task_id(task.id))
            .collect();

        // Create temp directory for segments
        let temp_dir = self.get_temp_dir(task);
        std::fs::create_dir_all(&temp_dir)?;

        // Save checkpoint
        let checkpoint_path = temp_dir.join("checkpoint.json");
        save_checkpoint(task, &checkpoint_path)?;

        let progress_arc = Arc::new(progress_callback);

        // Download all segments concurrently
        self.download_all_segments(task, &temp_dir, progress_arc.clone())
            .await?;

        // Merge segments
        let final_path = Path::new(&task.filepath);
        merge_segments(&task.segments, final_path, &temp_dir)
            .await
            .map_err(|e| format!("Failed to merge segments: {}", e))?;

        // Cleanup temp files
        crate::download::merger::cleanup_segment_files(&task.segments, &temp_dir)?;

        task.status = DownloadStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());
        progress_arc(task);

        Ok(())
    }

    /// Download all segments concurrently with specific HTTP client
    async fn download_all_segments_with_client<F>(
        &self,
        http_client: Arc<HttpClient>,
        task: &mut DownloadTask,
        temp_dir: &Path,
        progress_callback: Arc<F>,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static,
    {
        let task_arc = Arc::new(Mutex::new(task.clone()));
        let temp_dir = Arc::new(temp_dir.to_path_buf());

        // Create a semaphore to limit concurrent downloads
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.segment_config.max_concurrent_segments,
        ));

        let mut handles = vec![];

        for segment_idx in 0..task.segments.len() {
            let task_clone = task_arc.clone();
            let http_client_clone = http_client.clone();
            let temp_dir_clone = temp_dir.clone();
            let progress_callback_clone = progress_callback.clone();
            let semaphore_clone = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let result = Self::download_segment(
                    task_clone.clone(),
                    segment_idx,
                    http_client_clone,
                    temp_dir_clone,
                    progress_callback_clone,
                )
                .await;

                result
            });

            handles.push(handle);
        }

        // Wait for all segments to complete
        for handle in handles {
            handle.await??;
        }

        // Update task with final segment statuses
        *task = task_arc.lock().await.clone();

        Ok(())
    }

    /// Download all segments concurrently (legacy method using default client)
    async fn download_all_segments<F>(
        &self,
        task: &mut DownloadTask,
        temp_dir: &Path,
        progress_callback: Arc<F>,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static,
    {
        let task_arc = Arc::new(Mutex::new(task.clone()));
        let http_client = Arc::new(self.http_client.clone());
        let temp_dir = Arc::new(temp_dir.to_path_buf());

        // Create a semaphore to limit concurrent downloads
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.segment_config.max_concurrent_segments,
        ));

        let mut handles = vec![];

        for segment_idx in 0..task.segments.len() {
            let task_clone = task_arc.clone();
            let http_client_clone = http_client.clone();
            let temp_dir_clone = temp_dir.clone();
            let progress_callback_clone = progress_callback.clone();
            let semaphore_clone = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let result = Self::download_segment(
                    task_clone.clone(),
                    segment_idx,
                    http_client_clone,
                    temp_dir_clone,
                    progress_callback_clone,
                )
                .await;

                result
            });

            handles.push(handle);
        }

        // Wait for all segments to complete
        for handle in handles {
            handle.await??;
        }

        // Update task with final segment statuses
        *task = task_arc.lock().await.clone();

        Ok(())
    }

    /// Download a single segment
    async fn download_segment<F>(
        task_arc: Arc<Mutex<DownloadTask>>,
        segment_idx: usize,
        http_client: Arc<HttpClient>,
        temp_dir: Arc<PathBuf>,
        progress_callback: Arc<F>,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync,
    {
        let segment = {
            let task = task_arc.lock().await;
            task.segments[segment_idx].clone()
        };

        // Update segment status
        {
            let mut task = task_arc.lock().await;
            task.segments[segment_idx].status = SegmentStatus::Downloading;
        }

        // Create segment file
        let segment_path = temp_dir.join(format!("segment_{}", segment.segment_id));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&segment_path)?;

        // Build range request
        let url = {
            let task = task_arc.lock().await;
            task.url.clone()
        };

        let range_header = format!("bytes={}-{}", segment.start_byte, segment.end_byte);
        let response = http_client
            .get_client()
            .get(&url)
            .header("Range", range_header)
            .send()
            .await?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            let mut task = task_arc.lock().await;
            task.segments[segment_idx].status = SegmentStatus::Failed;
            return Err(format!(
                "HTTP error for segment {}: {}",
                segment.segment_id,
                response.status()
            )
            .into());
        }

        // Download segment data
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            // Update segment progress
            {
                let mut task = task_arc.lock().await;
                task.segments[segment_idx].downloaded_bytes = downloaded;

                // Calculate overall progress
                let total_downloaded: u64 = task.segments.iter().map(|s| s.downloaded_bytes).sum();

                if task.filesize > 0 {
                    task.progress = (total_downloaded as f64 / task.filesize as f64) * 100.0;
                }

                // Call progress callback (clone task to avoid holding lock)
                let task_clone = task.clone();
                drop(task); // Release lock before calling callback
                progress_callback(&task_clone);
            }
        }

        // Mark segment as completed
        {
            let mut task = task_arc.lock().await;
            task.segments[segment_idx].status = SegmentStatus::Completed;
            task.segments[segment_idx].downloaded_bytes = segment.size();
        }

        Ok(())
    }

    /// Resume a paused or failed download
    async fn resume_download<F>(
        &self,
        task: &mut DownloadTask,
        progress_callback: F,
    ) -> Result<(), DownloadError>
    where
        F: Fn(&DownloadTask) + Send + Sync + 'static,
    {
        let temp_dir = self.get_temp_dir(task);

        // Get segments that need to be downloaded
        let resumable_segments = get_resumable_segments(task, &temp_dir);

        let progress_arc = Arc::new(progress_callback);

        if resumable_segments.is_empty() {
            // All segments completed, just merge
            let final_path = Path::new(&task.filepath);
            merge_segments(&task.segments, final_path, &temp_dir)
                .await
                .map_err(|e| format!("Failed to merge segments: {}", e))?;

            crate::download::merger::cleanup_segment_files(&task.segments, &temp_dir)?;

            task.status = DownloadStatus::Completed;
            task.completed_at = Some(chrono::Utc::now());
            progress_arc(task);

            return Ok(());
        }

        // Update task segments with resumable segments
        for resumable in &resumable_segments {
            if let Some(segment) = task
                .segments
                .iter_mut()
                .find(|s| s.segment_id == resumable.segment_id)
            {
                segment.start_byte = resumable.start_byte;
                segment.downloaded_bytes = resumable.downloaded_bytes;
                segment.status = SegmentStatus::Pending;
            }
        }

        // Download remaining segments
        self.download_all_segments(task, &temp_dir, progress_arc.clone())
            .await?;

        // Merge segments
        let final_path = Path::new(&task.filepath);
        merge_segments(&task.segments, final_path, &temp_dir)
            .await
            .map_err(|e| format!("Failed to merge segments: {}", e))?;

        crate::download::merger::cleanup_segment_files(&task.segments, &temp_dir)?;

        task.status = DownloadStatus::Completed;
        task.completed_at = Some(chrono::Utc::now());
        progress_arc(task);

        Ok(())
    }

    /// Get temp directory for a task's segment files
    fn get_temp_dir(&self, task: &DownloadTask) -> PathBuf {
        let temp_base = std::env::temp_dir().join("zoo-downloader");
        temp_base.join(task.id.to_string())
    }
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new()
    }
}

