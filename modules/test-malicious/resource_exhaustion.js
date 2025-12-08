// Malicious Test Module: resource_exhaustion.js (T091)
//
// Purpose: Test resource limits enforcement (rate limiting, response size)
// Expected: Excessive requests and large responses should be blocked
//
// Security Test: SC-013 (Resource Limit Enforcement)

/**
 * Malicious Module: Attempts to exhaust system resources
 *
 * Attempts:
 * 1. Rate limit bypass (send > 60 requests/minute)
 * 2. Response size limit bypass (download > 10MB)
 * 3. Memory exhaustion (create large objects)
 */
export class ResourceExhaustionModule {
  constructor() {
    this.name = 'resource-exhaustion-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting resource exhaustion attacks');

    // Attack 1: Rate limit bypass
    try {
      context.log.warn('[ATTACK] Attempting to exceed rate limit (100 requests in quick succession)');

      const promises = [];
      for (let i = 0; i < 100; i++) {
        promises.push(
          context.http.get(`https://test.example.com/req${i}`, {})
            .catch(err => ({ error: err.message, index: i }))
        );
      }

      const results = await Promise.all(promises);
      const successful = results.filter(r => !r.error).length;

      if (successful > 60) {
        context.log.error(`[ATTACK] SUCCESS - Sent ${successful} requests! Rate limit bypassed!`);
      } else {
        context.log.info(`[EXPECTED] Rate limit enforced: only ${successful}/100 requests succeeded`);
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Rate limit enforcement: ${error.message}`);
    }

    // Attack 2: Response size limit bypass
    try {
      context.log.warn('[ATTACK] Attempting to download > 10MB response');

      // Try to download a large file
      const largeResponse = await context.http.get(
        'https://test.example.com/large-file?size=50mb',
        {}
      );

      if (largeResponse.length > 10 * 1024 * 1024) {
        context.log.error('[ATTACK] SUCCESS - Downloaded > 10MB! Response size limit bypassed!');
      } else {
        context.log.info('[EXPECTED] Response truncated or limited');
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Response size limit enforced: ${error.message}`);
    }

    // Attack 3: Memory exhaustion
    try {
      context.log.warn('[ATTACK] Attempting memory exhaustion');

      const hugeArray = new Array(100_000_000);
      for (let i = 0; i < hugeArray.length; i++) {
        hugeArray[i] = { data: new Array(1000).fill('x').join('') };
      }

      context.log.error('[ATTACK] SUCCESS - Allocated huge array! Memory limit bypassed!');
    } catch (error) {
      context.log.info(`[EXPECTED] Memory allocation prevented: ${error.message}`);
    }

    throw new Error('Resource exhaustion test completed');
  }
}

export default ResourceExhaustionModule;
