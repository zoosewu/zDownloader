// Zoo Downloader - Main Application Component
import { useMemo, useEffect } from 'react';
import { ThemeProvider, CssBaseline, Box } from '@mui/material';
import { useAtom } from 'jotai';
import { listen } from '@tauri-apps/api/event';

import { themeAtom } from './stores/settingsStore';
import { lightTheme, darkTheme } from './styles/theme';
import MainPage from './pages/MainPage';
import ResumeDialog from './components/ResumeDialog';
import {
  unfinishedDownloadsAtom,
  showResumeDialogAtom,
  downloadTasksAtom,
} from './stores/downloadStore';
import { invoke } from '@tauri-apps/api/core';
import './i18n';

function App() {
  const [currentTheme] = useAtom(themeAtom);
  const [unfinishedDownloads, setUnfinishedDownloads] = useAtom(unfinishedDownloadsAtom);
  const [showResumeDialog, setShowResumeDialog] = useAtom(showResumeDialogAtom);
  const [, setDownloadTasks] = useAtom(downloadTasksAtom);

  const theme = useMemo(() => {
    return currentTheme === 'Light' ? lightTheme : darkTheme;
  }, [currentTheme]);

  // Listen for unfinished downloads detected event
  useEffect(() => {
    const unlisten = listen('unfinished-downloads-detected', (event: any) => {
      console.log('Unfinished downloads detected:', event.payload);
      setUnfinishedDownloads(event.payload || []);
      if (event.payload && event.payload.length > 0) {
        setShowResumeDialog(true);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [setUnfinishedDownloads, setShowResumeDialog]);

  const handleResumeAll = async () => {
    try {
      for (const task of unfinishedDownloads) {
        await invoke('start_download', { taskId: task.id });
      }
      setShowResumeDialog(false);
    } catch (error) {
      console.error('Failed to resume downloads:', error);
    }
  };

  const handleResumeSelected = async (taskIds: string[]) => {
    try {
      for (const taskId of taskIds) {
        await invoke('start_download', { taskId });
      }
      setShowResumeDialog(false);
    } catch (error) {
      console.error('Failed to resume selected downloads:', error);
    }
  };

  const handleDiscardAll = () => {
    // Remove unfinished tasks from the list
    setDownloadTasks(prev =>
      prev.filter(task => !unfinishedDownloads.find(ut => ut.id === task.id))
    );
    setUnfinishedDownloads([]);
    setShowResumeDialog(false);
  };

  const handleCloseDialog = () => {
    setShowResumeDialog(false);
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          width: '100vw',
          height: '100vh',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'auto',
        }}
      >
        <MainPage />
        <ResumeDialog
          open={showResumeDialog}
          tasks={unfinishedDownloads}
          onResumeAll={handleResumeAll}
          onResumeSelected={handleResumeSelected}
          onDiscardAll={handleDiscardAll}
          onClose={handleCloseDialog}
        />
      </Box>
    </ThemeProvider>
  );
}

export default App;
