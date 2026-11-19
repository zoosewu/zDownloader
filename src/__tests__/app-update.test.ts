import { describe, expect, test } from 'vitest';
import { isPlatformNotReleased } from '../hooks/useAppUpdate';

describe('isPlatformNotReleased', () => {
  test("tauri-plugin-updater's error for a platform latest.json does not list yet", () => {
    // tauri-plugin-updater 2.9 Error::TargetNotFound
    expect(isPlatformNotReleased('the platform `darwin-x86_64` was not found on the response `platforms` object')).toBe(true);
  });

  test('other failures are still failures', () => {
    expect(isPlatformNotReleased('error sending request for url (https://github.com/...)')).toBe(false);
    expect(isPlatformNotReleased('signature verification failed')).toBe(false);
  });
});
