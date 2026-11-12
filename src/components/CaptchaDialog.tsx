// A site module needs a captcha solved. Image captchas are answered here; a reCAPTCHA is
// solved on the site's own page in a window of its own. One captcha at a time, oldest first.
import { useEffect, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  TextField,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import { backend, type CaptchaRequest } from '../backend';

/** Captchas waiting for an answer, kept in step with the backend. */
function usePendingCaptchas(): CaptchaRequest[] {
  const [pending, setPending] = useState<CaptchaRequest[]>([]);
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    backend.events
      .onCaptchaEvent((event) =>
        setPending((list) =>
          event.type === 'opened' ? [...list.filter((c) => c.id !== event.data.id), event.data] : list.filter((c) => c.id !== event.data)
        )
      )
      .then((stop) => (disposed ? stop() : (unlisten = stop)));
    // Captchas that opened before this mounted.
    backend.captcha
      .pending()
      .then((list) => !disposed && setPending((current) => [...list.filter((c) => !current.some((x) => x.id === c.id)), ...current]))
      .catch((error) => console.warn('[captcha] cannot list pending captchas:', error.message));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  return pending;
}

export default function CaptchaDialog() {
  const { t } = useTranslation();
  const pending = usePendingCaptchas();
  const current = pending[0] ?? null;
  const [text, setText] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setText('');
    setError(null);
  }, [current?.id]);

  if (!current) return null;

  const run = (action: Promise<unknown>) => action.catch((e) => setError((e as Error).message));
  const decline = () => run(backend.captcha.answer(current.id, null));
  const challenge = current.challenge;

  return (
    <Dialog open onClose={decline} maxWidth="xs" fullWidth>
      <DialogTitle>{t('captcha.title')}</DialogTitle>
      <DialogContent>
        {error && <Alert severity="error" sx={{ mb: 1 }}>{error}</Alert>}
        {challenge.kind === 'image' ? (
          <Box
            component="form"
            onSubmit={(e) => {
              e.preventDefault();
              run(backend.captcha.answer(current.id, text.trim()));
            }}
          >
            <Box
              component="img"
              src={`data:${challenge.mime};base64,${challenge.base64}`}
              alt={t('captcha.imageAlt')}
              sx={{ display: 'block', maxWidth: '100%', mx: 'auto', my: 1, imageRendering: 'pixelated' }}
            />
            <TextField
              value={text}
              onChange={(e) => setText(e.target.value)}
              label={t('captcha.enterText')}
              fullWidth
              autoFocus
              margin="dense"
            />
          </Box>
        ) : (
          <DialogContentText>{t('captcha.recaptchaMessage', { host: hostOf(challenge.pageUrl) })}</DialogContentText>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={decline}>{t('captcha.decline')}</Button>
        {challenge.kind === 'image' ? (
          <Button variant="contained" disabled={!text.trim()} onClick={() => run(backend.captcha.answer(current.id, text.trim()))}>
            {t('captcha.submit')}
          </Button>
        ) : (
          <Button variant="contained" autoFocus onClick={() => run(backend.captcha.openWindow(current.id))}>
            {t('captcha.openPage')}
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
}

function hostOf(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}
