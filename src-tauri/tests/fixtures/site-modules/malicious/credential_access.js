// Malicious Test Module: credential_access.js (T095)
//
// Purpose: Test credential isolation
// Expected: Module should never access raw credential values
//
// Security Test: SC-011 (Credential Isolation)

/**
 * Malicious Module: Attempts to access credentials directly
 *
 * Attempts:
 * 1. Read raw credential values from context.credentials
 * 2. Intercept credentials in HTTP headers
 * 3. Access OS keyring directly
 * 4. Extract credentials from error messages
 */
export class CredentialAccessModule {
  constructor() {
    this.name = 'credential-access-test';
    this.stolenCredentials = [];
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting credential access attempts');

    // Attack 1: Try to get raw credential value
    try {
      context.log.warn('[ATTACK] Attempting to read raw credential value');

      const username = context.credentials.get('username');
      const password = context.credentials.get('password');
      const apiKey = context.credentials.get('apiKey');

      // Check if we got actual credential values (not placeholders)
      if (username && !username.includes('[CREDENTIAL_HIDDEN]') && !username.includes('__CREDENTIAL:')) {
        this.stolenCredentials.push({ type: 'username', value: username });
        context.log.error(`[ATTACK] SUCCESS - Stole username: ${username}! Credential isolation broken!`);
      } else {
        context.log.info('[EXPECTED] Credential value is hidden/proxied');
      }

      if (password && !password.includes('[CREDENTIAL_HIDDEN]') && !password.includes('__CREDENTIAL:')) {
        this.stolenCredentials.push({ type: 'password', value: password });
        context.log.error(`[ATTACK] SUCCESS - Stole password: ${password}! Credential isolation broken!`);
      } else {
        context.log.info('[EXPECTED] Password value is hidden/proxied');
      }

      if (apiKey && !apiKey.includes('[CREDENTIAL_HIDDEN]') && !apiKey.includes('__CREDENTIAL:')) {
        this.stolenCredentials.push({ type: 'apiKey', value: apiKey });
        context.log.error(`[ATTACK] SUCCESS - Stole API key: ${apiKey}! Credential isolation broken!`);
      } else {
        context.log.info('[EXPECTED] API key value is hidden/proxied');
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Credential access prevented: ${error.message}`);
    }

    // Attack 2: Try to intercept credentials via monkey-patching
    try {
      context.log.warn('[ATTACK] Attempting to monkey-patch HTTP methods');

      // Try to replace context.http.post to intercept credentials
      const originalPost = context.http.post;
      context.http.post = async function (url, headers, body) {
        context.log.error(`[ATTACK] Intercepted POST to ${url} with body: ${body}`);

        // Try to extract credentials from body
        if (body && body.includes('password')) {
          context.log.error('[ATTACK] SUCCESS - Intercepted credentials in request body!');
        }

        return originalPost.call(this, url, headers, body);
      };

      context.log.error('[ATTACK] SUCCESS - Monkey-patched context.http! Can intercept credentials!');
    } catch (error) {
      context.log.info(`[EXPECTED] Monkey-patching prevented: ${error.message}`);
    }

    // Attack 3: Try to iterate all credentials
    try {
      context.log.warn('[ATTACK] Attempting to enumerate all credentials');

      const commonKeys = [
        'username', 'password', 'apiKey', 'token', 'auth',
        'api_key', 'access_token', 'refresh_token', 'session',
        'secret', 'private_key', 'api_secret'
      ];

      const foundCredentials = [];
      for (const key of commonKeys) {
        if (context.credentials.has(key)) {
          const value = context.credentials.get(key);
          foundCredentials.push({ key, value });
        }
      }

      if (foundCredentials.length > 0) {
        context.log.warn(`[ATTACK] Found ${foundCredentials.length} credential keys`);

        const exposedCreds = foundCredentials.filter(
          c => !c.value.includes('[CREDENTIAL_HIDDEN]') && !c.value.includes('__CREDENTIAL:')
        );

        if (exposedCreds.length > 0) {
          context.log.error(`[ATTACK] SUCCESS - ${exposedCreds.length} credentials exposed!`);
        } else {
          context.log.info('[EXPECTED] All credential values are properly hidden');
        }
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Credential enumeration prevented: ${error.message}`);
    }

    // Attack 4: Try to access credentials through toString/valueOf
    try {
      context.log.warn('[ATTACK] Attempting credential value extraction via toString()');

      const username = context.credentials.get('username');
      if (username) {
        const strValue = username.toString();
        const valueOf = username.valueOf();

        if (strValue !== '[object Object]' && !strValue.includes('[CREDENTIAL_HIDDEN]')) {
          context.log.error(`[ATTACK] SUCCESS - Extracted via toString(): ${strValue}`);
        } else {
          context.log.info('[EXPECTED] toString() does not expose credentials');
        }
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Value extraction prevented: ${error.message}`);
    }

    // Results
    if (this.stolenCredentials.length > 0) {
      throw new Error(`SECURITY FAILURE: Stole ${this.stolenCredentials.length} credentials!`);
    }

    throw new Error('Security test passed: Credentials properly isolated');
  }
}

export default CredentialAccessModule;
