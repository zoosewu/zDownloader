// Which downloads are selected in the table, and how clicks change that (as in a file manager).
import type { TaskId } from '../backend';

export interface Selection {
  /** In the order they were selected. */
  ids: TaskId[];
  /** The last clicked: where a Shift range starts. */
  anchor: TaskId | null;
}

export const noSelection: Selection = { ids: [], anchor: null };

/** Plain click, Ctrl (or ⌘) click, Shift click. */
export type ClickMode = 'only' | 'toggle' | 'range';

export const clickMode = (e: { ctrlKey: boolean; metaKey: boolean; shiftKey: boolean }): ClickMode =>
  e.shiftKey ? 'range' : e.ctrlKey || e.metaKey ? 'toggle' : 'only';

/** The selection after clicking `id`; `shown` is the table's order. */
export function click(selection: Selection, id: TaskId, mode: ClickMode, shown: TaskId[]): Selection {
  const { ids, anchor } = selection;
  switch (mode) {
    case 'toggle':
      return { ids: ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id], anchor: id };
    case 'range': {
      const from = shown.indexOf(anchor ?? id);
      const to = shown.indexOf(id);
      if (from < 0 || to < 0) return { ids: [id], anchor: id };
      return { ids: shown.slice(Math.min(from, to), Math.max(from, to) + 1), anchor };
    }
    case 'only':
      // Clicking the one selected download again deselects it.
      return ids.length === 1 && ids[0] === id ? noSelection : { ids: [id], anchor: id };
  }
}

/** Right click: on a selected download the menu is for the whole selection, otherwise only for it. */
export const forMenu = (selection: Selection, id: TaskId): Selection =>
  selection.ids.includes(id) ? selection : { ids: [id], anchor: id };

/** Only the downloads still shown (others finished, moved to another view or were removed). */
export function within(selection: Selection, shown: TaskId[]): Selection {
  const ids = selection.ids.filter((id) => shown.includes(id));
  if (ids.length === selection.ids.length) return selection;
  return { ids, anchor: selection.anchor !== null && ids.includes(selection.anchor) ? selection.anchor : (ids[ids.length - 1] ?? null) };
}

/** The download the detail panel shows: the last clicked one if still selected. */
export const focused = ({ ids, anchor }: Selection): TaskId | null =>
  anchor !== null && ids.includes(anchor) ? anchor : (ids[ids.length - 1] ?? null);
