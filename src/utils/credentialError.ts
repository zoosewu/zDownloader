// Credential Error Detection Utilities (T119)

/**
 * Check if an error message indicates a credential/authentication issue
 */
export function isCredentialError(errorMessage: string | undefined): boolean {
  if (!errorMessage) return false;

  const errorLower = errorMessage.toLowerCase();

  // Common authentication error patterns
  const patterns = [
    'auth',
    'credential',
    'login',
    'password',
    'unauthorized',
    '401',
    '403',
    'forbidden',
    'access denied',
    'invalid credentials',
    'authentication failed',
    'authentication required',
    'token expired',
    'session expired',
    'not authenticated',
    'please login',
    '認證',
    '密碼',
    '登入',
    '登录',
    '权限',
  ];

  return patterns.some((pattern) => errorLower.includes(pattern));
}

/**
 * Extract module ID from download task URL (if possible)
 * This is a best-effort attempt to identify which module handled the URL
 */
export function extractModuleIdFromUrl(url: string): string | null {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname.toLowerCase();

    // Map common hostnames to module IDs
    const hostnameToModuleId: Record<string, string> = {
      'k2s.cc': 'k2s',
      'keep2share.cc': 'k2s',
      // Add more mappings as modules are added
    };

    return hostnameToModuleId[hostname] || null;
  } catch {
    return null;
  }
}
