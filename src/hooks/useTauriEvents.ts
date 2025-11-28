// Custom hook for listening to Tauri events
import { useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useSetAtom } from 'jotai';
import {
  updateTaskAtom,
  clipboardUrlAtom,
  showClipboardHintAtom,
} from '../stores/downloadStore';
import { DownloadTask } from '../types/download';

interface ProgressUpdatePayload {
  task: DownloadTask;
}

interface StatusChangedPayload {
  task: DownloadTask;
}

interface ClipboardDetectedPayload {
  url: string;
}

export const useTauriEvents = () => {
  const updateTask = useSetAtom(updateTaskAtom);
  const setClipboardUrl = useSetAtom(clipboardUrlAtom);
  const setShowClipboardHint = useSetAtom(showClipboardHintAtom);

  useEffect(() => {
    let unlistenProgress: UnlistenFn | null = null;
    let unlistenStatus: UnlistenFn | null = null;
    let unlistenClipboard: UnlistenFn | null = null;

    // Listen for progress-updated events
    const setupProgressListener = async () => {
      unlistenProgress = await listen<ProgressUpdatePayload>(
        'progress-updated',
        (event) => {
          if (event.payload && event.payload.task) {
            updateTask(event.payload.task);
          }
        }
      );
    };

    // Listen for status-changed events
    const setupStatusListener = async () => {
      unlistenStatus = await listen<StatusChangedPayload>(
        'status-changed',
        (event) => {
          if (event.payload && event.payload.task) {
            updateTask(event.payload.task);
          }
        }
      );
    };

    // Listen for clipboard-detected events
    const setupClipboardListener = async () => {
      unlistenClipboard = await listen<ClipboardDetectedPayload>(
        'clipboard-detected',
        (event) => {
          if (event.payload && event.payload.url) {
            setClipboardUrl(event.payload.url);
            setShowClipboardHint(true);
          }
        }
      );
    };

    // Setup all listeners
    setupProgressListener();
    setupStatusListener();
    setupClipboardListener();

    // Cleanup on unmount
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenStatus) unlistenStatus();
      if (unlistenClipboard) unlistenClipboard();
    };
  }, [updateTask, setClipboardUrl, setShowClipboardHint]);
};

export default useTauriEvents;
