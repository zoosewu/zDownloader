import { useState } from 'react';
import { useAtomValue } from 'jotai';
import { settingsAtom } from '../stores/settingsStore';
import { open } from '@tauri-apps/plugin-dialog';

export function useDownloadPath(initialPath?: string) {
  const settings = useAtomValue(settingsAtom);
  const [path, setPath] = useState(initialPath || settings.defaultDownloadDir);
  const [isValid, setIsValid] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const openDirectoryPicker = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: path,
    });

    if (selected && typeof selected === 'string') {
      setPath(selected);
      setIsValid(true);
      setErrorMessage(null);
    }
  };

  const validatePath = async () => {
    // Basic validation - backend will perform full validation
    if (!path || path.trim() === '') {
      setIsValid(false);
      setErrorMessage('Download path cannot be empty');
      return false;
    }

    setIsValid(true);
    setErrorMessage(null);
    return true;
  };

  return {
    path,
    setPath,
    isValid,
    errorMessage,
    openDirectoryPicker,
    validatePath,
  };
}
