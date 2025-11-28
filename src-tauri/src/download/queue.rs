// Download queue management - simple in-memory queue for MVP
use crate::models::DownloadTask;
use uuid::Uuid;

pub struct DownloadQueue {
    tasks: Vec<DownloadTask>,
    max_concurrent: usize, // Maximum concurrent downloads (default: 3)
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent: 3, // Default: 3 concurrent downloads
        }
    }

    pub fn with_max_concurrent(max_concurrent: usize) -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent,
        }
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    pub fn get_max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn get_active_count(&self) -> usize {
        use crate::models::download_task::DownloadStatus;
        self.tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.status,
                    DownloadStatus::Downloading | DownloadStatus::Initializing
                )
            })
            .count()
    }

    pub fn can_start_download(&self) -> bool {
        self.get_active_count() < self.max_concurrent
    }

    pub fn get_pending_tasks(&self) -> Vec<DownloadTask> {
        use crate::models::download_task::DownloadStatus;
        self.tasks
            .iter()
            .filter(|t| t.status == DownloadStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn add_task(&mut self, task: DownloadTask) {
        self.tasks.push(task);
    }

    pub fn get_task(&self, id: &Uuid) -> Option<DownloadTask> {
        self.tasks.iter().find(|t| &t.id == id).cloned()
    }

    pub fn update_task(&mut self, task: DownloadTask) {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task.id) {
            self.tasks[pos] = task;
        }
    }

    pub fn get_all_tasks(&self) -> Vec<DownloadTask> {
        self.tasks.clone()
    }

    pub fn remove_task(&mut self, id: &Uuid) -> bool {
        if let Some(pos) = self.tasks.iter().position(|t| &t.id == id) {
            self.tasks.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self::new()
    }
}
