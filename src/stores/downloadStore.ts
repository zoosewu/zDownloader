// Jotai atoms for download state management
import { atom } from 'jotai';
import { DownloadTask } from '../types/download';

// Download tasks atom - all download tasks
export const downloadTasksAtom = atom<DownloadTask[]>([]);

// Active downloads atom (derived) - tasks currently downloading or initializing
export const activeDownloadsAtom = atom((get) => {
  const tasks = get(downloadTasksAtom);
  return tasks.filter(
    (task) => task.status === 'Downloading' || task.status === 'Initializing'
  );
});

// Completed downloads atom (derived) - successfully completed tasks
export const completedDownloadsAtom = atom((get) => {
  const tasks = get(downloadTasksAtom);
  return tasks.filter((task) => task.status === 'Completed');
});

// Failed downloads atom (derived) - failed tasks
export const failedDownloadsAtom = atom((get) => {
  const tasks = get(downloadTasksAtom);
  return tasks.filter((task) => task.status === 'Failed');
});

// Paused downloads atom (derived) - paused tasks
export const pausedDownloadsAtom = atom((get) => {
  const tasks = get(downloadTasksAtom);
  return tasks.filter((task) => task.status === 'Paused');
});

// Pending downloads atom (derived) - pending tasks
export const pendingDownloadsAtom = atom((get) => {
  const tasks = get(downloadTasksAtom);
  return tasks.filter((task) => task.status === 'Pending');
});

// Active download count atom (derived)
export const activeDownloadCountAtom = atom((get) => {
  return get(activeDownloadsAtom).length;
});

// Total download progress atom (derived) - average progress of active downloads
export const totalProgressAtom = atom((get) => {
  const activeTasks = get(activeDownloadsAtom);
  if (activeTasks.length === 0) return 0;
  const totalProgress = activeTasks.reduce((sum, task) => sum + task.progress, 0);
  return totalProgress / activeTasks.length;
});

// Clipboard URL atom - detected URL from clipboard
export const clipboardUrlAtom = atom<string | null>(null);

// Clipboard hint visibility atom
export const showClipboardHintAtom = atom<boolean>(false);

// Unfinished downloads atom - detected on app startup
export const unfinishedDownloadsAtom = atom<DownloadTask[]>([]);

// Resume dialog visibility atom
export const showResumeDialogAtom = atom<boolean>(false);

// Helper atom to add a task
export const addTaskAtom = atom(
  null,
  (get, set, task: DownloadTask) => {
    const tasks = get(downloadTasksAtom);
    set(downloadTasksAtom, [...tasks, task]);
  }
);

// Helper atom to update a task
export const updateTaskAtom = atom(
  null,
  (get, set, updatedTask: DownloadTask) => {
    console.log('updateTaskAtom called with:', updatedTask.id, 'Progress:', updatedTask.progress);
    const tasks = get(downloadTasksAtom);
    const index = tasks.findIndex((t) => t.id === updatedTask.id);
    console.log('Task index in array:', index, 'Total tasks:', tasks.length);
    if (index !== -1) {
      const newTasks = [...tasks];
      newTasks[index] = updatedTask;
      set(downloadTasksAtom, newTasks);
      console.log('Task updated in store');
    } else {
      console.warn('Task not found in store:', updatedTask.id);
    }
  }
);

// Helper atom to remove a task
export const removeTaskAtom = atom(
  null,
  (get, set, taskId: string) => {
    const tasks = get(downloadTasksAtom);
    set(downloadTasksAtom, tasks.filter((t) => t.id !== taskId));
  }
);

// Helper atom to clear completed tasks
export const clearCompletedAtom = atom(
  null,
  (get, set) => {
    const tasks = get(downloadTasksAtom);
    set(downloadTasksAtom, tasks.filter((t) => t.status !== 'Completed'));
  }
);

// Helper atom to set all tasks (for refreshing from backend)
export const setTasksAtom = atom(
  null,
  (_get, set, tasks: DownloadTask[]) => {
    set(downloadTasksAtom, tasks);
  }
);

// Selected task ID atom - tracks which download item is currently selected in the table
export const selectedTaskIdAtom = atom<string | null>(null);

// Selected task atom (derived) - the full task object of the selected item
export const selectedTaskAtom = atom((get) => {
  const selectedId = get(selectedTaskIdAtom);
  const tasks = get(downloadTasksAtom);
  return tasks.find((t) => t.id === selectedId) || null;
});
