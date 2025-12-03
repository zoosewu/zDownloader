// Developer Settings component - controls for development and testing
import React from 'react';
import {
  Box,
  Typography,
  FormControlLabel,
  Switch,
  Paper,
  Alert,
  AlertTitle,
  Stack,
  Button,
} from '@mui/material';
import { useAtom, useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import {
  mockModeEnabledAtom,
  useMockTasksAtom,
  autoProgressEnabledAtom,
  isDevEnvironmentAtom,
} from '../stores/devStore';
import { downloadTasksAtom } from '../stores/downloadStore';
import { generateMockTasks } from '../utils/mockData';

/**
 * Developer Settings - controls for mock mode and testing features
 * Only available in development environment
 */
const DeveloperSettings: React.FC = () => {
  const { t } = useTranslation();
  const isDev = useAtomValue(isDevEnvironmentAtom);
  const [mockMode, setMockMode] = useAtom(mockModeEnabledAtom);
  const [useMockTasks, setUseMockTasks] = useAtom(useMockTasksAtom);
  const [autoProgress, setAutoProgress] = useAtom(autoProgressEnabledAtom);
  const [, setDownloadTasks] = useAtom(downloadTasksAtom);

  // Load mock tasks into the store
  const handleLoadMockTasks = () => {
    const mockTasks = generateMockTasks();
    setDownloadTasks(mockTasks);
  };

  // Clear all tasks
  const handleClearTasks = () => {
    setDownloadTasks([]);
  };

  if (!isDev) {
    return (
      <Box>
        <Alert severity="info">
          <AlertTitle>{t('dev.settings.notAvailable')}</AlertTitle>
          {t('dev.settings.productionMode')}
        </Alert>
      </Box>
    );
  }

  return (
    <Box>
      <Alert severity="warning" sx={{ mb: 3 }}>
        <AlertTitle>{t('dev.settings.warningTitle')}</AlertTitle>
        {t('dev.settings.warningDescription')}
      </Alert>

      <Stack spacing={3}>
        {/* Mock Mode Section */}
        <Paper elevation={1} sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>
            {t('dev.settings.mockMode.title')}
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            {t('dev.settings.mockMode.description')}
          </Typography>

          <FormControlLabel
            control={
              <Switch
                checked={mockMode}
                onChange={(e) => setMockMode(e.target.checked)}
                color="primary"
              />
            }
            label={t('dev.settings.mockMode.enableLabel')}
          />

          {mockMode && (
            <Alert severity="info" sx={{ mt: 2 }}>
              {t('dev.settings.mockMode.enabledMessage')}
            </Alert>
          )}
        </Paper>

        {/* Mock Tasks Section */}
        <Paper elevation={1} sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>
            {t('dev.settings.mockTasks.title')}
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            {t('dev.settings.mockTasks.description')}
          </Typography>

          <Stack spacing={2}>
            <FormControlLabel
              control={
                <Switch
                  checked={useMockTasks}
                  onChange={(e) => setUseMockTasks(e.target.checked)}
                  color="primary"
                />
              }
              label={t('dev.settings.mockTasks.enableLabel')}
            />

            {useMockTasks && (
              <FormControlLabel
                control={
                  <Switch
                    checked={autoProgress}
                    onChange={(e) => setAutoProgress(e.target.checked)}
                    color="primary"
                  />
                }
                label={t('dev.settings.mockTasks.autoProgressLabel')}
              />
            )}

            <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
              <Button
                variant="outlined"
                onClick={handleLoadMockTasks}
                size="small"
              >
                {t('dev.settings.mockTasks.loadButton')}
              </Button>
              <Button
                variant="outlined"
                color="error"
                onClick={handleClearTasks}
                size="small"
              >
                {t('dev.settings.mockTasks.clearButton')}
              </Button>
            </Stack>
          </Stack>
        </Paper>

        {/* Info Section */}
        <Paper elevation={1} sx={{ p: 3, backgroundColor: 'action.hover' }}>
          <Typography variant="subtitle2" gutterBottom>
            {t('dev.settings.info.title')}
          </Typography>
          <Typography variant="body2" component="div">
            <ul style={{ margin: 0, paddingLeft: 20 }}>
              <li>{t('dev.settings.info.item1')}</li>
              <li>{t('dev.settings.info.item2')}</li>
              <li>{t('dev.settings.info.item3')}</li>
              <li>{t('dev.settings.info.item4')}</li>
            </ul>
          </Typography>
        </Paper>
      </Stack>
    </Box>
  );
};

export default DeveloperSettings;
