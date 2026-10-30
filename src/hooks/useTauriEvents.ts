// Non-download backend events (CAPTCHA prompts). Mounted once at the app root so they are
// handled on every page; download events are handled by useDownloadSync.
import { useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useSetAtom } from 'jotai';
import { currentCaptchaAtom, showCaptchaDialogAtom } from '../stores/captchaStore';
import { CaptchaRequiredEvent, CaptchaSolvedEvent } from '../types/captcha';

export const useTauriEvents = (ready: boolean) => {
  const setCurrentCaptcha = useSetAtom(currentCaptchaAtom);
  const setShowCaptchaDialog = useSetAtom(showCaptchaDialogAtom);

  useEffect(() => {
    if (!ready) return;
    let disposed = false;
    const unlisten: UnlistenFn[] = [];

    (async () => {
      try {
        const subscriptions = await Promise.all([
          listen<CaptchaRequiredEvent>('captcha-required', (event) => {
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
          }),
          listen<CaptchaSolvedEvent>('captcha-solved', () => {
            setCurrentCaptcha(null);
            setShowCaptchaDialog(false);
          }),
        ]);
        // Unmounted while subscribing: release now instead of leaking the listeners.
        if (disposed) subscriptions.forEach((u) => u());
        else unlisten.push(...subscriptions);
      } catch (error) {
        console.warn('[events] CAPTCHA events unavailable:', error);
      }
    })();

    return () => {
      disposed = true;
      unlisten.forEach((u) => u());
    };
  }, [ready, setCurrentCaptcha, setShowCaptchaDialog]);
};

export default useTauriEvents;
