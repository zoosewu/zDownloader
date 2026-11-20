// Account details a module logs in with, with the module's own help on getting them. Stored
// values are never shown: leave a field empty to keep it, or clear it.
import { useEffect, useState } from 'react';
import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  InputAdornment,
  Link,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import DialogError from '../DialogError';
import { HelpOutlined as HelpIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { backend, type InstalledInfo } from '../../backend';
import { localized } from '../../utils/localized';

interface CredentialsDialogProps {
  module: InstalledInfo | null;
  onClose: () => void;
}

export default function CredentialsDialog({ module, onClose }: CredentialsDialogProps) {
  const { t, i18n } = useTranslation();
  const text = (value: Parameters<typeof localized>[0]) => localized(value, i18n.language);
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
        {(module.credentialsHelp || module.credentialsHelpUrl) && (
          <Typography variant="body2" color="text.secondary" sx={{ whiteSpace: 'pre-wrap' }}>
            {module.credentialsHelp && text(module.credentialsHelp)}
            {module.credentialsHelpUrl && (
              <>
                {module.credentialsHelp && ' '}
                <Link href={module.credentialsHelpUrl} target="_blank" rel="noreferrer">
                  {t('modules.credentials.more')}
                </Link>
              </>
            )}
          </Typography>
        )}
        {module.credentials.map((field, index) => {
          const stored = field.set && values[field.key] !== null;
          const helper = [field.help && text(field.help), field.optional && t('modules.credentials.optional')].filter(Boolean).join(' · ');
          return (
            <TextField
              key={field.key}
              label={text(field.label)}
              type={field.secret ? 'password' : 'text'}
              value={values[field.key] ?? ''}
              onChange={(e) => setValues((v) => ({ ...v, [field.key]: e.target.value }))}
              placeholder={stored ? t('modules.credentials.stored') : undefined}
              helperText={helper || undefined}
              autoFocus={index === 0}
              autoComplete="off"
              fullWidth
              margin="normal"
              slotProps={{
                formHelperText: { sx: { whiteSpace: 'pre-wrap' } },
                input: {
                  endAdornment:
                    stored || field.helpUrl ? (
                      <InputAdornment position="end">
                        {stored && (
                          <Button size="small" onClick={() => setValues((v) => ({ ...v, [field.key]: null }))}>
                            {t('modules.credentials.clear')}
                          </Button>
                        )}
                        {field.helpUrl && (
                          <Tooltip title={t('modules.credentials.more')}>
                            <IconButton size="small" edge="end" href={field.helpUrl} target="_blank" rel="noreferrer">
                              <HelpIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        )}
                      </InputAdornment>
                    ) : undefined,
                },
              }}
            />
          );
        })}
      </DialogContent>
      <DialogActions>
        <DialogError error={error} />
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" onClick={save} disabled={Object.keys(changes).length === 0}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
