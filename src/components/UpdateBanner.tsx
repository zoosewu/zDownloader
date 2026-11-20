// A card floating over the bottom-left corner when a new version is out: what changed,
// "Update and restart", "Later". Shows progress while downloading and the reason if it fails.
// It floats so the page never moves when it appears or goes.
import React, { useState } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, LinearProgress, Paper, Typography } from '@mui/material';
import { SystemUpdateAlt as UpdateIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { useAppUpdate } from '../hooks/useAppUpdate';
import { formatBytes } from '../utils/downloadCalculations';

const UpdateBanner: React.FC = () => {
  const { t } = useTranslation();
  const { state, install, dismissed, dismiss } = useAppUpdate();
  const [showNotes, setShowNotes] = useState(false);

  const update = 'update' in state ? state.update : undefined;
  if (!update || (state.kind === 'available' && dismissed === update.version)) return null;
  const busy = state.kind === 'downloading' || state.kind === 'installing';
  const tone = state.kind === 'failed' ? 'error' : 'info';

  return (
    <Paper
      elevation={6}
      role="status"
      sx={{
        position: 'fixed',
        left: 16,
        // Above the footer.
        bottom: 36,
        zIndex: (theme) => theme.zIndex.snackbar,
        maxWidth: 'min(560px, calc(100vw - 32px))',
        display: 'flex',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: 1,
        px: 1.5,
        py: 0.75,
        bgcolor: `${tone}.main`,
        color: (theme) => theme.palette.getContrastText(theme.palette[tone].main),
      }}
    >
      <UpdateIcon fontSize="small" />
      <Typography variant="body2" sx={{ flex: 1, minWidth: 0 }}>
        {state.kind === 'failed'
          ? t('update.failed', { message: state.message })
          : state.kind === 'downloading'
            ? t('update.downloading', {
                version: update.version,
                progress: state.total ? `${Math.floor((state.done / state.total) * 100)}%` : formatBytes(state.done),
              })
            : state.kind === 'installing'
              ? t('update.installing')
              : t('update.available', { version: update.version })}
      </Typography>
      {state.kind === 'downloading' && state.total && (
        <LinearProgress variant="determinate" value={(state.done / state.total) * 100} color="inherit" sx={{ width: 120 }} />
      )}
      {update.body && (
        <Button color="inherit" size="small" onClick={() => setShowNotes(true)}>
          {t('update.notes')}
        </Button>
      )}
      <Button color="inherit" size="small" variant="outlined" disabled={busy} onClick={() => install(update)}>
        {state.kind === 'failed' ? t('update.retry') : t('update.install')}
      </Button>
      {!busy && (
        <Button color="inherit" size="small" onClick={() => dismiss(update.version)}>
          {t('update.later')}
        </Button>
      )}

      <Dialog open={showNotes} onClose={() => setShowNotes(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{t('update.notesTitle', { version: update.version })}</DialogTitle>
        <DialogContent>
          <Typography variant="body2" component="pre" sx={{ whiteSpace: 'pre-wrap', fontFamily: 'inherit', m: 0 }}>
            {update.body}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowNotes(false)}>{t('common.close')}</Button>
        </DialogActions>
      </Dialog>
    </Paper>
  );
};

export default UpdateBanner;
