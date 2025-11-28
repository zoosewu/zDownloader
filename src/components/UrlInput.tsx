// URL input component - paste URL with clipboard prompt
import React, { useState } from 'react';
import {
  Box,
  TextField,
  Button,
  Paper,
  Alert,
  Collapse,
  IconButton,
} from '@mui/material';
import {
  Close as CloseIcon,
  ContentPaste as PasteIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';

interface UrlInputProps {
  onSubmit: (url: string, filename?: string) => void;
  showClipboardHint?: boolean;
  clipboardUrl?: string;
  onAcceptClipboard?: () => void;
  onDismissClipboard?: () => void;
}

const UrlInput: React.FC<UrlInputProps> = ({
  onSubmit,
  showClipboardHint = false,
  clipboardUrl,
  onAcceptClipboard,
  onDismissClipboard,
}) => {
  const { t } = useTranslation();
  const [url, setUrl] = useState('');
  const [filename, setFilename] = useState('');
  const [error, setError] = useState('');

  const isValidUrl = (urlString: string): boolean => {
    try {
      const urlObj = new URL(urlString);
      return urlObj.protocol === 'http:' || urlObj.protocol === 'https:';
    } catch {
      return false;
    }
  };

  const handleSubmit = () => {
    if (!url.trim()) {
      setError(t('downloads.errors.emptyUrl'));
      return;
    }

    if (!isValidUrl(url.trim())) {
      setError(t('downloads.errors.invalidUrl'));
      return;
    }

    setError('');
    onSubmit(url.trim(), filename.trim() || undefined);
    setUrl('');
    setFilename('');
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      handleSubmit();
    }
  };

  const handlePasteClick = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text && isValidUrl(text)) {
        setUrl(text);
        setError('');
      } else {
        setError(t('downloads.errors.invalidClipboard'));
      }
    } catch (err) {
      setError(t('downloads.errors.clipboardPermission'));
    }
  };

  const handleAcceptClipboard = () => {
    if (clipboardUrl) {
      setUrl(clipboardUrl);
      setError('');
    }
    onAcceptClipboard?.();
  };

  return (
    <Paper elevation={0} sx={{ padding: 2, backgroundColor: 'background.paper' }}>
      {/* Clipboard hint banner */}
      <Collapse in={showClipboardHint && !!clipboardUrl}>
        <Alert
          severity="info"
          sx={{ marginBottom: 2 }}
          action={
            <Box>
              <Button
                color="inherit"
                size="small"
                onClick={handleAcceptClipboard}
              >
                {t('downloads.clipboard.accept')}
              </Button>
              <IconButton
                aria-label="close"
                color="inherit"
                size="small"
                onClick={onDismissClipboard}
              >
                <CloseIcon fontSize="inherit" />
              </IconButton>
            </Box>
          }
        >
          {t('downloads.clipboard.detected')}: {clipboardUrl?.substring(0, 50)}...
        </Alert>
      </Collapse>

      {/* URL input field */}
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <TextField
          fullWidth
          label={t('downloads.urlLabel')}
          placeholder={t('downloads.urlPlaceholder')}
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyPress={handleKeyPress}
          error={!!error}
          helperText={error}
          InputProps={{
            endAdornment: (
              <IconButton
                onClick={handlePasteClick}
                edge="end"
                aria-label={t('downloads.actions.paste')}
              >
                <PasteIcon />
              </IconButton>
            ),
          }}
        />

        {/* Optional filename input */}
        <TextField
          fullWidth
          label={t('downloads.filenameLabel')}
          placeholder={t('downloads.filenamePlaceholder')}
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
          onKeyPress={handleKeyPress}
          helperText={t('downloads.filenameHelp')}
        />

        {/* Submit button */}
        <Button
          variant="contained"
          size="large"
          onClick={handleSubmit}
          disabled={!url.trim()}
        >
          {t('downloads.actions.download')}
        </Button>
      </Box>
    </Paper>
  );
};

export default UrlInput;
