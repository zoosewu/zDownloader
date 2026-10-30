// Download queue management - simple in-memory queue for MVP

use crate::models::DownloadTask;
use uuid::Uuid;

pub struct DownloadQueue {
    tasks: Vec<DownloadTask>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
        }
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
