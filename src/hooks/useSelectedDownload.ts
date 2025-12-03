// useSelectedDownload hook - Manage selected download item state
import { useCallback, useEffect } from 'react';
import { useAtom, useAtomValue } from 'jotai';
import {
  selectedTaskIdAtom,
  selectedTaskAtom,
  downloadTasksAtom,
} from '../stores/downloadStore';

/**
 * Hook to manage selected download item
 * @returns Selected task state and control functions
 */
export function useSelectedDownload() {
  const [selectedTaskId, setSelectedTaskId] = useAtom(selectedTaskIdAtom);
  const selectedTask = useAtomValue(selectedTaskAtom);
  const tasks = useAtomValue(downloadTasksAtom);

  // Clear selection if selected task is deleted
  useEffect(() => {
    if (selectedTaskId && !tasks.find((t) => t.id === selectedTaskId)) {
      console.log('[useSelectedDownload] Selected task was deleted, clearing selection');
      setSelectedTaskId(null);
    }
  }, [tasks, selectedTaskId, setSelectedTaskId]);

  // Select a task by ID
  const selectTask = useCallback(
    (taskId: string | null) => {
      setSelectedTaskId(taskId);
    },
    [setSelectedTaskId]
  );

  // Clear selection
  const clearSelection = useCallback(() => {
    setSelectedTaskId(null);
  }, [setSelectedTaskId]);

  // Toggle selection (select if not selected, clear if already selected)
  const toggleSelection = useCallback(
    (taskId: string) => {
      if (selectedTaskId === taskId) {
        setSelectedTaskId(null);
      } else {
        setSelectedTaskId(taskId);
      }
    },
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
