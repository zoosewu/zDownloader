// Malicious Test Module: sandbox_escape.js (T093)
//
// Purpose: Test sandbox escape prevention
// Expected: All attempts to escape sandbox should be blocked
//
// Security Test: SC-012 (Sandbox Escape Protection)

/**
 * Malicious Module: Attempts to escape JavaScript sandbox
 *
 * Attempts:
 * 1. Access to eval() and Function() constructor
 * 2. Prototype pollution attacks
 * 3. Access to process/require/import
 * 4. Access to Deno/Node.js APIs
 * 5. File system access attempts
 */
export class SandboxEscapeModule {
  constructor() {
    this.name = 'sandbox-escape-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting sandbox escape attempts');

    const attacks = [];

    // Attack 1: Try to use eval
    try {
      context.log.warn('[ATTACK] Attempting to use eval()');
      const result = eval('1 + 1');
      attacks.push({ attack: 'eval()', success: true, result });
      context.log.error('[ATTACK] SUCCESS - eval() is available! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: 'eval()', success: false, error: error.message });
      context.log.info('[EXPECTED] eval() is blocked');
    }

    // Attack 2: Try to use Function constructor
    try {
      context.log.warn('[ATTACK] Attempting to use Function() constructor');
      const fn = new Function('return 42');
      const result = fn();
      attacks.push({ attack: 'Function constructor', success: true, result });
      context.log.error('[ATTACK] SUCCESS - Function() is available! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: 'Function constructor', success: false, error: error.message });
      context.log.info('[EXPECTED] Function() constructor is blocked');
    }

    // Attack 3: Try to access Deno namespace
    try {
      context.log.warn('[ATTACK] Attempting to access Deno namespace');
      const denoVersion = Deno.version;
      attacks.push({ attack: 'Deno namespace', success: true, result: denoVersion });
      context.log.error('[ATTACK] SUCCESS - Deno namespace accessible! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: 'Deno namespace', success: false, error: error.message });
      context.log.info('[EXPECTED] Deno namespace is not accessible');
    }

    // Attack 4: Try to access process
    try {
      context.log.warn('[ATTACK] Attempting to access process object');
      const processEnv = process.env;
      attacks.push({ attack: 'process object', success: true, result: processEnv });
      context.log.error('[ATTACK] SUCCESS - process object accessible! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: 'process object', success: false, error: error.message });
      context.log.info('[EXPECTED] process object is not accessible');
    }

    // Attack 5: Try to modify Object.prototype
    try {
      context.log.warn('[ATTACK] Attempting prototype pollution');
      Object.prototype.polluted = 'hacked';
      const testObj = {};
      if (testObj.polluted === 'hacked') {
        attacks.push({ attack: 'Prototype pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - Prototype pollution worked! Sandbox broken!');
      } else {
        attacks.push({ attack: 'Prototype pollution', success: false });
        context.log.info('[EXPECTED] Prototype is frozen');
      }
    } catch (error) {
      attacks.push({ attack: 'Prototype pollution', success: false, error: error.message });
      context.log.info('[EXPECTED] Prototype modification blocked');
    }

    // Attack 6: Try to access __proto__
    try {
      context.log.warn('[ATTACK] Attempting __proto__ access');
      const obj = {};
      obj.__proto__.hack = 'pwned';
      attacks.push({ attack: '__proto__ access', success: true });
      context.log.error('[ATTACK] SUCCESS - __proto__ writable! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: '__proto__ access', success: false, error: error.message });
      context.log.info('[EXPECTED] __proto__ is protected');
    }

    // Attack 7: Try to access constructor.constructor (alternate eval)
    try {
      context.log.warn('[ATTACK] Attempting constructor.constructor escape');
      const fn = (''.constructor.constructor('return 42'))();
      attacks.push({ attack: 'constructor.constructor', success: true, result: fn });
      context.log.error('[ATTACK] SUCCESS - constructor.constructor escape! Sandbox broken!');
    } catch (error) {
      attacks.push({ attack: 'constructor.constructor', success: false, error: error.message });
      context.log.info('[EXPECTED] constructor.constructor blocked');
    }

    // Count results
    const successful = attacks.filter(a => a.success).length;
    const blocked = attacks.filter(a => !a.success).length;

    context.log.info(`[RESULTS] Sandbox escape test: ${blocked}/${attacks.length} blocked, ${successful} succeeded`);

    if (successful > 0) {
      throw new Error(`SECURITY FAILURE: ${successful} sandbox escape attempts succeeded!`);
    }

    throw new Error(`Security test passed: All ${blocked} sandbox escape attempts blocked`);
  }
}

export default SandboxEscapeModule;
