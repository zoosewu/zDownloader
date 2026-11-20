// Settings: the app's version, "check for updates now", and its result.
import React, { useEffect, useState } from 'react';
import { Box, Button, CircularProgress, Typography } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { getVersion } from '@tauri-apps/api/app';
import { useAppUpdate } from '../hooks/useAppUpdate';

const AppUpdateSection: React.FC = () => {
  const { t } = useTranslation();
  const { state, checkNow, install, dismiss } = useAppUpdate();
  const [version, setVersion] = useState('');

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
  }, []);

  const result =
    state.kind === 'upToDate'
      ? t('update.upToDate')
      : state.kind === 'available'
        ? t('update.available', { version: state.update.version })
        : state.kind === 'failed'
          ? // The updater's wording when the release has no latest.json yet (nothing published).
            /valid release JSON/i.test(state.message)
            ? t('update.noRelease')
            : t('update.failed', { message: state.message })
          : null;

  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
      <Typography variant="body2">{t('update.current', { version })}</Typography>
      <Button
        variant="outlined"
        onClick={() => {
          dismiss(null); // a manual check shows the update even if it was put off
          checkNow();
        }}
        disabled={state.kind === 'checking' || state.kind === 'downloading' || state.kind === 'installing'}
        // The spinner's place is kept, so the button keeps its width while checking.
        startIcon={<Box sx={{ width: 12, height: 12, display: 'flex' }}>{state.kind === 'checking' && <CircularProgress size={12} />}</Box>}
      >
        {t('update.checkNow')}
      </Button>
      {state.kind === 'available' && (
        <Button variant="contained" onClick={() => install(state.update)}>
          {t('update.install')}
        </Button>
      )}
      {result && (
        <Typography variant="body2" color={state.kind === 'failed' ? 'error' : 'text.secondary'}>
          {result}
        </Typography>
      )}
    </Box>
  );
};

export default AppUpdateSection;
