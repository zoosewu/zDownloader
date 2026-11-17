// Selection of downloads in the table. Split in two so that only what shows the selected
// task re-renders when it changes: the table needs the ids, the detail panel the task.
import { useAtom, useAtomValue } from 'jotai';
import type { Task } from '../backend';
import { selectedTaskAtom, selectedTaskIdAtom, selectionAtom } from '../stores/downloadStore';

/** The selection and the download the detail panel shows. Does not re-render on progress. */
export function useSelectedDownload() {
  const [selection, setSelection] = useAtom(selectionAtom);
  return { selection, setSelection, selectedTaskId: useAtomValue(selectedTaskIdAtom) };
}

/** The task the detail panel shows (re-renders on each of its changes). */
export function useSelectedTask(): Task | null {
  return useAtomValue(selectedTaskAtom);
}
