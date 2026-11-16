// Root of the add-downloads window (window label "add"): its own React tree and store, so it
// loads the settings itself (theme, language, categories, download folder) and re-reads them
// whenever the window gets focus, in case they changed in the main window.
import { useEffect, useMemo, useState } from 'react';
import { Alert, Box, CircularProgress, CssBaseline, ThemeProvider } from '@mui/material';
import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend } from '../backend';
import { settingsAtom } from '../stores/settingsStore';
import { darkTheme, lightTheme } from '../styles/theme';
import { waitForTauri } from '../utils/tauri';
import AddDownloads from './AddDownloads';
import '../i18n';

export default function AddWindowApp() {
  const { i18n } = useTranslation();
  const [settings, setSettings] = useAtom(settingsAtom);
  const [state, setState] = useState<'loading' | 'ready' | 'failed'>('loading');

  useEffect(() => {
    const load = () =>
      backend.settings.get().then((s) => {
        setSettings(s);
        if (s.language !== i18n.language) i18n.changeLanguage(s.language);
      });
    waitForTauri()
      .then((ok) => (ok ? load() : Promise.reject(new Error('backend unavailable'))))
      .then(() => setState('ready'))
      .catch(() => setState('failed'));
    const refresh = () => load().catch(() => {});
    window.addEventListener('focus', refresh);
    return () => window.removeEventListener('focus', refresh);
  }, [setSettings, i18n]);

  const theme = useMemo(() => (settings.theme === 'light' ? lightTheme : darkTheme), [settings.theme]);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ width: '100vw', height: '100vh', display: 'flex', flexDirection: 'column', bgcolor: 'background.default' }}>
        {state === 'loading' && <CircularProgress sx={{ m: 'auto' }} />}
        {state === 'failed' && (
          <Alert severity="error" sx={{ m: 2 }}>
            無法連線到 Zoo Downloader 的後端，請關閉這個視窗後再試一次。
          </Alert>
        )}
        {state === 'ready' && <AddDownloads />}
      </Box>
    </ThemeProvider>
  );
}
