/**
 * T137: Module Management UI Integration Test
 *
 * Tests the complete module management functionality:
 * 1. Module list loading
 * 2. Module scanning
 * 3. Module reloading
 * 4. Status display
 * 5. Error handling
 */

import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ModuleMetadata } from '../../types/module';

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
      return options?.defaultValue || key;
    },
    i18n: { language: 'zh-TW' },
  }),
}));

// Mock module API
vi.mock('../../api/modules', () => ({
  getAllModules: vi.fn(async () => []),
  scanModules: vi.fn(async () => ({ totalFound: 0, successfullyLoaded: 0, failed: [] })),
  reloadModule: vi.fn(async () => undefined),
}));

// Import after mocking
const { default: ModuleList } = await import('../../components/ModuleList');
const modulesApi = await import('../../api/modules');

describe('T137: Module Management UI Integration', () => {
  const mockModules: ModuleMetadata[] = [
    {
      id: 'k2s',
      name: 'Keep2Share',
      version: '1.0.0',
      description: 'Keep2Share file hosting service',
      author: 'Zoo Downloader',
      urlPatterns: ['https://k2s.cc/*', 'https://keep2share.cc/*'],
      permissions: {
        network: {
          allowDomains: ['k2s.cc', 'keep2share.cc', 'api.keep2share.cc'],
        },
        credentials: ['username', 'password'],
      },
      status: 'loaded' as const,
    },
    {
      id: 'example',
      name: 'Example Module',
      version: '0.1.0',
      description: 'Example module for testing',
      author: 'Test Author',
      urlPatterns: ['https://example.com/*'],
      permissions: {
        network: {
          allowDomains: ['example.com'],
        },
      },
      status: 'loaded' as const,
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('should load and display modules on mount', async () => {
    vi.mocked(modulesApi.getAllModules).mockResolvedValue(mockModules);

    render(<ModuleList />);

    // Should show loading state initially
    expect(screen.getByRole('progressbar')).toBeInTheDocument();

    // Wait for modules to load
    await waitFor(() => {
      expect(screen.getByText('Keep2Share')).toBeInTheDocument();
      expect(screen.getByText('Example Module')).toBeInTheDocument();
    });

    // Verify API was called
    expect(modulesApi.getAllModules).toHaveBeenCalledTimes(1);
  });

  test('should show empty state when no modules are found', async () => {
    vi.mocked(modulesApi.getAllModules).mockResolvedValue([]);

    render(<ModuleList />);

    await waitFor(() => {
      expect(screen.getByText(/沒有找到模組/i)).toBeInTheDocument();
    });
  });

  test('should scan for modules when scan button is clicked', async () => {
    const user = userEvent.setup();

    // Initial load: no modules
    vi.mocked(modulesApi.getAllModules).mockResolvedValueOnce([]);

    // After scan: modules found
    vi.mocked(modulesApi.scanModules).mockResolvedValue({
      totalFound: 2,
      successfullyLoaded: 2,
      failed: [],
    });
    vi.mocked(modulesApi.getAllModules).mockResolvedValueOnce(mockModules);

    render(<ModuleList />);

    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText(/沒有找到模組/i)).toBeInTheDocument();
    });

    // Click scan button
    const scanButton = screen.getByRole('button', {
      name: /掃描模組/i,
    });
    await user.click(scanButton);

    // Should show scanning state (button disabled)
    await waitFor(() => {
      expect(modulesApi.scanModules).toHaveBeenCalledTimes(1);
    });

    // Should show success message
    await waitFor(() => {
      expect(screen.getByText(/掃描完成/i)).toBeInTheDocument();
    });

    // Should reload modules after scan
    await waitFor(() => {
      expect(screen.getByText('Keep2Share')).toBeInTheDocument();
    });

    expect(modulesApi.getAllModules).toHaveBeenCalledTimes(2);
  });

  test('should handle scan errors gracefully', async () => {
    const user = userEvent.setup();

    vi.mocked(modulesApi.getAllModules).mockResolvedValue([]);
    vi.mocked(modulesApi.scanModules).mockRejectedValue(new Error('Permission denied'));

    render(<ModuleList />);

    await waitFor(() => {
      expect(screen.getByRole('button', {
        name: /掃描模組/i,
      })).toBeInTheDocument();
    });

    const scanButton = screen.getByRole('button', {
      name: /掃描模組/i,
    });
    await user.click(scanButton);

    // Should show error message
    await waitFor(() => {
      expect(screen.getByText(/掃描模組失敗/i)).toBeInTheDocument();
    });
  });

  test('should handle initial load errors', async () => {
    vi.mocked(modulesApi.getAllModules).mockRejectedValue(
      new Error('Database connection failed')
    );

    render(<ModuleList />);

    // Should show error message
    await waitFor(() => {
      expect(screen.getByText(/載入模組失敗/i)).toBeInTheDocument();
    });
  });

  test('should disable scan button while scanning', async () => {
    const user = userEvent.setup();

    vi.mocked(modulesApi.getAllModules).mockResolvedValue([]);
    vi.mocked(modulesApi.scanModules).mockImplementation(
      () =>
        new Promise(resolve =>
          setTimeout(() => resolve({ totalFound: 0, successfullyLoaded: 0, failed: [] }), 100)
        )
    );

    render(<ModuleList />);

    await waitFor(() => {
      expect(screen.getByRole('button', {
        name: /掃描模組/i,
      })).toBeInTheDocument();
    });

    const scanButton = screen.getByRole('button', {
      name: /掃描模組/i,
    });
    await user.click(scanButton);

    // Button should be disabled while scanning
    expect(scanButton).toBeDisabled();

    // Wait for scan to complete
    await waitFor(() => {
      expect(scanButton).not.toBeDisabled();
    });
  });

  test('should display module metadata correctly', async () => {
    vi.mocked(modulesApi.getAllModules).mockResolvedValue(mockModules);

    render(<ModuleList />);

    await waitFor(() => {
      // Check if module metadata is displayed
      expect(screen.getByText('Keep2Share')).toBeInTheDocument();
      expect(screen.getByText('Keep2Share file hosting service')).toBeInTheDocument();

      expect(screen.getByText('Example Module')).toBeInTheDocument();
      expect(screen.getByText('Example module for testing')).toBeInTheDocument();
    });
  });

  test('should refresh module list after successful scan', async () => {
    const user = userEvent.setup();

    vi.mocked(modulesApi.getAllModules).mockResolvedValueOnce([]);
    vi.mocked(modulesApi.scanModules).mockResolvedValue({
      totalFound: 2,
      successfullyLoaded: 2,
      failed: [],
    });
    vi.mocked(modulesApi.getAllModules).mockResolvedValueOnce(mockModules);

    render(<ModuleList />);

    await waitFor(() => {
      expect(screen.getByText(/沒有找到模組/i)).toBeInTheDocument();
    });

    const scanButton = screen.getByRole('button', {
      name: /掃描模組/i,
    });
    await user.click(scanButton);

    // After scan, modules should be visible
    await waitFor(() => {
      expect(screen.getByText('Keep2Share')).toBeInTheDocument();
      expect(screen.queryByText(/沒有找到模組/i)).not.toBeInTheDocument();
    });
  });
});
