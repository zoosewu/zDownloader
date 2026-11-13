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
    const timeoutId = setTimeout(() => {
      backend.urls
        .inspect(target)
        .then((info) => {
          setModule(info.module);
          if (!isUserEditedFilename) setFilename(info.module ? '' : (info.filename ?? ''));
        })
        .catch(() => setModule(null));
    }, 400);
    return () => clearTimeout(timeoutId);
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

    console.log('[AddDownloadForm] ========== Form Submission Started ==========');
    console.log('[AddDownloadForm] URL:', url);
    console.log('[AddDownloadForm] Filename:', filename);
    console.log('[AddDownloadForm] Download Path:', downloadPath);

    if (!url.trim()) {
      console.log('[AddDownloadForm] ✗ Validation failed: URL is empty');
      setError('請輸入下載網址');
      return;
    }

    if (!isValidUrl(url.trim())) {
      console.log('[AddDownloadForm] ✗ Validation failed: Invalid URL format');
      setError('無效的網址格式');
      return;
    }

    console.log('[AddDownloadForm] ✓ Basic validation passed');

    setError(null);
    setLoading(true);

    try {
      await addDownload(url.trim(), filename.trim(), downloadPath);

      console.log('[AddDownloadForm] Navigating to main page...');
      navigateTo('main');
      console.log('[AddDownloadForm] ========== Form Submission Completed ==========');
    } catch (e: any) {
      console.error('[AddDownloadForm] ✗ Error during form submission:', e);
      console.error('[AddDownloadForm] Error message:', e.message);
      console.error('[AddDownloadForm] Error stack:', e.stack);
      setError(e.message || '建立下載任務失敗');
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
        label="下載網址"
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        onKeyPress={handleKeyPress}
        fullWidth
        required
        margin="normal"
        placeholder="https://example.com/file.zip"
        helperText="請輸入要下載的檔案網址"
      />

      {module && (
        <Alert severity={missingCredentials(module) ? 'warning' : 'info'} sx={{ mt: 1 }}>
          {missingCredentials(module)
            ? t('modules.match.needsCredentials', { name: module.name })
            : t('modules.match.resolvedBy', { name: module.name })}
        </Alert>
      )}

      <TextField
        label="檔案名稱（選填）"
        value={filename}
        onChange={(e) => {
          setFilename(e.target.value);
          setIsUserEditedFilename(true); // Mark as user-edited to prevent auto-overwrite
        }}
        onKeyPress={handleKeyPress}
        fullWidth
        margin="normal"
        placeholder="留空則使用網址中的檔名"
      />

      <TextField
        label="下載路徑"
        value={downloadPath}
        fullWidth
        margin="normal"
        helperText={pathError || "檔案將儲存到此路徑"}
        error={!!pathError}
        InputProps={{
          readOnly: true,
          endAdornment: (
            <InputAdornment position="end">
              <Button onClick={openDirectoryPicker}>瀏覽</Button>
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
          {loading ? '建立中...' : '新增下載'}
        </Button>
        <Button
          variant="outlined"
          onClick={onCancel}
          disabled={loading}
          sx={{ flex: 1 }}
        >
          取消
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
