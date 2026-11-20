// A site module needs the user: an image captcha or a question (`zoo.ask`, e.g. an extraction
// code) is answered here; a reCAPTCHA is solved on the site's own page in a window of its own.
// One at a time, oldest first.
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
import { localized } from '../utils/localized';

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
  const { t, i18n } = useTranslation();
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
  const submit = () => run(backend.captcha.answer(current.id, text.trim()));
  const challenge = current.challenge;
  const typed = challenge.kind === 'image' || challenge.kind === 'ask';

  return (
    <Dialog open onClose={decline} maxWidth="xs" fullWidth>
      <DialogTitle>{t(challenge.kind === 'ask' ? 'captcha.askTitle' : 'captcha.title')}</DialogTitle>
      <DialogContent>
        {error && <Alert severity="error" sx={{ mb: 1 }}>{error}</Alert>}
        {challenge.kind === 'recaptcha' ? (
          <DialogContentText>{t('captcha.recaptchaMessage', { host: hostOf(challenge.pageUrl) })}</DialogContentText>
        ) : (
          <Box
            component="form"
            onSubmit={(e) => {
              e.preventDefault();
              if (text.trim()) submit();
            }}
          >
            {challenge.kind === 'image' ? (
              <Box
                component="img"
                src={`data:${challenge.mime};base64,${challenge.base64}`}
                alt={t('captcha.imageAlt')}
                sx={{ display: 'block', maxWidth: '100%', mx: 'auto', my: 1, imageRendering: 'pixelated' }}
              />
            ) : (
              <>
                {challenge.error && (
                  <Alert severity="warning" sx={{ mb: 1, whiteSpace: 'pre-wrap' }}>
                    {localized(challenge.error, i18n.language)}
                  </Alert>
                )}
                <DialogContentText sx={{ whiteSpace: 'pre-wrap' }}>{localized(challenge.message, i18n.language)}</DialogContentText>
              </>
            )}
            <TextField
              value={text}
              onChange={(e) => setText(e.target.value)}
              label={challenge.kind === 'image' ? t('captcha.enterText') : undefined}
              type={challenge.kind === 'ask' && challenge.secret ? 'password' : 'text'}
              fullWidth
              autoFocus
              margin="dense"
            />
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={decline}>{t('captcha.decline')}</Button>
        {typed ? (
          <Button variant="contained" disabled={!text.trim()} onClick={submit}>
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
