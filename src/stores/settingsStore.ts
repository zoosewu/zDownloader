// Jotai atoms for global settings state
import { atom } from 'jotai';
import type { AppSettings } from '../backend';

// Default settings
const defaultSettings: AppSettings = {
  language: 'zh-TW',
  theme: 'dark',
  defaultDownloadDir: '',  // Will be set from backend
  maxConcurrentDownloads: null,  // unlimited
  segmentSizeMb: 10,
  threadCount: 8,
  speedLimitMode: 'unlimited',
  speedLimitKbps: null,
  enableProxy: false,
  proxyFallbackDirect: true,
  proxyVerificationIntervalHours: 24,
  enableClipboardMonitoring: true,
  clipboardAutoStart: false,
  enableNotifications: true,
  enableSystemTray: true,
  autoRetryEnabled: true,
  maxRetryCount: 3,
  retryIntervalSecs: 5,
  userAgent: null,
  checkModuleUpdates: true,
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
