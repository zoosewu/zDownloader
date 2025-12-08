// Module Template
// Replace this with your module implementation

/// <reference types="zoo-downloader" />

/**
 * @typedef {import('zoo-downloader').ModuleContext} ModuleContext
 * @typedef {import('zoo-downloader').DownloadInfo} DownloadInfo
 * @typedef {import('zoo-downloader').HttpResponse} HttpResponse
 */

/**
 * Your Site Module
 *
 * Replace "YourModule" with your site name (e.g., K2SModule, MediafireModule)
 * @implements {import('zoo-downloader').SiteModule}
 */
export class YourModule {
  constructor() {
    // Initialize your module state
    this.session = null;
  }

  /**
   * Check if this module can handle the given URL
   * @param {string} url - URL to check
   * @returns {boolean} Whether this module supports the URL
   */
  canHandle(url) {
    // Implement your URL matching logic
    // Example: return url.includes('example.com/file/');
    return url.includes('example.com/file/');
  }

  /**
   * Extract download information from URL
   * @param {string} url - File URL
   * @param {ModuleContext} context - Execution context provided by the downloader
   * @returns {Promise<DownloadInfo>} Download information
   */
  async extractDownloadInfo(url, context) {
    try {
      // 1. Make HTTP request to get the page
      const response = await context.http.get(url);

      // 2. Check for authentication requirement
      if (this.requiresAuth(response.body) && context.credentials) {
        await this.login(context);
        // Retry with authenticated session
        const authResponse = await context.http.get(url);
        return this.parseDownloadInfo(authResponse.body, url);
      }

      // 3. Parse download information
      return this.parseDownloadInfo(response.body, url);
    } catch (error) {
      throw new Error(`Failed to extract download info: ${error.message}`);
    }
  }

  /**
   * Check if authentication is required
   * @param {string} html - Page HTML
   * @returns {boolean} Whether auth is needed
   */
  requiresAuth(html) {
    // Implement your auth detection logic
    // Example: return html.includes('login-required');
    return false;
  }

  /**
   * Perform login
   * @param {ModuleContext} context - Execution context
   * @returns {Promise<void>}
   */
  async login(context) {
    // Implement your login logic using context.credentials
    const loginData = {
      username: context.credentials.username,
      password: context.credentials.password
    };

    const response = await context.http.post(
      'https://example.com/login',
      {
        body: JSON.stringify(loginData),
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );

    if (response.status !== 200) {
      throw new Error('Login failed');
    }
  }

  /**
   * Parse download information from HTML
   * @param {string} html - Page HTML
   * @param {string} originalUrl - Original URL for reference
   * @returns {object} Download info
   */
  parseDownloadInfo(html, originalUrl) {
    // Implement your parsing logic
    const downloadUrl = this.extractDownloadUrl(html);
    const filename = this.extractFilename(html);
    const filesize = this.extractFilesize(html);

    return {
      downloadUrl: downloadUrl,
      filename: filename,
      filesize: filesize,
      headers: {
        'Referer': originalUrl,
        'User-Agent': 'Zoo-Downloader/1.0'
      }
    };
  }

  /**
   * Extract download URL from HTML
   * @param {string} html - Page HTML
   * @returns {string} Download URL
   */
  extractDownloadUrl(html) {
    // Implement using regex or DOM parsing
    const match = html.match(/data-download="(.+?)"/);
    return match ? match[1] : null;
  }

  /**
   * Extract filename from HTML
   * @param {string} html - Page HTML
   * @returns {string} Filename
   */
  extractFilename(html) {
    // Implement your logic
    const match = html.match(/<h1>(.+?)<\/h1>/);
    return match ? match[1] : 'download';
  }

  /**
   * Extract file size from HTML
   * @param {string} html - Page HTML
   * @returns {number|null} File size in bytes
   */
  extractFilesize(html) {
    // Implement your logic
    const match = html.match(/File size: ([\d.]+) (MB|GB)/);
    if (!match) return null;

    const size = parseFloat(match[1]);
    const unit = match[2];
    return unit === 'GB' ? size * 1024 * 1024 * 1024 : size * 1024 * 1024;
  }
}

// Export the module class (required)
export default YourModule;
