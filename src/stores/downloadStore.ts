// Download state: one map of tasks by id, kept in sync with the backend by useDownloadSync.
import { atom } from 'jotai';
import { selectAtom } from 'jotai/utils';
import type { DownloadEvent, Task, TaskId } from '../backend';
import { focused, noSelection, type Selection } from '../utils/selection';

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
        // Unchanged numbers keep the same object, so nothing showing this task re-renders.
        if (!task || (task.downloaded === tick.downloaded && task.total === tick.total && task.speed === tick.speed)) continue;
        if (next === tasks) next = { ...tasks };
        next[tick.id] = { ...task, downloaded: tick.downloaded, total: tick.total, speed: tick.speed };
      }
      return next;
    }
    // For notifications; the `upserted` before it already carried the task.
    case 'finished':
      return tasks;
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

// Views of the store. Each one notifies its readers only when its own value changes: a
// progress tick changes one task object and nothing else, so it re-renders that task's row
// (and the footer's speed) instead of the whole page. Keep it that way: read the narrowest
// view a component needs, never the whole map.

const sameItems = <T>(a: readonly T[], b: readonly T[]) => a.length === b.length && a.every((x, i) => x === b[i]);

/** Task ids in list order, which is also the queue order. Changes on add, remove and move. */
export function orderedIds(tasks: TaskMap): TaskId[] {
  return Object.values(tasks)
    .sort((a, b) => a.order - b.order)
    .map((t) => t.id);
}
export const taskIdsAtom = selectAtom(tasksByIdAtom, orderedIds, sameItems);

/** One task (undefined once removed). A view per id; create it with useMemo. */
export const taskAtom = (id: TaskId) => selectAtom(tasksByIdAtom, (tasks) => tasks[id]);

export interface TaskCounts {
  total: number;
  running: number;
  paused: number;
  completed: number;
  /** Bytes per second over all running tasks. */
  speed: number;
}
export function countTasks(tasks: TaskMap): TaskCounts {
  const counts: TaskCounts = { total: 0, running: 0, paused: 0, completed: 0, speed: 0 };
  for (const t of Object.values(tasks)) {
    counts.total++;
    if (t.state === 'running') {
      counts.running++;
      counts.speed += t.speed;
    } else if (t.state === 'paused') counts.paused++;
    else if (t.state === 'completed') counts.completed++;
  }
  return counts;
}
const sameCounts = (a: TaskCounts, b: TaskCounts) =>
  a.total === b.total && a.running === b.running && a.paused === b.paused && a.completed === b.completed && a.speed === b.speed;
export const taskCountsAtom = selectAtom(tasksByIdAtom, countTasks, sameCounts);

/** Tasks that failed because their source changed (the user is asked what to do). */
export const sourceChangedTasksAtom = selectAtom(
  tasksByIdAtom,
  (tasks) => Object.values(tasks).filter((t) => t.state === 'failed' && t.error?.kind === 'sourceChanged'),
  sameItems,
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

/** The downloads selected in the table (see utils/selection). */
export const selectionAtom = atom<Selection>(noSelection);

/** The selected download the detail panel shows. */
export const selectedTaskIdAtom = atom((get) => focused(get(selectionAtom)));

/** That download's task. */
export const selectedTaskAtom = atom((get) => {
  const id = get(selectedTaskIdAtom);
  return id ? get(tasksByIdAtom)[id] ?? null : null;
});
