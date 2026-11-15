// Selection of a download in the table. Split in two so that only what shows the selected
// task re-renders when it changes: the table needs the id, the detail panel the task.
import { useCallback, useEffect } from 'react';
import { useAtom, useAtomValue } from 'jotai';
import type { Task } from '../backend';
import { selectedTaskIdAtom, selectedTaskAtom } from '../stores/downloadStore';

/** The selected id and ways to change it. Does not re-render on the task's progress. */
export function useSelectedDownload() {
  const [selectedTaskId, setSelectedTaskId] = useAtom(selectedTaskIdAtom);

  const selectTask = useCallback((taskId: string | null) => setSelectedTaskId(taskId), [setSelectedTaskId]);

  const clearSelection = useCallback(() => setSelectedTaskId(null), [setSelectedTaskId]);

  const toggleSelection = useCallback(
    (taskId: string) => setSelectedTaskId(selectedTaskId === taskId ? null : taskId),
    [selectedTaskId, setSelectedTaskId]
  );

  return {
    selectedTaskId,
    selectTask,
    clearSelection,
    toggleSelection,
    isSelected: (taskId: string) => selectedTaskId === taskId,
  };
}

/** The selected task (re-renders on each of its changes); clears the selection once it is removed. */
export function useSelectedTask(): Task | null {
  const [selectedTaskId, setSelectedTaskId] = useAtom(selectedTaskIdAtom);
  const selectedTask = useAtomValue(selectedTaskAtom);

  useEffect(() => {
    if (selectedTaskId && !selectedTask) setSelectedTaskId(null);
  }, [selectedTaskId, selectedTask, setSelectedTaskId]);

  return selectedTask;
}
