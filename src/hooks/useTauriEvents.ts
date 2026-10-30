// Custom hook for listening to Tauri events
import { useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useSetAtom } from 'jotai';
import { updateTaskAtom } from '../stores/downloadStore';
import {
  currentCaptchaAtom,
  showCaptchaDialogAtom,
} from '../stores/captchaStore';
import { DownloadTask } from '../types/download';
import { CaptchaRequiredEvent, CaptchaSolvedEvent } from '../types/captcha';

interface ProgressUpdatePayload {
  task: DownloadTask;
}

interface StatusChangedPayload {
  task: DownloadTask;
}

export const useTauriEvents = () => {
  const updateTask = useSetAtom(updateTaskAtom);
  const setCurrentCaptcha = useSetAtom(currentCaptchaAtom);
  const setShowCaptchaDialog = useSetAtom(showCaptchaDialogAtom);

  useEffect(() => {
    let unlistenProgress: UnlistenFn | null = null;
    let unlistenStatus: UnlistenFn | null = null;
    let unlistenCaptchaRequired: UnlistenFn | null = null;
    let unlistenCaptchaSolved: UnlistenFn | null = null;

    // Listen for progress-updated events
    const setupProgressListener = async () => {
      unlistenProgress = await listen<ProgressUpdatePayload>(
        'progress-updated',
        (event) => {
          console.log('Progress event received:', event.payload);
          if (event.payload && event.payload.task) {
            console.log('Updating task:', event.payload.task.id, 'Progress:', event.payload.task.progress);
            updateTask(event.payload.task);
          } else {
            console.warn('Invalid progress event payload:', event.payload);
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

    // Listen for captcha-required events
    const setupCaptchaRequiredListener = async () => {
      unlistenCaptchaRequired = await listen<CaptchaRequiredEvent>(
        'captcha-required',
        (event) => {
          if (event.payload) {
            // Convert event payload to CaptchaChallenge format
            setCurrentCaptcha({
              id: event.payload.challenge_id,
              captcha_type: event.payload.captcha_type,
              challenge_data: event.payload.challenge_data,
              params: event.payload.params,
              task_id: event.payload.task_id,
              module_id: event.payload.module_id,
              created_at: Date.now() / 1000,
            });
            setShowCaptchaDialog(true);
          }
        }
      );
    };

    // Listen for captcha-solved events
    const setupCaptchaSolvedListener = async () => {
      unlistenCaptchaSolved = await listen<CaptchaSolvedEvent>(
        'captcha-solved',
        (event) => {
          if (event.payload) {
            // Clear CAPTCHA dialog when solved
            setCurrentCaptcha(null);
            setShowCaptchaDialog(false);
          }
        }
      );
    };

    // Setup all listeners
    setupProgressListener();
    setupStatusListener();
    setupCaptchaRequiredListener();
    setupCaptchaSolvedListener();

    // Cleanup on unmount
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenStatus) unlistenStatus();
      if (unlistenCaptchaRequired) unlistenCaptchaRequired();
      if (unlistenCaptchaSolved) unlistenCaptchaSolved();
    };
  }, [
    updateTask,
    setCurrentCaptcha,
    setShowCaptchaDialog,
  ]);
};

export default useTauriEvents;
