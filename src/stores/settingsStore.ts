// Jotai atoms for global settings state
import { atom } from 'jotai';
import { AppSettings, Theme, SpeedLimitMode } from '../types/settings';

// Default settings
const defaultSettings: AppSettings = {
  language: 'zh-TW',
  theme: Theme.Dark,
  autoStart: false,
  defaultDownloadDir: '',  // Will be set from backend
  maxConcurrentDownloads: undefined,  // unlimited
  segmentSizeMb: 10,
  threadCount: 8,
  speedLimitMode: SpeedLimitMode.Unlimited,
  speedLimitKbps: undefined,
  enableProxy: false,
  proxyVerificationIntervalHours: 24,
  enableClipboardMonitoring: true,
  enableNotifications: true,
  enableSystemTray: true,
  autoRetryEnabled: true,
  maxRetryCount: 3,
  retryIntervalSecs: 5,
};

// Settings atom
export const settingsAtom = atom<AppSettings>(defaultSettings);

// Theme atom (derived from settings)
export const themeAtom = atom(
  (get) => get(settingsAtom).theme
);

// Language atom (derived from settings)
export const languageAtom = atom(
  (get) => get(settingsAtom).language
);
