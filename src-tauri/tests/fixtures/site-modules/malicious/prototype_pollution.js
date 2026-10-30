// Malicious Test Module: prototype_pollution.js (T096)
//
// Purpose: Test prototype pollution protection
// Expected: All prototype modification attempts should be blocked
//
// Security Test: SC-012 (Sandbox Escape Protection - Prototype Pollution)

/**
 * Malicious Module: Attempts prototype pollution attacks
 *
 * Attempts various prototype pollution techniques:
 * 1. Direct Object.prototype modification
 * 2. __proto__ property assignment
 * 3. constructor.prototype manipulation
 * 4. Array.prototype pollution
 * 5. JSON parsing attacks
 */
export class PrototypePollutionModule {
  constructor() {
    this.name = 'prototype-pollution-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting prototype pollution attempts');

    const attacks = [];

    // Attack 1: Direct Object.prototype modification
    try {
      context.log.warn('[ATTACK] Attempting Object.prototype.polluted = "hacked"');

      Object.prototype.polluted = 'hacked';
      const testObj = {};

      if (testObj.polluted === 'hacked') {
        attacks.push({ attack: 'Object.prototype pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - Object.prototype is mutable! Security breach!');
      } else {
        attacks.push({ attack: 'Object.prototype pollution', success: false });
        context.log.info('[EXPECTED] Object.prototype is frozen');
      }
    } catch (error) {
      attacks.push({ attack: 'Object.prototype pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] Object.prototype modification blocked: ${error.message}`);
    }

    // Attack 2: __proto__ assignment
    try {
      context.log.warn('[ATTACK] Attempting __proto__ property assignment');

      const obj = {};
      obj.__proto__.injected = 'malicious';

      const anotherObj = {};
      if (anotherObj.injected === 'malicious') {
        attacks.push({ attack: '__proto__ pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - __proto__ is writable! Security breach!');
      } else {
        attacks.push({ attack: '__proto__ pollution', success: false });
        context.log.info('[EXPECTED] __proto__ is protected');
      }
    } catch (error) {
      attacks.push({ attack: '__proto__ pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] __proto__ assignment blocked: ${error.message}`);
    }

    // Attack 3: constructor.prototype pollution
    try {
      context.log.warn('[ATTACK] Attempting constructor.prototype pollution');

      const obj = {};
      obj.constructor.prototype.hacked = true;

      const testObj = {};
      if (testObj.hacked === true) {
        attacks.push({ attack: 'constructor.prototype pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - constructor.prototype is mutable! Security breach!');
      } else {
        attacks.push({ attack: 'constructor.prototype pollution', success: false });
        context.log.info('[EXPECTED] constructor.prototype is protected');
      }
    } catch (error) {
      attacks.push({ attack: 'constructor.prototype pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] constructor.prototype blocked: ${error.message}`);
    }

    // Attack 4: Array.prototype pollution
    try {
      context.log.warn('[ATTACK] Attempting Array.prototype pollution');

      Array.prototype.polluted = 'evil';
      const testArray = [];

      if (testArray.polluted === 'evil') {
        attacks.push({ attack: 'Array.prototype pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - Array.prototype is mutable! Security breach!');
      } else {
        attacks.push({ attack: 'Array.prototype pollution', success: false });
        context.log.info('[EXPECTED] Array.prototype is frozen');
      }
    } catch (error) {
      attacks.push({ attack: 'Array.prototype pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] Array.prototype modification blocked: ${error.message}`);
    }

    // Attack 5: Nested __proto__ pollution
    try {
      context.log.warn('[ATTACK] Attempting nested __proto__ pollution');

      const payload = JSON.parse('{"__proto__": {"polluted": true}}');
      const testObj = {};

      if (testObj.polluted === true) {
        attacks.push({ attack: 'JSON __proto__ pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - JSON parsing allows pollution! Security breach!');
      } else {
        attacks.push({ attack: 'JSON __proto__ pollution', success: false });
        context.log.info('[EXPECTED] JSON parsing is safe from pollution');
      }
    } catch (error) {
      attacks.push({ attack: 'JSON __proto__ pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] JSON pollution prevented: ${error.message}`);
    }

    // Attack 6: Deep property pollution
    try {
      context.log.warn('[ATTACK] Attempting deep property pollution via merge');

      const merge = (target, source) => {
        for (const key in source) {
          if (typeof source[key] === 'object' && source[key] !== null) {
            target[key] = merge(target[key] || {}, source[key]);
          } else {
            target[key] = source[key];
          }
        }
        return target;
      };

      const malicious = JSON.parse('{"__proto__": {"polluted": "deep"}}');
      const target = {};
      merge(target, malicious);

      const testObj = {};
      if (testObj.polluted === 'deep') {
        attacks.push({ attack: 'Deep merge pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - Deep merge allows pollution! Security breach!');
      } else {
        attacks.push({ attack: 'Deep merge pollution', success: false });
        context.log.info('[EXPECTED] Deep merge is safe');
      }
    } catch (error) {
      attacks.push({ attack: 'Deep merge pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] Deep merge pollution prevented: ${error.message}`);
    }

    // Attack 7: defineProperty pollution
    try {
      context.log.warn('[ATTACK] Attempting Object.defineProperty pollution');

      Object.defineProperty(Object.prototype, 'pwned', {
        get() { return 'hacked'; },
        configurable: true
      });

      const testObj = {};
      if (testObj.pwned === 'hacked') {
        attacks.push({ attack: 'defineProperty pollution', success: true });
        context.log.error('[ATTACK] SUCCESS - defineProperty works! Security breach!');
      } else {
        attacks.push({ attack: 'defineProperty pollution', success: false });
        context.log.info('[EXPECTED] defineProperty is blocked');
      }
    } catch (error) {
      attacks.push({ attack: 'defineProperty pollution', success: false, error: error.message });
      context.log.info(`[EXPECTED] defineProperty blocked: ${error.message}`);
    }

    // Count results
    const successful = attacks.filter(a => a.success).length;
    const blocked = attacks.filter(a => !a.success).length;

    context.log.info(`[RESULTS] Prototype pollution test: ${blocked}/${attacks.length} blocked, ${successful} succeeded`);

    if (successful > 0) {
      throw new Error(`SECURITY FAILURE: ${successful} prototype pollution attacks succeeded!`);
    }

    throw new Error(`Security test passed: All ${blocked} prototype pollution attempts blocked`);
  }
}

export default PrototypePollutionModule;
