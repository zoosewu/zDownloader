// Zoo Downloader - Main Application Component
import { useMemo } from 'react';
import { ThemeProvider, CssBaseline, Box } from '@mui/material';
import { useAtom } from 'jotai';

import { themeAtom } from './stores/settingsStore';
import { lightTheme, darkTheme } from './styles/theme';
import MainPage from './pages/MainPage';
import './i18n';

function App() {
  const [currentTheme] = useAtom(themeAtom);

  const theme = useMemo(() => {
    return currentTheme === 'Light' ? lightTheme : darkTheme;
  }, [currentTheme]);

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
      </Box>
    </ThemeProvider>
  );
}

export default App;
