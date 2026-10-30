// Settings-related TypeScript types

export interface AppSettings {
  // General settings
  language: string;  // "zh-TW" | "en" | "ja"
  theme: Theme;
  autoStart: boolean;

  // Download settings
  defaultDownloadDir: string;
  maxConcurrentDownloads?: number;  // undefined = unlimited
  segmentSizeMb: number;  // default 10MB
  threadCount: number;   // default 8
  speedLimitMode: SpeedLimitMode;
  speedLimitKbps?: number;

  // Proxy settings
  enableProxy: boolean;
  proxyVerificationIntervalHours: number;  // default 24

  // UI settings
  enableClipboardMonitoring: boolean;
  enableNotifications: boolean;
  enableSystemTray: boolean;

  // Advanced settings
  autoRetryEnabled: boolean;
  maxRetryCount: number;
  retryIntervalSecs: number;
}

// Values must match Rust's #[serde(rename_all = "camelCase")] output
export enum Theme {
  Light = 'light',
  Dark = 'dark',
}

export enum SpeedLimitMode {
  Unlimited = 'unlimited',
  Fixed = 'fixed',
}
