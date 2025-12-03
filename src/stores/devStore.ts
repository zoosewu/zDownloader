// Development mode store - controls mock data and download behavior
import { atom } from 'jotai';
import { atomWithStorage } from 'jotai/utils';

/**
 * Check if running in development mode
 */
export const isDevEnvironmentAtom = atom(() => {
  return import.meta.env.DEV;
});

/**
 * Enable/disable mock mode (stored in localStorage)
 * When enabled, downloads won't actually execute
 */
export const mockModeEnabledAtom = atomWithStorage('dev:mockMode', true);

/**
 * Enable/disable using mock download tasks (stored in localStorage)
 * When enabled, app will display mock tasks instead of real tasks
 */
export const useMockTasksAtom = atomWithStorage('dev:useMockTasks', false);

/**
 * Enable/disable auto-progress simulation for mock downloads
 * When enabled, mock downloads will automatically progress
 */
export const autoProgressEnabledAtom = atomWithStorage('dev:autoProgress', true);

/**
 * Derived atom: should show dev mode banner
 */
export const shouldShowDevBannerAtom = atom((get) => {
  const isDev = get(isDevEnvironmentAtom);
  const mockMode = get(mockModeEnabledAtom);
  const useMockTasks = get(useMockTasksAtom);

  return isDev && (mockMode || useMockTasks);
});

/**
 * Derived atom: dev mode status summary
 */
export const devModeStatusAtom = atom((get) => {
  const isDev = get(isDevEnvironmentAtom);
  const mockMode = get(mockModeEnabledAtom);
  const useMockTasks = get(useMockTasksAtom);
  const autoProgress = get(autoProgressEnabledAtom);

  return {
    isDev,
    mockMode,
    useMockTasks,
    autoProgress,
    isActive: isDev && (mockMode || useMockTasks),
  };
});
