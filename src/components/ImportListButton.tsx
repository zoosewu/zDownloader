// Import a text file of links (one per line) into the add-downloads window, to check them
// before they are added.
import React from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Upload as UploadIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { backend } from '../backend';

const ImportListButton: React.FC<{ onError?: (message: string) => void }> = ({ onError }) => {
  const { t } = useTranslation();
  const [loading, setLoading] = React.useState(false);

  const importList = async () => {
    setLoading(true);
    try {
      const file = await open({ multiple: false, filters: [{ name: 'Text', extensions: ['txt'] }] });
      if (file) await backend.addWindow.import(file);
    } catch (error) {
      onError?.(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      color="inherit"
      startIcon={loading ? <CircularProgress size={14} /> : <UploadIcon fontSize="small" />}
      onClick={importList}
      disabled={loading}
    >
      {t('addWindow.importList')}
    </Button>
  );
};

export default ImportListButton;
