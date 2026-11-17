// General Settings Component - UI for general application settings
import {
  Box,
  Button,
  FormControl,
  FormControlLabel,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  Switch,
  TextField,
  Typography,
  Alert,
} from '@mui/material';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAtom } from 'jotai';
import { settingsAtom } from '../stores/settingsStore';
import { backend, type AppSettings, type Theme } from '../backend';
import { open } from '@tauri-apps/plugin-dialog';
import AppUpdateSection from './AppUpdateSection';

export default function GeneralSettings() {
  const { t, i18n } = useTranslation();
  const [saved, setGlobalSettings] = useAtom(settingsAtom);
  // The form edits a draft of the app's settings (loaded at startup, replaced by every save);
  // the backend is the only place they are written.
  const [settings, setSettings] = useState<AppSettings>(saved);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  async function handleSave() {
    try {
      setSaving(true);
      const next = await backend.settings.update(settings);
      setGlobalSettings(next);
      setSettings(next);

      // Update i18n language if changed
      if (settings.language !== i18n.language) {
        await i18n.changeLanguage(settings.language);
      }

      setMessage({ type: 'success', text: t('settings.saveSuccess') || 'Settings saved successfully' });
    } catch (error) {
      console.error('Failed to save settings:', error);
      setMessage({ type: 'error', text: t('settings.errors.saveFailed') || 'Failed to save settings' });
    } finally {
      setSaving(false);
    }
  }

  async function handleSelectPath() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.defaultDownloadDir,
      });

      if (selected && settings) {
        setSettings({
          ...settings,
          defaultDownloadDir: selected as string,
        });
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  }

  return (
    <Box>
      <Stack spacing={2.5}>
        {message && (
          <Alert severity={message.type} onClose={() => setMessage(null)}>
            {message.text}
          </Alert>
        )}

        {/* Language */}
        <FormControl fullWidth>
          <InputLabel>{t('settings.language')}</InputLabel>
          <Select
            value={settings.language}
            label={t('settings.language')}
            onChange={(e) => setSettings({ ...settings, language: e.target.value })}
          >
            <MenuItem value="zh-TW">繁體中文</MenuItem>
            <MenuItem value="en">English</MenuItem>
            <MenuItem value="ja">日本語</MenuItem>
          </Select>
        </FormControl>

        {/* Theme */}
        <FormControl fullWidth>
          <InputLabel>{t('settings.theme')}</InputLabel>
          <Select
            value={settings.theme}
            label={t('settings.theme')}
            onChange={(e) => setSettings({ ...settings, theme: e.target.value as Theme })}
          >
            <MenuItem value="light">{t('settings.themeLight')}</MenuItem>
            <MenuItem value="dark">{t('settings.themeDark')}</MenuItem>
          </Select>
        </FormControl>

        {/* Download Path */}
        <Box>
          <Typography variant="subtitle2" gutterBottom>
            {t('settings.downloadPath')}
          </Typography>
          <Stack direction="row" spacing={1}>
            <TextField
              fullWidth
              value={settings.defaultDownloadDir}
              slotProps={{ input: { readOnly: true } }}
              size="small"
            />
            <Button variant="outlined" onClick={handleSelectPath}>
              {t('settings.selectPath')}
            </Button>
          </Stack>
        </Box>

        {/* Max Concurrent Downloads */}
        <TextField
          fullWidth
          type="number"
          label={t('settings.maxConcurrent')}
          value={settings.maxConcurrentDownloads ?? ''}
          onChange={(e) => {
            const value = e.target.value === '' ? null : parseInt(e.target.value);
            setSettings({ ...settings, maxConcurrentDownloads: value });
          }}
          helperText={t('settings.maxConcurrentHelp') || 'Leave empty for unlimited'}
        />

        {/* Max Threads per Download */}
        <TextField
          fullWidth
          type="number"
          label={t('settings.maxThreads')}
          value={settings.threadCount}
          onChange={(e) => setSettings({ ...settings, threadCount: parseInt(e.target.value) || 8 })}
          slotProps={{ htmlInput: { min: 1, max: 32 } }}
        />

        {/* UI Settings */}
        <Box>
          <Typography variant="subtitle2" gutterBottom>
            {t('settings.uiSettings') || 'UI Settings'}
          </Typography>
          <Stack spacing={1}>
            <FormControlLabel
              control={
                <Switch
                  checked={settings.enableClipboardMonitoring}
                  onChange={(e) =>
                    setSettings({ ...settings, enableClipboardMonitoring: e.target.checked })
                  }
                />
              }
              label={t('settings.clipboardMonitor')}
            />
            <FormControlLabel
              control={
                <Switch
                  checked={settings.autoStartLinks}
                  onChange={(e) => setSettings({ ...settings, autoStartLinks: e.target.checked })}
                />
              }
              label={t('settings.autoStartLinks')}
            />
            <FormControlLabel
              control={
                <Switch
                  checked={settings.enableNotifications}
                  onChange={(e) => setSettings({ ...settings, enableNotifications: e.target.checked })}
                />
              }
              label={t('settings.notifications')}
            />
            <FormControlLabel
              control={
                <Switch
                  checked={settings.enableSystemTray}
                  onChange={(e) => setSettings({ ...settings, enableSystemTray: e.target.checked })}
                />
              }
              label={t('settings.systemTray') || 'Enable System Tray'}
            />
          </Stack>
        </Box>

        <Box>
          <Typography variant="subtitle2" gutterBottom>
            {t('update.title')}
          </Typography>
          <Stack spacing={1}>
            <FormControlLabel
              control={
                <Switch
                  checked={settings.checkAppUpdates}
                  onChange={(e) => setSettings({ ...settings, checkAppUpdates: e.target.checked })}
                />
              }
              label={t('update.automatic')}
            />
            <AppUpdateSection />
          </Stack>
        </Box>

        {/* Save Button */}
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', gap: 2 }}>
          <Button variant="outlined" onClick={() => setSettings(saved)} disabled={saving}>
            {t('common.reset')}
          </Button>
          <Button variant="contained" onClick={handleSave} disabled={saving}>
            {saving ? t('common.saving') || 'Saving...' : t('common.save')}
          </Button>
        </Box>
      </Stack>
    </Box>
  );
}
