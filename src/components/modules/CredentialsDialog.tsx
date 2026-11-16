// Account details a module logs in with. Stored values are never shown: leave a field empty
// to keep it, or clear it.
import { useEffect, useState } from 'react';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogTitle, InputAdornment, TextField } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { backend, type InstalledInfo } from '../../backend';

interface CredentialsDialogProps {
  module: InstalledInfo | null;
  onClose: () => void;
}

export default function CredentialsDialog({ module, onClose }: CredentialsDialogProps) {
  const { t } = useTranslation();
  /** Typed values; `null` clears a stored one. Untouched fields are absent. */
  const [values, setValues] = useState<Record<string, string | null>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setValues({});
    setError(null);
  }, [module?.key]);

  if (!module) return null;

  const changes = Object.fromEntries(Object.entries(values).filter(([, v]) => v === null || v !== ''));
  const save = async () => {
    try {
      await backend.modules.setCredentials(module.key, changes);
      onClose();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <Dialog open onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle>{module.name}</DialogTitle>
      <DialogContent>
        {error && <Alert severity="error">{error}</Alert>}
        {module.credentials.map((field, index) => {
          const stored = field.set && values[field.key] !== null;
          return (
            <TextField
              key={field.key}
              label={field.label}
              type={field.secret ? 'password' : 'text'}
              value={values[field.key] ?? ''}
              onChange={(e) => setValues((v) => ({ ...v, [field.key]: e.target.value }))}
              placeholder={stored ? t('modules.credentials.stored') : undefined}
              helperText={field.optional ? t('modules.credentials.optional') : undefined}
              autoFocus={index === 0}
              autoComplete="off"
              fullWidth
              margin="normal"
              slotProps={{
                input: {
                  endAdornment: stored ? (
                  <InputAdornment position="end">
                    <Button size="small" onClick={() => setValues((v) => ({ ...v, [field.key]: null }))}>
                      {t('modules.credentials.clear')}
                    </Button>
                  </InputAdornment>
                  ) : undefined,
                },
              }}
            />
          );
        })}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" onClick={save} disabled={Object.keys(changes).length === 0}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
