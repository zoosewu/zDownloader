#!/usr/bin/env node
// Module Testing Tool (T158)
// Usage: node test-module.js <module-dir>

/// <reference types="zoo-downloader" />

import { readFileSync } from 'fs';
import { join } from 'path';

/**
 * Mock Context API for local testing
 */
class MockContext {
  constructor() {
    this.http = {
      get: async (url, options = {}) => {
        console.log(`[MOCK HTTP] GET ${url}`);
        return {
          status: 200,
          statusText: 'OK',
          headers: { 'content-type': 'text/html' },
          body: '<html>Mock response</html>',
          url
        };
      },
      post: async (url, data, options = {}) => {
        console.log(`[MOCK HTTP] POST ${url}`, data);
        return {
          status: 200,
          statusText: 'OK',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ success: true }),
          url
        };
      },
      put: async (url, data, options = {}) => {
        console.log(`[MOCK HTTP] PUT ${url}`, data);
        return { status: 200, statusText: 'OK', headers: {}, body: '{}', url };
      },
      delete: async (url, options = {}) => {
        console.log(`[MOCK HTTP] DELETE ${url}`);
        return { status: 200, statusText: 'OK', headers: {}, body: '', url };
      }
    };

    this.log = {
      debug: (...args) => console.log('[DEBUG]', ...args),
      info: (...args) => console.log('[INFO]', ...args),
      warn: (...args) => console.warn('[WARN]', ...args),
      error: (...args) => console.error('[ERROR]', ...args)
    };

    this.captcha = {
      requestCaptcha: async (type, params) => {
        console.log(`[MOCK CAPTCHA] Request ${type}`, params);
        return { id: 'mock-captcha-id', type, params };
      },
      waitForSolution: async (challengeId) => {
        console.log(`[MOCK CAPTCHA] Waiting for solution: ${challengeId}`);
        return { token: 'mock-solution-token', challengeId };
      }
    };

    this.credentials = {
      has: async (keys) => {
        console.log(`[MOCK CREDS] Checking keys:`, keys);
        return true;
      },
      get: async (keys) => {
        console.log(`[MOCK CREDS] Getting keys:`, keys);
        const creds = {};
        keys.forEach(key => creds[key] = `mock-${key}`);
        return creds;
      }
    };
  }
}

/**
 * Validate manifest.json
 */
function validateManifest(modulePath) {
  console.log('\n=== Validating Manifest ===');

  const manifestPath = join(modulePath, 'manifest.json');
  let manifest;

  try {
    const content = readFileSync(manifestPath, 'utf8');
    manifest = JSON.parse(content);
  } catch (error) {
    console.error('❌ Failed to read/parse manifest.json:', error.message);
    return false;
  }

  const requiredFields = ['id', 'name', 'version', 'urlPatterns'];
  let valid = true;

  for (const field of requiredFields) {
    if (!manifest[field]) {
      console.error(`❌ Missing required field: ${field}`);
      valid = false;
    } else {
      console.log(`✅ ${field}: ${JSON.stringify(manifest[field]).slice(0, 50)}`);
    }
  }

  if (manifest.permissions && manifest.permissions.network) {
    console.log(`✅ Network permissions: ${manifest.permissions.network.allowDomains?.length || 0} domains`);
  }

  return valid;
}

/**
 * Test module loading and basic functions
 */
async function testModule(modulePath) {
  console.log('\n=== Testing Module ===');

  const indexPath = join(modulePath, 'index.js');
  let ModuleClass;

  try {
    const module = await import(indexPath);
    const exports = Object.keys(module);
    console.log(`✅ Module exports:`, exports);

    // Find the main module class (usually the first export)
    const className = exports.find(key => key.endsWith('Module') || key === 'default');
    if (!className) {
      console.error('❌ No module class found (expecting *Module or default export)');
      return false;
    }

    ModuleClass = module[className] || module.default;
    console.log(`✅ Found module class: ${className}`);
  } catch (error) {
    console.error('❌ Failed to load module:', error.message);
    return false;
  }

  // Instantiate module
  let instance;
  try {
    instance = new ModuleClass();
    console.log('✅ Module instantiated successfully');
  } catch (error) {
    console.error('❌ Failed to instantiate module:', error.message);
    return false;
  }

  // Check canHandle method
  if (typeof instance.canHandle !== 'function') {
    console.error('❌ Missing canHandle() method');
    return false;
  }
  console.log('✅ canHandle() method exists');

  // Check extractDownloadInfo method
  if (typeof instance.extractDownloadInfo !== 'function') {
    console.error('❌ Missing extractDownloadInfo() method');
    return false;
  }
  console.log('✅ extractDownloadInfo() method exists');

  // Test with mock data
  console.log('\n=== Running Mock Tests ===');
  const mockContext = new MockContext();
  const testUrl = 'https://example.com/file/123';

  try {
    const canHandle = instance.canHandle(testUrl);
    console.log(`✅ canHandle('${testUrl}'): ${canHandle}`);
  } catch (error) {
    console.warn(`⚠️  canHandle() threw error:`, error.message);
  }

  try {
    console.log(`\nCalling extractDownloadInfo('${testUrl}')...`);
    const result = await instance.extractDownloadInfo(testUrl, mockContext);

    console.log('\n✅ extractDownloadInfo() result:');
    console.log('  downloadUrl:', result.downloadUrl || '(missing)');
    console.log('  filename:', result.filename || '(missing)');
    console.log('  filesize:', result.filesize || '(null)');
    console.log('  headers:', Object.keys(result.headers || {}).length, 'headers');

    if (!result.downloadUrl) {
      console.warn('⚠️  Warning: downloadUrl is missing');
    }
    if (!result.filename) {
      console.warn('⚠️  Warning: filename is missing');
    }

  } catch (error) {
    console.error('❌ extractDownloadInfo() failed:', error.message);
    console.error('   Stack:', error.stack);
    return false;
  }

  return true;
}

/**
 * Main test runner
 */
async function main() {
  const modulePath = process.argv[2];

  if (!modulePath) {
    console.error('Usage: node test-module.js <module-directory>');
    console.error('Example: node test-module.js modules/my-module');
    process.exit(1);
  }

  console.log(`Testing module at: ${modulePath}\n`);

  const manifestValid = validateManifest(modulePath);
  const moduleValid = await testModule(modulePath);

  console.log('\n=== Test Summary ===');
  console.log(`Manifest: ${manifestValid ? '✅ PASS' : '❌ FAIL'}`);
  console.log(`Module:   ${moduleValid ? '✅ PASS' : '❌ FAIL'}`);

  if (manifestValid && moduleValid) {
    console.log('\n🎉 All tests passed! Module is ready.');
    process.exit(0);
  } else {
    console.log('\n❌ Some tests failed. Please fix the issues above.');
    process.exit(1);
  }
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
