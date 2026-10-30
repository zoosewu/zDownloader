// Malicious Test Module: ssrf_attempt.js (T092)
//
// Purpose: Test SSRF (Server-Side Request Forgery) protection
// Expected: Requests to internal/private IPs should be blocked
//
// Security Test: SC-014 (Internal Network Access Prevention)

/**
 * Malicious Module: Attempts SSRF attacks
 *
 * Attempts to access:
 * 1. Private IP ranges (10.x.x.x, 192.168.x.x, 172.16-31.x.x)
 * 2. Localhost (127.0.0.1, localhost, ::1)
 * 3. Link-local addresses (169.254.x.x)
 * 4. DNS rebinding attacks
 */
export class SSRFAttemptModule {
  constructor() {
    this.name = 'ssrf-attempt-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting SSRF attack attempts');

    const targets = [
      // Private IP ranges
      { url: 'http://10.0.0.1/admin', desc: 'Private IP 10.x.x.x' },
      { url: 'http://192.168.1.1/router', desc: 'Private IP 192.168.x.x' },
      { url: 'http://172.16.0.1/api', desc: 'Private IP 172.16-31.x.x' },

      // Localhost variants
      { url: 'http://127.0.0.1:8080/secrets', desc: 'Localhost (127.0.0.1)' },
      { url: 'http://localhost:3000/admin', desc: 'Localhost (domain name)' },
      { url: 'http://[::1]:8080/internal', desc: 'IPv6 localhost' },

      // Link-local
      { url: 'http://169.254.169.254/latest/meta-data/', desc: 'AWS metadata service' },

      // Alternative representations
      { url: 'http://0x7f000001/', desc: 'Localhost (hex representation)' },
      { url: 'http://2130706433/', desc: 'Localhost (decimal representation)' },
      { url: 'http://0177.0.0.1/', desc: 'Localhost (octal representation)' },

      // DNS tricks
      { url: 'http://localtest.me/', desc: 'DNS pointing to localhost' },
      { url: 'http://127.0.0.1.nip.io/', desc: 'DNS wildcard service' },
    ];

    let successful = 0;
    let blocked = 0;

    for (const target of targets) {
      try {
        context.log.warn(`[ATTACK] Attempting SSRF: ${target.desc} -> ${target.url}`);

        const response = await context.http.get(target.url, {});

        // If we reach here, SSRF protection failed
        successful++;
        context.log.error(`[ATTACK] SUCCESS - Accessed ${target.desc}! SSRF vulnerability!`);
      } catch (error) {
        // Expected: All SSRF attempts should be blocked
        blocked++;
        context.log.info(`[EXPECTED] SSRF blocked for ${target.desc}: ${error.message}`);
      }
    }

    context.log.info(`[RESULTS] SSRF test: ${blocked}/${targets.length} blocked, ${successful} succeeded`);

    if (successful > 0) {
      throw new Error(`SECURITY FAILURE: ${successful} SSRF attacks succeeded!`);
    }

    throw new Error(`Security test passed: All ${blocked} SSRF attempts blocked`);
  }
}

export default SSRFAttemptModule;
