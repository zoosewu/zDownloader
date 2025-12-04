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
  CircularProgress,
} from '@mui/material';
import { useAtom } from 'jotai';
import { listen } from '@tauri-apps/api/event';

import { themeAtom, settingsAtom } from './stores/settingsStore';
import { lightTheme, darkTheme } from './styles/theme';
import MainPage from './pages/MainPage';
import SettingsPage from './pages/SettingsPage';
import AddDownloadPage from './pages/AddDownloadPage';
import ResumeDialog from './components/ResumeDialog';
import Footer from './components/Footer';
import { useNavigation } from './hooks/useNavigation';
import { useClipboard } from './hooks/useClipboard';
import CaptchaDialog from './components/CaptchaDialog';
import LoginDialog from './components/LoginDialog';
import {
  unfinishedDownloadsAtom,
  showResumeDialogAtom,
  downloadTasksAtom,
} from './stores/downloadStore';
import { invoke, waitForTauri } from './utils/tauri';
import { getSettings } from './api/settings';
import './i18n';

function App() {
  const [tauriReady, setTauriReady] = useState(false);

  // Wait for Tauri to be ready
  useEffect(() => {
    const initTauri = async () => {
      console.log('========================================');
      console.log('Zoo Downloader Frontend Initializing...');
      console.log('For debugging, keep this console open.');
      console.log('Backend logs are saved to:');
      console.log('  - Windows: %APPDATA%\\zoo-downloader\\logs\\');
      console.log('  - macOS: ~/Library/Logs/zoo-downloader/');
      console.log('  - Linux: ~/.local/share/zoo-downloader/logs/');
      console.log('========================================');

      console.log('[App] Waiting for Tauri API to be ready...');
      const ready = await waitForTauri();

      if (!ready) {
        console.error('[App] ✗ Tauri failed to initialize - API not available');
        console.error('[App] The application may not function correctly');
        // Don't set tauriReady to true if Tauri is not available
        // This will keep showing the loading screen
        return;
      }

      console.log('[App] Tauri API is ready, verifying with test command...');

      // Verify with a test command
      try {
        const result = await invoke('greet', { name: 'Test' });
        console.log('[App] ✓ Test command succeeded:', result);
        console.log('[App] ✓ Tauri initialized successfully');
        setTauriReady(true);
      } catch (error) {
        console.error('[App] ✗ Failed to invoke test command:', error);
        console.error('[App] Tauri API seems available but commands are failing');
        // Don't set tauriReady to true if commands are failing
      }
    };

    initTauri();
  }, []);

  useClipboard(tauriReady); // Start clipboard monitoring only when Tauri is ready
  const { currentPage, navigateTo } = useNavigation();
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

  // Listen for tray pause all event
  useEffect(() => {
    const unlisten = listen('tray-pause-all', async () => {
      console.log('Tray pause all triggered');
      try {
        await invoke('pause_all_downloads');
      } catch (error) {
        console.error('Failed to pause all downloads:', error);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Listen for tray resume all event
  useEffect(() => {
    const unlisten = listen('tray-resume-all', async () => {
      console.log('Tray resume all triggered');
      try {
        await invoke('resume_all_downloads');
      } catch (error) {
        console.error('Failed to resume all downloads:', error);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

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

  // Show loading screen while Tauri is initializing
  if (!tauriReady) {
    return (
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <Box
          sx={{
            width: '100vw',
            height: '100vh',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexDirection: 'column',
            gap: 2,
          }}
        >
          <CircularProgress size={60} />
          <Typography variant="h6" color="text.secondary">
            正在初始化 Zoo Downloader...
          </Typography>
        </Box>
      </ThemeProvider>
    );
  }

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
              onClick={() => navigateTo('main')}
              sx={{
                fontWeight: currentPage === 'main' || currentPage === 'add-download' ? 'bold' : 'normal',
              }}
            >
              下載
            </Button>
            <Button
              color="inherit"
              onClick={() => navigateTo('settings')}
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
          {currentPage === 'add-download' && <AddDownloadPage />}
        </Box>

        {/* Global Footer */}
        <Footer />

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
