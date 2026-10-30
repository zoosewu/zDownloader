// Advanced Settings Component - UI for advanced application settings
import {
  Box,
  Button,
  FormControl,
  FormControlLabel,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Switch,
  TextField,
  Typography,
  Alert,
  InputAdornment,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAtom } from 'jotai';
import { backend, type AppSettings, type SpeedLimitMode } from '../backend';
import { settingsAtom } from '../stores/settingsStore';
import { open, save } from '@tauri-apps/plugin-dialog';

export default function AdvancedSettings() {
  const { t } = useTranslation();
  const [, setGlobalSettings] = useAtom(settingsAtom);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  async function loadSettings() {
    try {
      setLoading(true);
      const loadedSettings = await backend.settings.get();
      setSettings(loadedSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.loadFailed') || 'Failed to load settings' });
    } finally {
      setLoading(false);
    }
  }

  async function handleSave() {
    if (!settings) return;

    try {
      setSaving(true);
      setGlobalSettings(await backend.settings.update(settings));
      setMessage({ type: 'success', text: t('settings.saveSuccess') || 'Settings saved successfully' });
    } catch (error) {
      console.error('Failed to save settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.saveFailed') || 'Failed to save settings' });
    } finally {
      setSaving(false);
    }
  }

  async function handleReset() {
    if (!confirm(t('settings.confirmReset') || 'Reset all settings to default values?')) {
      return;
    }

    try {
      const defaultSettings = await backend.settings.reset();
      setGlobalSettings(defaultSettings);
      setSettings(defaultSettings);
      setMessage({ type: 'success', text: t('settings.resetSuccess') || 'Settings reset to defaults' });
    } catch (error) {
      console.error('Failed to reset settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.resetFailed') || 'Failed to reset settings' });
    }
  }

  async function handleExport() {
    try {
      const exportPath = await save({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        defaultPath: 'zoo-downloader-settings.json',
      });

      if (exportPath) {
        await backend.settings.export(exportPath as string);
        setMessage({ type: 'success', text: t('settings.exportSuccess') || 'Settings exported successfully' });
      }
    } catch (error) {
      console.error('Failed to export settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.exportFailed') || 'Failed to export settings' });
    }
  }

  async function handleImport() {
    try {
      const importPath = await open({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        multiple: false,
      });

      if (importPath) {
        const imported = await backend.settings.import(importPath as string);
        setGlobalSettings(imported);
        setSettings(imported);
        setMessage({ type: 'success', text: t('settings.importSuccess') || 'Settings imported successfully' });
      }
    } catch (error) {
      console.error('Failed to import settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.importFailed') || 'Failed to import settings' });
    }
  }

  if (loading) {
    return <Box sx={{ p: 3, textAlign: 'center' }}><Typography>Loading...</Typography></Box>;
  }

  if (!settings) {
    return <Box sx={{ p: 3, textAlign: 'center' }}><Typography>Failed to load settings</Typography></Box>;
  }

  return (
    <Paper elevation={1} sx={{ p: 3 }}>
      <Stack spacing={3}>
        {message && (
          <Alert severity={message.type} onClose={() => setMessage(null)}>
            {message.text}
          </Alert>
        )}

        {/* Download Settings */}
        <Box>
          <Typography variant="h6" gutterBottom>
            {t('settings.downloadSettings') || 'Download Settings'}
          </Typography>
          <Stack spacing={2}>
            <TextField
              fullWidth
              type="number"
              label={t('settings.segmentSize') || 'Segment Size (MB)'}
              value={settings.segmentSizeMb}
              onChange={(e) => setSettings({ ...settings, segmentSizeMb: parseInt(e.target.value) || 10 })}
              inputProps={{ min: 1, max: 100 }}
              helperText={t('settings.segmentSizeHelp') || 'Size of each download segment (1-100 MB)'}
            />
          </Stack>
        </Box>

        {/* Speed Limit */}
        <Box>
          <Typography variant="h6" gutterBottom>
            {t('settings.speedLimit') || 'Speed Limit'}
          </Typography>
          <Stack spacing={2}>
            <FormControl fullWidth>
              <InputLabel>{t('settings.speedLimitMode') || 'Speed Limit Mode'}</InputLabel>
              <Select
                value={settings.speedLimitMode}
                label={t('settings.speedLimitMode') || 'Speed Limit Mode'}
                onChange={(e) => setSettings({ ...settings, speedLimitMode: e.target.value as SpeedLimitMode })}
              >
                <MenuItem value="unlimited">{t('settings.unlimited') || 'Unlimited'}</MenuItem>
                <MenuItem value="fixed">{t('settings.fixed') || 'Fixed'}</MenuItem>
              </Select>
            </FormControl>

            {settings.speedLimitMode === 'fixed' && (
              <TextField
                fullWidth
                type="number"
                label={t('settings.speedLimitValue') || 'Speed Limit (KB/s)'}
                value={settings.speedLimitKbps ?? ''}
                onChange={(e) => {
                  const value = e.target.value === '' ? null : parseInt(e.target.value);
                  setSettings({ ...settings, speedLimitKbps: value });
                }}
                InputProps={{
                  endAdornment: <InputAdornment position="end">KB/s</InputAdornment>,
                }}
              />
            )}
          </Stack>
        </Box>

        {/* Auto Retry Settings */}
        <Box>
          <Typography variant="h6" gutterBottom>
            {t('settings.autoRetry') || 'Auto Retry'}
          </Typography>
          <Stack spacing={2}>
            <FormControlLabel
              control={
                <Switch
                  checked={settings.autoRetryEnabled}
                  onChange={(e) => setSettings({ ...settings, autoRetryEnabled: e.target.checked })}
                />
              }
              label={t('settings.enableAutoRetry') || 'Enable Auto Retry'}
            />

            {settings.autoRetryEnabled && (
              <>
                <TextField
                  fullWidth
                  type="number"
                  label={t('settings.maxRetryCount') || 'Max Retry Count'}
                  value={settings.maxRetryCount}
                  onChange={(e) => setSettings({ ...settings, maxRetryCount: parseInt(e.target.value) || 3 })}
                  inputProps={{ min: 1, max: 10 }}
                />
                <TextField
                  fullWidth
                  type="number"
                  label={t('settings.retryInterval') || 'Retry Interval (seconds)'}
                  value={settings.retryIntervalSecs}
                  onChange={(e) => setSettings({ ...settings, retryIntervalSecs: parseInt(e.target.value) || 5 })}
                  inputProps={{ min: 1, max: 60 }}
                />
              </>
            )}
          </Stack>
        </Box>

        {/* Proxy Settings */}
        <Box>
          <Typography variant="h6" gutterBottom>
            {t('settings.proxySettings') || 'Proxy Settings'}
          </Typography>
          <Stack spacing={2}>
            <FormControlLabel
              control={
                <Switch
                  checked={settings.enableProxy}
                  onChange={(e) => setSettings({ ...settings, enableProxy: e.target.checked })}
                />
              }
              label={t('proxy.enabled')}
            />

            {settings.enableProxy && (
              <FormControlLabel
                control={
                  <Switch
                    checked={settings.proxyFallbackDirect}
                    onChange={(e) => setSettings({ ...settings, proxyFallbackDirect: e.target.checked })}
                  />
                }
                label={t('settings.proxyFallbackDirect')}
              />
            )}
          </Stack>
        </Box>

        {/* Import/Export */}
        <Box>
          <Typography variant="h6" gutterBottom>
            {t('settings.importExport') || 'Import/Export'}
          </Typography>
          <Stack direction="row" spacing={2}>
            <Button variant="outlined" onClick={handleExport}>
              {t('settings.export') || 'Export Settings'}
            </Button>
            <Button variant="outlined" onClick={handleImport}>
              {t('settings.import') || 'Import Settings'}
            </Button>
            <Button variant="outlined" color="warning" onClick={handleReset}>
              {t('settings.resetToDefaults') || 'Reset to Defaults'}
            </Button>
          </Stack>
        </Box>

        {/* Save Button */}
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', gap: 2 }}>
          <Button variant="outlined" onClick={loadSettings} disabled={saving}>
            {t('common.reset')}
          </Button>
          <Button variant="contained" onClick={handleSave} disabled={saving}>
            {saving ? t('common.saving') || 'Saving...' : t('common.save')}
          </Button>
        </Box>
      </Stack>
    </Paper>
  );
}
