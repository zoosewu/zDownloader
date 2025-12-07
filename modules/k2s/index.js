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

    // K2S API login endpoint
    const loginUrl = `${this.apiBaseUrl}/login`;

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
   * T083: Detect account type (free/premium)
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
      const accountUrl = `${this.apiBaseUrl}/account/info`;

      const response = await context.http.get(accountUrl, {
        'Authorization': `Bearer ${this.session}`,
        'Content-Type': 'application/json'
      });

      const data = JSON.parse(response);

      if (data.status === 'success' && data.account) {
        this.accountType = data.account.is_premium ? 'premium' : 'free';
        context.log.info(`K2S account type detected: ${this.accountType}`);
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
      const fileInfoUrl = `${this.apiBaseUrl}/file/info`;

      const response = await context.http.post(
        fileInfoUrl,
        {
          'Authorization': `Bearer ${this.session}`,
          'Content-Type': 'application/json'
        },
        JSON.stringify({ file_id: fileId })
      );

      const data = JSON.parse(response);

      if (data.status !== 'success' || !data.file) {
        throw new Error(`File info request failed: ${data.message || 'Unknown error'}`);
      }

      return {
        id: data.file.id,
        name: data.file.name,
        size: data.file.size,
        isAvailable: data.file.is_available,
        access: data.file.access
      };
    } catch (error) {
      context.log.error(`Failed to get file info: ${error.message}`);
      throw error;
    }
  }

  /**
   * T082: Handle CAPTCHA requirement
   *
   * @param {string} captchaUrl - CAPTCHA image URL
   * @param {object} context - Module context
   * @returns {Promise<string>} CAPTCHA solution
   */
  async handleCaptcha(captchaUrl, context) {
    context.log.info('CAPTCHA required, requesting user input');

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
      const downloadUrl = `${this.apiBaseUrl}/file/download`;

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
        // Check if CAPTCHA is required
        if (data.code === 'captcha_required' && data.captcha_url) {
          context.log.info('CAPTCHA required for download');
          const captchaSolution = await this.handleCaptcha(data.captcha_url, context);

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
          access: fileInfo.access
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
