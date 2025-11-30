// Custom hook for download control logic - calling Tauri commands
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSetAtom } from 'jotai';
import { removeTaskAtom, setTasksAtom } from '../stores/downloadStore';
import { DownloadTask } from '../types/download';

export const useDownloads = () => {
  const removeTask = useSetAtom(removeTaskAtom);
  const setTasks = useSetAtom(setTasksAtom);

  // Create a new download
  const createDownload = useCallback(
    async (url: string, filename?: string): Promise<string | null> => {
      try {
        const taskId = await invoke<string>('create_download', {
          url,
          filename: filename || null,
        });

        // The task will be added to the store via Tauri events
        // when the backend emits progress-updated or status-changed

        return taskId;
      } catch (error) {
        console.error('Failed to create download:', error);
        return null;
      }
    },
    []
  );

  // Start a download
  const startDownload = useCallback(async (taskId: string): Promise<boolean> => {
    try {
      await invoke('start_download', { taskId });
      return true;
    } catch (error) {
      console.error('Failed to start download:', error);
      return false;
    }
  }, []);

  // Pause a download
  const pauseDownload = useCallback(async (taskId: string): Promise<boolean> => {
    try {
      await invoke('pause_download', { taskId });
      return true;
    } catch (error) {
      console.error('Failed to pause download:', error);
      return false;
    }
  }, []);

  // Cancel a download
  const cancelDownload = useCallback(
    async (taskId: string): Promise<boolean> => {
      try {
        await invoke('cancel_download', { taskId });
        // Remove from local store
        removeTask(taskId);
        return true;
      } catch (error) {
        console.error('Failed to cancel download:', error);
        return false;
      }
    },
    [removeTask]
  );

  // Get download list from backend
  const refreshDownloadList = useCallback(async (): Promise<DownloadTask[]> => {
    try {
      const tasks = await invoke<DownloadTask[]>('get_download_list');
      // Update the store with the fetched tasks
      setTasks(tasks);
      return tasks;
    } catch (error) {
      console.error('Failed to get download list:', error);
      return [];
    }
  }, [setTasks]);

  // Create and start download in one step
  const createAndStartDownload = useCallback(
    async (url: string, filename?: string): Promise<boolean> => {
      const taskId = await createDownload(url, filename);
      if (taskId) {
        const started = await startDownload(taskId);
        if (started) {
          // Refresh the list to show the new download
          await refreshDownloadList();
        }
        return started;
      }
      return false;
    },
    [createDownload, startDownload, refreshDownloadList]
  );

  return {
    createDownload,
    startDownload,
    pauseDownload,
    cancelDownload,
    refreshDownloadList,
    createAndStartDownload,
  };
};

export default useDownloads;
