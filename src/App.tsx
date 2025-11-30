// Zoo Downloader - Main Application Component
import { useMemo, useEffect, useState } from 'react';
import {
  ThemeProvider,
  CssBaseline,
  Box,
  AppBar,
  Toolbar,
  Typography,
  Button,
} from '@mui/material';
import { useAtom } from 'jotai';
import { listen } from '@tauri-apps/api/event';

import { themeAtom, settingsAtom } from './stores/settingsStore';
import { lightTheme, darkTheme } from './styles/theme';
import MainPage from './pages/MainPage';
import SettingsPage from './pages/SettingsPage';
import ResumeDialog from './components/ResumeDialog';
import CaptchaDialog from './components/CaptchaDialog';
import LoginDialog from './components/LoginDialog';
import {
  unfinishedDownloadsAtom,
  showResumeDialogAtom,
  downloadTasksAtom,
} from './stores/downloadStore';
import { invoke } from '@tauri-apps/api/core';
import { getSettings } from './api/settings';
import './i18n';

function App() {
  const [currentPage, setCurrentPage] = useState<'main' | 'settings'>('main');
  const [currentTheme] = useAtom(themeAtom);
  const [, setSettings] = useAtom(settingsAtom);
  const [unfinishedDownloads, setUnfinishedDownloads] = useAtom(unfinishedDownloadsAtom);
  const [showResumeDialog, setShowResumeDialog] = useAtom(showResumeDialogAtom);
  const [, setDownloadTasks] = useAtom(downloadTasksAtom);

  const theme = useMemo(() => {
    return currentTheme === 'Light' ? lightTheme : darkTheme;
  }, [currentTheme]);

  // Load settings from backend on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const backendSettings = await getSettings();
        setSettings(backendSettings);
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    };
    loadSettings();
  }, [setSettings]);

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
        }}
      >
        {/* Navigation Bar */}
        <AppBar position="static" elevation={1}>
          <Toolbar>
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Zoo Downloader
            </Typography>
            <Button
              color="inherit"
              onClick={() => setCurrentPage('main')}
              sx={{
                fontWeight: currentPage === 'main' ? 'bold' : 'normal',
              }}
            >
              下載
            </Button>
            <Button
              color="inherit"
              onClick={() => setCurrentPage('settings')}
              sx={{
                fontWeight: currentPage === 'settings' ? 'bold' : 'normal',
              }}
            >
              設定
            </Button>
          </Toolbar>
        </AppBar>

        {/* Page Content */}
        <Box sx={{ flex: 1, overflow: 'auto' }}>
          {currentPage === 'main' && <MainPage />}
          {currentPage === 'settings' && <SettingsPage />}
        </Box>

        {/* Dialogs */}
        <ResumeDialog
          open={showResumeDialog}
          tasks={unfinishedDownloads}
          onResumeAll={handleResumeAll}
          onResumeSelected={handleResumeSelected}
          onDiscardAll={handleDiscardAll}
          onClose={handleCloseDialog}
        />
        <CaptchaDialog />
        <LoginDialog />
      </Box>
    </ThemeProvider>
  );
}

export default App;
