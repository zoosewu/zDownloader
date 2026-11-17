// What the download list shows: a status (sidebar "Status") or a user category ("Categories").
// Views are narrow like the rest of the store: a progress tick changes no task's view, so the
// sidebar and the filtered list do not re-render on progress.
import { atom } from 'jotai';
import { selectAtom } from 'jotai/utils';
import type { Task, TaskId } from '../backend';
import { tasksByIdAtom, orderedIds, type TaskMap } from './downloadStore';
import { settingsAtom } from './settingsStore';

export const STATUS_VIEWS = ['all', 'running', 'waiting', 'paused', 'completed', 'failed', 'cancelled'] as const;
export type StatusView = (typeof STATUS_VIEWS)[number];
/** A status, or `cat:<category id>`. */
export type View = StatusView | `cat:${string}`;

export const categoryView = (id: string): View => `cat:${id}`;

/** The status group a task is listed under. A failed task that will retry by itself is waiting. */
export function statusOf(task: Task): Exclude<StatusView, 'all'> {
  switch (task.state) {
    case 'running':
      return 'running';
    case 'queued':
    case 'scheduled':
      return 'waiting';
    case 'failed':
      return task.nextRetryAt ? 'waiting' : 'failed';
    default:
      return task.state;
  }
}

export function inView(task: Task, view: View): boolean {
  if (view === 'all') return true;
  if (view.startsWith('cat:')) return task.tags.includes(view.slice(4));
  return statusOf(task) === view;
}

/** Reordering means queue order, so it is offered only where the whole queue is visible. */
export const canReorder = (view: View) => view === 'all' || view === 'waiting';

const STORAGE_KEY = 'zoo.view';
function loadView(): View {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && (STATUS_VIEWS.includes(saved as StatusView) || saved.startsWith('cat:'))) return saved as View;
  } catch {
    // Storage unavailable: start at "all".
  }
  return 'all';
}

const chosenViewAtom = atom<View>(loadView());

/** The selected view; a category that no longer exists falls back to "all". */
export const viewAtom = atom(
  (get) => {
    const view = get(chosenViewAtom);
    if (!view.startsWith('cat:')) return view;
    return get(settingsAtom).categories.some((c) => categoryView(c.id) === view) ? view : 'all';
  },
  (_get, set, view: View) => {
    set(chosenViewAtom, view);
    try {
      localStorage.setItem(STORAGE_KEY, view);
    } catch {
      // Not remembered; fine.
    }
  },
);

const sameItems = <T>(a: readonly T[], b: readonly T[]) => a.length === b.length && a.every((x, i) => x === b[i]);

// One cached selector per view, so switching back and forth reuses them.
const idsByView = new Map<View, ReturnType<typeof idsIn>>();
function idsIn(view: View) {
  return selectAtom(
    tasksByIdAtom,
    // Newest first: the queue starts from the oldest, which the table shows at the bottom.
    (tasks: TaskMap) => orderedIds(tasks).filter((id) => inView(tasks[id], view)).reverse(),
    sameItems,
  );
}

/** Ids of the tasks in the selected view, as the table shows them: reverse queue order, so a
 * new download appears at the top while downloads still start oldest first. */
export const visibleTaskIdsAtom = atom((get): TaskId[] => {
  const view = get(viewAtom);
  let ids = idsByView.get(view);
  if (!ids) idsByView.set(view, (ids = idsIn(view)));
  return get(ids);
});

export type ViewCounts = Record<string, number>;

export function countViews(tasks: TaskMap): ViewCounts {
  const counts: ViewCounts = Object.fromEntries(STATUS_VIEWS.map((v) => [v, 0]));
  for (const task of Object.values(tasks)) {
    counts.all++;
    counts[statusOf(task)]++;
    for (const tag of task.tags) counts[categoryView(tag)] = (counts[categoryView(tag)] ?? 0) + 1;
  }
  return counts;
}
const sameCounts = (a: ViewCounts, b: ViewCounts) => {
  const keys = Object.keys(a);
  return keys.length === Object.keys(b).length && keys.every((k) => a[k] === b[k]);
};
/** Tasks per view, for the sidebar. */
export const viewCountsAtom = selectAtom(tasksByIdAtom, countViews, sameCounts);
