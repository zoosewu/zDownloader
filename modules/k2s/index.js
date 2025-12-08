// Keep2Share (K2S) Module (Feature 004 - Phase 4.5)
//
// This module handles file downloads from Keep2Share (k2s.cc) and keep2share.cc.
// It supports both free and premium accounts with automatic authentication.

/**
 * K2S Module for handling Keep2Share downloads
 *
 * Phase 4.5 Implementation (T079-T084):
 * - T079: URL detection and validation
 * - T080: Authentication flow (login with credentials)
 * - T081: URL parsing and file info extraction
 * - T082: CAPTCHA handling integration
 * - T083: Account type detection (free/premium)
 */
export class K2SModule {
  constructor() {
    this.session = null;
    this.accountType = null; // 'free', 'premium', or null
    this.apiBaseUrl = 'https://api.keep2share.cc/v2';

    // T168: Premium account expiry tracking
    this.premiumExpired = false; // True if premium has expired
    this.premiumUntil = null;    // Premium expiry date (ISO string)

    // T165: API Endpoint Configuration
    // Supports both current endpoints and official API v2 endpoints
    // Switch to 'official' if current endpoints fail
    this.endpointMode = 'current'; // 'current' or 'official'

    this.endpoints = {
      // Current implementation (may be custom/older API)
      current: {
        login: '/login',
        fileInfo: '/file/info',
        download: '/file/download',
        accountInfo: '/account/info',
      },
      // Official API v2 endpoints (from K2S documentation)
      official: {
        login: '/login',
        fileInfo: '/getFilesInfo',
        download: '/getUrl',
        accountInfo: '/accountInfo',
        requestCaptcha: '/requestReCaptcha',  // T166
        verifyCaptcha: '/reCaptcha',          // T166
      },
    };

    // T166: ReCAPTCHA configuration
    this.captchaConfig = {
      useOfficialFlow: true,  // Try official ReCAPTCHA v2 flow first
      fallbackToLegacy: true,  // Fall back to legacy captcha_required if official fails
    };
  }

  /**
   * T165: Get current API endpoint
   *
   * @param {string} type - Endpoint type (login, fileInfo, download, accountInfo)
   * @returns {string} Full endpoint URL
   */
  getEndpoint(type) {
    const path = this.endpoints[this.endpointMode][type];
    return `${this.apiBaseUrl}${path}`;
  }

  /**
   * T165: Switch to official API endpoints
   *
   * Call this method if current endpoints fail
   */
  switchToOfficialEndpoints() {
    this.endpointMode = 'official';
    console.log('[K2S] Switched to official API endpoints');
  }

  /**
   * T165: Prepare API request payload based on endpoint mode
   *
   * Official API uses different parameter names
   * @param {string} type - Request type (fileInfo, download)
   * @param {object} params - Parameters
   * @returns {object} Formatted payload
   */
  preparePayload(type, params) {
    if (this.endpointMode === 'official') {
      switch (type) {
        case 'fileInfo':
          // Official API: /getFilesInfo expects { ids: ["file_id1", "file_id2"] }
          return { ids: [params.file_id] };

        case 'download':
          // Official API: /getUrl may have different parameters
          // Keep current format for now, update if needed
          return params;

        default:
          return params;
      }
    }
    // Current mode: use parameters as-is
    return params;
  }

  /**
   * T079: Check if this module can handle the given URL
   *
   * @param {string} url - URL to check
   * @returns {boolean} Whether this module supports the URL
   */
  canHandle(url) {
    try {
      const urlObj = new URL(url);
      const hostname = urlObj.hostname.toLowerCase();
      const pathname = urlObj.pathname.toLowerCase();

      // Match k2s.cc or keep2share.cc domains
      const validDomains = [
        'k2s.cc',
        'keep2share.cc',
        'www.k2s.cc',
        'www.keep2share.cc'
      ];

      const isDomainValid = validDomains.some(domain => hostname === domain || hostname.endsWith('.' + domain));
      const isPathValid = pathname.startsWith('/file/');

      return isDomainValid && isPathValid;
    } catch (error) {
      context.log.warn(`Invalid URL in canHandle: ${error.message}`);
      return false;
    }
  }

  /**
   * T080: Authenticate with K2S using credentials
   *
   * @param {object} context - Module context with credentials API
   * @returns {Promise<string>} Session token
   * @throws {Error} If authentication fails
   */
  async authenticate(context) {
    context.log.info('Starting K2S authentication');

    // Check credentials availability
    if (!context.credentials.has('username') || !context.credentials.has('password')) {
      throw new Error('K2S requires username and password credentials');
    }

    // K2S API login endpoint (T165: using configurable endpoint)
    const loginUrl = this.getEndpoint('login');

    try {
      // Prepare login request
      // Note: Actual credentials are injected by CredentialProxy, we just pass markers
      const loginPayload = JSON.stringify({
        username: '__CREDENTIAL:username__',
        password: '__CREDENTIAL:password__'
      });

      // Make authenticated request (CredentialProxy handles credential injection)
      const response = await context.http.post(
        loginUrl,
        { 'Content-Type': 'application/json' },
        loginPayload
      );

      const data = JSON.parse(response);

      if (data.status !== 'success' || !data.auth_token) {
        throw new Error(`Authentication failed: ${data.message || 'Unknown error'}`);
      }

      this.session = data.auth_token;
      context.log.info('K2S authentication successful');

      return this.session;
    } catch (error) {
      context.log.error(`K2S authentication error: ${error.message}`);
      throw new Error(`Failed to authenticate with K2S: ${error.message}`);
    }
  }

  /**
   * T083, T168: Detect account type (free/premium) with expiry handling
   *
   * T168: Now handles expired premium accounts and logs expiry information
   *
   * @param {object} context - Module context
   * @returns {Promise<string>} Account type: 'free' or 'premium'
   */
  async detectAccountType(context) {
    if (this.accountType) {
      return this.accountType;
    }

    context.log.info('Detecting K2S account type');

    try {
      const accountUrl = this.getEndpoint('accountInfo'); // T165

      const response = await context.http.get(accountUrl, {
        'Authorization': `Bearer ${this.session}`,
        'Content-Type': 'application/json'
      });

      const data = JSON.parse(response);

      if (data.status === 'success' && data.account) {
        const account = data.account;

        // T168: Check for premium_expired flag
        if (account.premium_expired === true) {
          this.accountType = 'free';

          // Log premium expiry information
          if (account.premium_until) {
            context.log.warn(
              `K2S premium account has expired (expired on: ${account.premium_until}). ` +
              `Account downgraded to free tier.`
            );
          } else {
            context.log.warn(
              'K2S premium account has expired. Account downgraded to free tier.'
            );
          }

          // Store expiry info for reference
          this.premiumExpired = true;
          this.premiumUntil = account.premium_until;

        } else if (account.is_premium) {
          // Active premium account
          this.accountType = 'premium';

          // Log premium expiry date if available
          if (account.premium_until) {
            context.log.info(
              `K2S premium account active (valid until: ${account.premium_until})`
            );
            this.premiumUntil = account.premium_until;
          } else {
            context.log.info('K2S premium account active');
          }

          this.premiumExpired = false;

        } else {
          // Free account
          this.accountType = 'free';
          context.log.info('K2S free account detected');
          this.premiumExpired = false;
        }

      } else {
        this.accountType = 'free';
        context.log.warn('Could not detect account type, assuming free account');
      }

      return this.accountType;
    } catch (error) {
      context.log.warn(`Account type detection failed: ${error.message}, assuming free`);
      this.accountType = 'free';
      return this.accountType;
    }
  }

  /**
   * T081: Parse file ID from K2S URL
   *
   * @param {string} url - K2S file URL
   * @returns {string} File ID
   * @throws {Error} If URL format is invalid
   */
  parseFileId(url) {
    try {
      const urlObj = new URL(url);
      const pathParts = urlObj.pathname.split('/');

      // URL format: https://k2s.cc/file/abcd1234/filename.ext
      // or: https://k2s.cc/file/abcd1234
      const fileIndex = pathParts.indexOf('file');

      if (fileIndex === -1 || fileIndex + 1 >= pathParts.length) {
        throw new Error('Invalid K2S URL format: missing file ID');
      }

      const fileId = pathParts[fileIndex + 1];

      if (!fileId || fileId.length < 8) {
        throw new Error('Invalid K2S file ID');
      }

      return fileId;
    } catch (error) {
      throw new Error(`Failed to parse K2S URL: ${error.message}`);
    }
  }

  /**
   * T081: Get file information from K2S API
   *
   * @param {string} fileId - File ID
   * @param {object} context - Module context
   * @returns {Promise<object>} File information
   */
  async getFileInfo(fileId, context) {
    context.log.info(`Fetching file info for: ${fileId}`);

    try {
      const fileInfoUrl = this.getEndpoint('fileInfo'); // T165

      // T165: Prepare payload based on endpoint mode
      const payload = this.preparePayload('fileInfo', { file_id: fileId });

      const response = await context.http.post(
        fileInfoUrl,
        {
          'Authorization': `Bearer ${this.session}`,
          'Content-Type': 'application/json'
        },
        JSON.stringify(payload)
      );

      const data = JSON.parse(response);

      if (data.status !== 'success') {
        throw new Error(`File info request failed: ${data.message || 'Unknown error'}`);
      }

      // T165: Handle different response formats
      let fileData;
      if (this.endpointMode === 'official' && data.files && data.files.length > 0) {
        // Official API: /getFilesInfo returns { files: [{ ... }] }
        fileData = data.files[0];
      } else if (data.file) {
        // Current API: returns { file: { ... } }
        fileData = data.file;
      } else {
        throw new Error('File info not found in response');
      }

      return {
        id: fileData.id,
        name: fileData.name,
        size: fileData.size,
        isAvailable: fileData.is_available,
        access: fileData.access
      };
    } catch (error) {
      context.log.error(`Failed to get file info: ${error.message}`);
      throw error;
    }
  }

  /**
   * T166: Handle CAPTCHA requirement using official ReCAPTCHA v2 flow
   *
   * Official flow:
   * 1. Request ReCAPTCHA challenge (POST /requestReCaptcha)
   * 2. Display ReCAPTCHA to user with siteKey
   * 3. Submit challenge + response (POST /reCaptcha)
   * 4. Retry original request
   *
   * @param {string} fileId - File ID requiring CAPTCHA
   * @param {object} context - Module context
   * @returns {Promise<void>} Resolves when CAPTCHA is verified
   */
  async handleOfficialReCaptcha(fileId, context) {
    context.log.info('[T166] Using official ReCAPTCHA v2 flow');

    try {
      // Step 1: Request ReCAPTCHA challenge
      const requestCaptchaUrl = this.getEndpoint('requestCaptcha');

      const challengeResponse = await context.http.post(
        requestCaptchaUrl,
        {
          'Authorization': `Bearer ${this.session}`,
          'Content-Type': 'application/json'
        },
        JSON.stringify({ file_id: fileId })
      );

      const challengeData = JSON.parse(challengeResponse);

      if (!challengeData.siteKey || !challengeData.challenge) {
        throw new Error('Invalid ReCAPTCHA challenge response');
      }

      context.log.info(`[T166] ReCAPTCHA challenge received: siteKey=${challengeData.siteKey}`);

      // Step 2: Request user to solve CAPTCHA
      // T166: Updated captcha.request() to support siteKey
      const solution = await context.captcha.request(null, challengeData.siteKey);

      context.log.info('[T166] User solved ReCAPTCHA, verifying...');

      // Step 3: Submit challenge + response
      const verifyCaptchaUrl = this.getEndpoint('verifyCaptcha');

      const verifyResponse = await context.http.post(
        verifyCaptchaUrl,
        {
          'Authorization': `Bearer ${this.session}`,
          'Content-Type': 'application/json'
        },
        JSON.stringify({
          re_captcha_challenge: challengeData.challenge,
          re_captcha_response: solution
        })
      );

      const verifyData = JSON.parse(verifyResponse);

      if (verifyData.status !== 'success') {
        throw new Error(`ReCAPTCHA verification failed: ${verifyData.message || 'Unknown error'}`);
      }

      context.log.info('[T166] ReCAPTCHA verified successfully');
    } catch (error) {
      context.log.error(`[T166] Official ReCAPTCHA flow failed: ${error.message}`);
      throw error;
    }
  }

  /**
   * T166: Handle CAPTCHA requirement (Legacy flow - fallback)
   *
   * @param {string} captchaUrl - CAPTCHA image URL
   * @param {object} context - Module context
   * @returns {Promise<string>} CAPTCHA solution
   */
  async handleLegacyCaptcha(captchaUrl, context) {
    context.log.info('[T166] Using legacy CAPTCHA flow (fallback)');

    try {
      // Request CAPTCHA solution from user via Tauri event
      const solution = await context.captcha.request(captchaUrl, null);
      return solution;
    } catch (error) {
      // If captcha handling not implemented yet, provide helpful error
      if (error.message && error.message.includes('not yet implemented')) {
        throw new Error('CAPTCHA required but handler not yet available. This will be implemented in Phase 4.5+');
      }
      throw error;
    }
  }

  /**
   * T081: Get direct download link
   *
   * @param {string} fileId - File ID
   * @param {object} context - Module context
   * @returns {Promise<object>} Download link information
   */
  async getDownloadLink(fileId, context) {
    context.log.info(`Requesting download link for: ${fileId}`);

    try {
      const downloadUrl = this.getEndpoint('download'); // T165

      const payload = JSON.stringify({
        file_id: fileId,
        free_download: this.accountType === 'free' ? 1 : 0
      });

      const response = await context.http.post(
        downloadUrl,
        {
          'Authorization': `Bearer ${this.session}`,
          'Content-Type': 'application/json'
        },
        payload
      );

      const data = JSON.parse(response);

      if (data.status !== 'success') {
        // T166: Check for official ReCAPTCHA v2 error (HTTP 406 + errorCode 33)
        if (this.captchaConfig.useOfficialFlow &&
            (data.code === 406 || data.code === '406') &&
            data.errorCode === 33) {
          context.log.info('[T166] Official ReCAPTCHA v2 required (error 406/33)');

          try {
            // Use official ReCAPTCHA flow
            await this.handleOfficialReCaptcha(fileId, context);

            // Retry original request after CAPTCHA verification
            const retryResponse = await context.http.post(
              downloadUrl,
              {
                'Authorization': `Bearer ${this.session}`,
                'Content-Type': 'application/json'
              },
              payload
            );

            const retryData = JSON.parse(retryResponse);

            if (retryData.status !== 'success' || !retryData.url) {
              throw new Error(`Download link request failed after ReCAPTCHA: ${retryData.message || 'Unknown error'}`);
            }

            return {
              url: retryData.url,
              waitTime: retryData.wait_time || 0
            };
          } catch (officialError) {
            // T166: Official flow failed, try fallback if enabled
            if (this.captchaConfig.fallbackToLegacy) {
              context.log.warn('[T166] Official ReCAPTCHA failed, trying legacy flow');
              // Continue to legacy flow below
            } else {
              throw officialError;
            }
          }
        }

        // T166: Legacy CAPTCHA flow (fallback)
        if (data.code === 'captcha_required' && data.captcha_url) {
          context.log.info('[T166] Legacy CAPTCHA required');
          const captchaSolution = await this.handleLegacyCaptcha(data.captcha_url, context);

          // Retry with CAPTCHA solution
          const retryPayload = JSON.stringify({
            file_id: fileId,
            free_download: this.accountType === 'free' ? 1 : 0,
            captcha_response: captchaSolution
          });

          const retryResponse = await context.http.post(
            downloadUrl,
            {
              'Authorization': `Bearer ${this.session}`,
              'Content-Type': 'application/json'
            },
            retryPayload
          );

          const retryData = JSON.parse(retryResponse);

          if (retryData.status !== 'success' || !retryData.url) {
            throw new Error(`Download link request failed after CAPTCHA: ${retryData.message || 'Unknown error'}`);
          }

          return {
            url: retryData.url,
            waitTime: retryData.wait_time || 0
          };
        }

        throw new Error(`Download link request failed: ${data.message || 'Unknown error'}`);
      }

      return {
        url: data.url,
        waitTime: data.wait_time || 0
      };
    } catch (error) {
      context.log.error(`Failed to get download link: ${error.message}`);
      throw error;
    }
  }

  /**
   * Extract download information from K2S URL (Main entry point)
   *
   * @param {string} url - K2S file URL
   * @param {object} context - Execution context provided by the main app
   * @returns {Promise<object>} Download information
   */
  async extractDownloadInfo(url, context) {
    context.log.info(`K2S module processing URL: ${url}`);

    try {
      // Step 1: Parse file ID from URL (T081)
      const fileId = this.parseFileId(url);
      context.log.info(`Extracted file ID: ${fileId}`);

      // Step 2: Authenticate if not already authenticated (T080)
      if (!this.session) {
        await this.authenticate(context);
      }

      // Step 3: Detect account type (T083)
      await this.detectAccountType(context);

      // Step 4: Get file information (T081)
      const fileInfo = await this.getFileInfo(fileId, context);
      context.log.info(`File info: ${fileInfo.name} (${fileInfo.size} bytes)`);

      if (!fileInfo.isAvailable) {
        throw new Error('File is not available (may be deleted or suspended)');
      }

      // Step 5: Get direct download link (T081 + T082)
      const downloadLink = await this.getDownloadLink(fileId, context);

      // Step 6: Handle wait time for free accounts
      if (downloadLink.waitTime > 0) {
        context.log.info(`Free account wait time: ${downloadLink.waitTime} seconds`);
      }

      // Step 7: Return download information
      return {
        downloadUrl: downloadLink.url,
        filename: fileInfo.name,
        filesize: fileInfo.size,
        waitTime: downloadLink.waitTime,
        accountType: this.accountType,
        headers: {
          'Referer': url,
          'User-Agent': 'Zoo-Downloader/1.0'
        },
        metadata: {
          fileId: fileId,
          originalUrl: url,
          access: fileInfo.access,
          // T168: Include premium expiry information
          premiumExpired: this.premiumExpired,
          premiumUntil: this.premiumUntil
        },
        // T169: Download optimization hints for parallel downloads
        downloadHints: {
          // Optimal chunk size for range requests (20 MB)
          chunkSize: 20 * 1024 * 1024,
          // Maximum parallel connections based on account type
          maxConnections: this.accountType === 'premium' ? 20 : 5,
          // Whether the server supports range requests
          supportsRangeRequests: true,
          // Recommended delay between chunk requests (ms)
          requestDelay: this.accountType === 'premium' ? 0 : 100
        }
      };
    } catch (error) {
      context.log.error(`K2S extraction failed: ${error.message}`);
      throw new Error(`Failed to extract K2S download info: ${error.message}`);
    }
  }
}

// Export the module class (required)
export default K2SModule;
