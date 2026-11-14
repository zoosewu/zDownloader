// Add Download Form - Form for adding new downloads with validation
import React, { useState, useEffect } from 'react';
import {
  Box,
  TextField,
  Button,
  Alert,
  Collapse,
  InputAdornment,
} from '@mui/material';
import { useAtomValue, useSetAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend, type InstalledInfo, type StartMode } from '../backend';
import { upsertTaskAtom } from '../stores/downloadStore';
import { invoke } from '../utils/tauri';
import { useNavigation } from '../hooks/useNavigation';
import { useDownloadPath } from '../hooks/useDownloadPath';
import { settingsAtom } from '../stores/settingsStore';
import HeaderFieldsEditor from './HeaderFieldsEditor';
import { emptyHeaderFields, fromHeaderFields, type HeaderFields } from '../utils/headers';

interface AddDownloadFormProps {
  initialUrl?: string;
  onCancel: () => void;
}

const AddDownloadForm: React.FC<AddDownloadFormProps> = ({
  initialUrl = '',
  onCancel,
}) => {
  const { t } = useTranslation();
  const settings = useAtomValue(settingsAtom);
  const [url, setUrl] = useState(initialUrl);
  const [filename, setFilename] = useState('');
  const { path: downloadPath, openDirectoryPicker, errorMessage: pathError } = useDownloadPath(settings.defaultDownloadDir);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [isUserEditedFilename, setIsUserEditedFilename] = useState(false); // Track if user manually edited filename
  /** The installed module that will resolve the URL; null for a direct download. */
  const [module, setModule] = useState<InstalledInfo | null>(null);
  const [headers, setHeaders] = useState<HeaderFields>(emptyHeaderFields);
  const [showHeaders, setShowHeaders] = useState(false);
  const [scheduleAt, setScheduleAt] = useState(''); // datetime-local value; empty = start now
  const upsertTask = useSetAtom(upsertTaskAtom);

  const { navigateTo } = useNavigation();

  // Fill in a download link from the clipboard when the form opens empty.
  useEffect(() => {
    if (initialUrl) return;
    (async () => {
      const content = (await invoke<string>('get_clipboard_content').catch(() => '')).trim();
      if (content && (await backend.urls.inspect(content).catch(() => null))?.downloadable) {
        setUrl(content);
      }
    })();
  }, []); // Only on mount

  // Which module resolves the URL, and the file name it names (modules name their own files).
  useEffect(() => {
    const target = url.trim();
    if (!target) {
      setModule(null);
      return;
    }
    let stale = false; // a newer URL was typed before this answer arrived
    const timeoutId = setTimeout(() => {
      backend.urls
        .inspect(target)
        .then((info) => {
          if (stale) return;
          setModule(info.module);
          if (!isUserEditedFilename) setFilename(info.module ? '' : (info.filename ?? ''));
        })
        .catch(() => !stale && setModule(null));
    }, 400);
    return () => {
      stale = true;
      clearTimeout(timeoutId);
    };
  }, [url, isUserEditedFilename]);

  const isValidUrl = (urlString: string): boolean => {
    try {
      const urlObj = new URL(urlString);
      return urlObj.protocol === 'http:' || urlObj.protocol === 'https:';
    } catch {
      return false;
    }
  };

  /** Queue the download (or schedule it) and show it in the list right away. */
  const addDownload = async (downloadUrl: string, name: string, dir: string) => {
    const start: StartMode = scheduleAt
      ? { mode: 'at', at: new Date(scheduleAt).toISOString() }
      : { mode: 'now' };
    const task = await backend.downloads.add({
      url: downloadUrl,
      filename: name || null,
      dir,
      start,
      headers: fromHeaderFields(headers),
    });
    upsertTask(task);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!url.trim()) {
      setError(t('downloads.errors.emptyUrl'));
      return;
    }

    if (!isValidUrl(url.trim())) {
      setError(t('downloads.errors.invalidUrl'));
      return;
    }

    setError(null);
    setLoading(true);

    try {
      await addDownload(url.trim(), filename.trim(), downloadPath);
      navigateTo('main');
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !loading) {
      handleSubmit(event as any);
    }
  };

  return (
    <Box component="form" onSubmit={handleSubmit}>
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <TextField
        label={t('downloads.details.url')}
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        onKeyPress={handleKeyPress}
        fullWidth
        required
        margin="normal"
        placeholder="https://example.com/file.zip"
      />

      {module && (
        <Alert severity={missingCredentials(module) ? 'warning' : 'info'} sx={{ mt: 1 }}>
          {missingCredentials(module)
            ? t('modules.match.needsCredentials', { name: module.name })
            : t('modules.match.resolvedBy', { name: module.name })}
        </Alert>
      )}

      <TextField
        label={t('downloads.filenameLabel')}
        value={filename}
        onChange={(e) => {
          setFilename(e.target.value);
          setIsUserEditedFilename(true); // Mark as user-edited to prevent auto-overwrite
        }}
        onKeyPress={handleKeyPress}
        fullWidth
        margin="normal"
      />

      <TextField
        label={t('settings.downloadPath')}
        value={downloadPath}
        fullWidth
        margin="normal"
        helperText={pathError}
        error={!!pathError}
        InputProps={{
          readOnly: true,
          endAdornment: (
            <InputAdornment position="end">
              <Button onClick={openDirectoryPicker}>{t('common.browse')}</Button>
            </InputAdornment>
          ),
        }}
      />

      <TextField
        label={t('downloads.schedule.startAt')}
        type="datetime-local"
        value={scheduleAt}
        onChange={(e) => setScheduleAt(e.target.value)}
        fullWidth
        margin="normal"
        InputLabelProps={{ shrink: true }}
        inputProps={{ min: new Date(Date.now() - new Date().getTimezoneOffset() * 60000).toISOString().slice(0, 16) }}
      />

      <Button size="small" onClick={() => setShowHeaders((v) => !v)} sx={{ mt: 1, textTransform: 'none' }}>
        {t('headers.toggle')}
      </Button>
      <Collapse in={showHeaders}>
        <HeaderFieldsEditor value={headers} onChange={setHeaders} />
      </Collapse>

      <Box sx={{ display: 'flex', gap: 2, mt: 3 }}>
        <Button
          variant="contained"
          type="submit"
          disabled={loading || !url.trim()}
          sx={{ flex: 1 }}
        >
          {t('downloads.addDownload')}
        </Button>
        <Button
          variant="outlined"
          onClick={onCancel}
          disabled={loading}
          sx={{ flex: 1 }}
        >
          {t('common.cancel')}
        </Button>
      </Box>

    </Box>
  );
};

/** Required account details the module has no value for yet. */
function missingCredentials(module: InstalledInfo): boolean {
  return module.credentials.some((c) => !c.optional && !c.set);
}

export default AddDownloadForm;
