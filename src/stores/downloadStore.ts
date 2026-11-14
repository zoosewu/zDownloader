// Download state: one map of tasks by id, kept in sync with the backend by useDownloadSync.
import { atom } from 'jotai';
import type { DownloadEvent, Task, TaskId } from '../backend';

export type TaskMap = Record<TaskId, Task>;

/** Keep whichever copy of a task is newer (higher `rev`). */
function newer(existing: Task | undefined, incoming: Task): Task {
  return existing && existing.rev > incoming.rev ? existing : incoming;
}

/** Apply one backend event. Returns the same object when nothing changed. */
export function applyEvent(tasks: TaskMap, event: DownloadEvent): TaskMap {
  switch (event.type) {
    case 'upserted': {
      const kept = newer(tasks[event.data.id], event.data);
      return kept === tasks[event.data.id] ? tasks : { ...tasks, [event.data.id]: kept };
    }
    case 'removed': {
      if (!(event.data in tasks)) return tasks;
      const { [event.data]: _removed, ...rest } = tasks;
      return rest;
    }
    case 'progress': {
      let next = tasks;
      for (const tick of event.data) {
        const task = next[tick.id];
        if (!task) continue;
        if (next === tasks) next = { ...tasks };
        next[tick.id] = { ...task, downloaded: tick.downloaded, total: tick.total, speed: tick.speed };
      }
      return next;
    }
  }
}

/**
 * Merge a full `downloads_list` snapshot. Events can arrive before or after the snapshot,
 * so per task the higher `rev` wins; a task missing from the snapshot is dropped unless it
 * is newer than everything in it (added after the snapshot was taken).
 */
export function mergeSnapshot(tasks: TaskMap, snapshot: Task[]): TaskMap {
  const snapshotRev = snapshot.reduce((max, t) => Math.max(max, t.rev), 0);
  const next: TaskMap = {};
  for (const task of snapshot) next[task.id] = newer(tasks[task.id], task);
  for (const task of Object.values(tasks)) {
    if (!(task.id in next) && task.rev > snapshotRev) next[task.id] = task;
  }
  return next;
}

export const tasksByIdAtom = atom<TaskMap>({});

/** All tasks in list order, which is also the queue order. */
export const downloadTasksAtom = atom((get) => Object.values(get(tasksByIdAtom)).sort((a, b) => a.order - b.order));

export const activeDownloadsAtom = atom((get) =>
  get(downloadTasksAtom).filter((t) => t.state === 'running')
);

export const pausedDownloadsAtom = atom((get) =>
  get(downloadTasksAtom).filter((t) => t.state === 'paused')
);

export const completedDownloadsAtom = atom((get) =>
  get(downloadTasksAtom).filter((t) => t.state === 'completed')
);

export const applyEventAtom = atom(null, (get, set, event: DownloadEvent) => {
  set(tasksByIdAtom, applyEvent(get(tasksByIdAtom), event));
});

export const mergeSnapshotAtom = atom(null, (get, set, snapshot: Task[]) => {
  set(tasksByIdAtom, mergeSnapshot(get(tasksByIdAtom), snapshot));
});

/** Apply a command's returned task immediately, without waiting for its event. */
export const upsertTaskAtom = atom(null, (get, set, task: Task) => {
  set(tasksByIdAtom, applyEvent(get(tasksByIdAtom), { type: 'upserted', data: task }));
});

// Selected task ID atom - tracks which download item is currently selected in the table
export const selectedTaskIdAtom = atom<TaskId | null>(null);

// Selected task atom (derived) - the full task object of the selected item
export const selectedTaskAtom = atom((get) => {
  const id = get(selectedTaskIdAtom);
  return id ? get(tasksByIdAtom)[id] ?? null : null;
});
