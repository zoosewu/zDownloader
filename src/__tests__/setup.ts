/**
 * Vitest Setup File
 * Configures testing environment for React components
 */
import '@testing-library/jest-dom';
import { vi } from 'vitest';
import React from 'react';

// Mock Tauri API
const mockInvoke = vi.fn();
const mockEvent = {
  listen: vi.fn(),
  emit: vi.fn(),
};

vi.mock('../utils/tauri', () => ({
  invoke: mockInvoke,
  event: mockEvent,
}));

// Mock MUI Icons to avoid ESM import issues
vi.mock('@mui/icons-material/Refresh', () => ({
  default: (props: any) => React.createElement('svg', { ...props, 'data-testid': 'RefreshIcon' }),
}));

vi.mock('@mui/icons-material/Lock', () => ({
  default: (props: any) => React.createElement('svg', { ...props, 'data-testid': 'LockIcon' }),
}));

// Mock window.__TAURI_INTERNALS__
global.window = Object.create(window);
Object.defineProperty(window, '__TAURI_INTERNALS__', {
  value: {
    invoke: mockInvoke,
  },
  writable: true,
});

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
});

// Export mocks for use in tests
export { mockInvoke, mockEvent };
