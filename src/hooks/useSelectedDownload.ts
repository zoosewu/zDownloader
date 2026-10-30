// useSelectedDownload hook - Manage selected download item state
import { useCallback, useEffect } from 'react';
import { useAtom, useAtomValue } from 'jotai';
import { selectedTaskIdAtom, selectedTaskAtom } from '../stores/downloadStore';

export function useSelectedDownload() {
  const [selectedTaskId, setSelectedTaskId] = useAtom(selectedTaskIdAtom);
  const selectedTask = useAtomValue(selectedTaskAtom);

  // Clear selection if the selected task was removed
  useEffect(() => {
    if (selectedTaskId && !selectedTask) {
      setSelectedTaskId(null);
    }
  }, [selectedTaskId, selectedTask, setSelectedTaskId]);

  const selectTask = useCallback((taskId: string | null) => setSelectedTaskId(taskId), [setSelectedTaskId]);

  const clearSelection = useCallback(() => setSelectedTaskId(null), [setSelectedTaskId]);

  const toggleSelection = useCallback(
    (taskId: string) => setSelectedTaskId(selectedTaskId === taskId ? null : taskId),
    [selectedTaskId, setSelectedTaskId]
  );

  return {
    selectedTaskId,
    selectedTask,
    selectTask,
    clearSelection,
    toggleSelection,
    isSelected: (taskId: string) => selectedTaskId === taskId,
  };
}
