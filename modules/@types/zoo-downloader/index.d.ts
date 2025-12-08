// Type definitions for Zoo Downloader Site Module Development
// Project: https://github.com/zoo-downloader/zoo-downloader
// Definitions by: Zoo Downloader Community
// TypeScript Version: 5.0+

/**
 * # Zoo Downloader Module Type Definitions
 *
 * This file provides TypeScript type definitions for developing Zoo Downloader site modules.
 * Modules are JavaScript files that extend the downloader to support specific websites.
 *
 * ## Quick Start
 *
 * Add this to the top of your module file:
 * ```javascript
 * /// <reference types="zoo-downloader" />
 * ```
 *
 * Then use JSDoc annotations for IDE support:
 * ```javascript
 * /** @type {import('zoo-downloader').SiteModule} *\/
 * export class MySiteModule {
 *   canHandle(url) { return true; }
 *   async extractDownloadInfo(url, context) {
 *     const response = await context.http.get(url);
 *     return {
 *       downloadUrl: '...',
 *       filename: '...',
 *       filesize: null,
 *       headers: {}
 *     };
 *   }
 * }
 * ```
 *
 * @module zoo-downloader
 */

// ============================================================================
// Core Module Interface
// ============================================================================

/**
 * Site module interface
 *
 * All modules must implement this interface to integrate with Zoo Downloader.
 *
 * @example
 * ```javascript
 * export class K2SModule {
 *   canHandle(url) {
 *     return url.includes('k2s.cc/file/');
 *   }
 *
 *   async extractDownloadInfo(url, context) {
 *     const response = await context.http.get(url);
 *     return {
 *       downloadUrl: this.parseDownloadUrl(response.body),
 *       filename: this.parseFilename(response.body),
 *       filesize: null,
 *       headers: { 'Referer': url }
 *     };
 *   }
 * }
 * ```
 */
export interface SiteModule {
  /**
   * Check if this module can handle the given URL
   *
   * @param url - The URL to check
   * @returns `true` if this module supports the URL, `false` otherwise
   *
   * @example
   * ```javascript
   * canHandle(url) {
   *   return url.includes('k2s.cc') || url.includes('keep2share.cc');
   * }
   * ```
   */
  canHandle(url: string): boolean;

  /**
   * Extract download information from the URL
   *
   * This is the main function that parses the share link and returns
   * the actual download URL and file metadata.
   *
   * @param url - The share link URL
   * @param context - Execution context provided by the downloader
   * @returns Download information
   * @throws {Error} If extraction fails
   *
   * @example
   * ```javascript
   * async extractDownloadInfo(url, context) {
   *   // Login if credentials are available
   *   if (context.credentials) {
   *     await this.login(context);
   *   }
   *
   *   // Fetch the page
   *   const response = await context.http.get(url);
   *
   *   // Parse download URL
   *   const downloadUrl = this.parseDownloadUrl(response.body);
   *   if (!downloadUrl) {
   *     throw new Error('Failed to parse download URL');
   *   }
   *
   *   return {
   *     downloadUrl,
   *     filename: this.parseFilename(response.body),
   *     filesize: this.parseFilesize(response.body),
   *     headers: { 'Referer': url }
   *   };
   * }
   * ```
   */
  extractDownloadInfo(
    url: string,
    context: ModuleContext
  ): Promise<DownloadInfo>;
}

// ============================================================================
// Module Context API
// ============================================================================

/**
 * Module execution context
 *
 * Provided by the downloader to give modules controlled access to:
 * - HTTP requests (within allowed domains)
 * - Credentials (securely proxied)
 * - CAPTCHA solving
 * - Logging
 *
 * @example
 * ```javascript
 * async extractDownloadInfo(url, context) {
 *   // HTTP request
 *   const response = await context.http.get(url);
 *
 *   // Logging
 *   context.log.info('Processing URL:', url);
 *
 *   // Credentials (if available)
 *   if (context.credentials) {
 *     const username = context.credentials.username;
 *   }
 *
 *   // CAPTCHA (if needed)
 *   if (needsCaptcha(response.body)) {
 *     const solution = await context.captcha.request({ ... });
 *   }
 * }
 * ```
 */
export interface ModuleContext {
  /** HTTP client (domain-restricted) */
  http: HttpClient;

  /** Credential proxy (if module requires authentication) */
  credentials?: Credentials;

  /** CAPTCHA solver */
  captcha: CaptchaClient;

  /** Logging interface */
  log: Logger;
}

// ============================================================================
// HTTP Client
// ============================================================================

/**
 * HTTP client for making web requests
 *
 * **Important:** Can only access domains listed in your module's manifest.
 *
 * @example
 * ```javascript
 * // GET request
 * const response = await context.http.get('https://k2s.cc/file/abc123');
 *
 * // POST request with JSON
 * const response = await context.http.post('https://k2s.cc/api/login', {
 *   body: JSON.stringify({ username: 'user', password: 'pass' }),
 *   headers: { 'Content-Type': 'application/json' }
 * });
 *
 * // With custom headers
 * const response = await context.http.get(url, {
 *   headers: { 'User-Agent': 'MyDownloader/1.0' }
 * });
 * ```
 */
export interface HttpClient {
  /**
   * Send GET request
   *
   * @param url - Target URL (must be in allowed domains)
   * @param options - Request options
   * @returns HTTP response
   * @throws {Error} If domain is not allowed or request fails
   */
  get(url: string, options?: HttpRequestOptions): Promise<HttpResponse>;

  /**
   * Send POST request
   *
   * @param url - Target URL (must be in allowed domains)
   * @param options - Request options
   * @returns HTTP response
   * @throws {Error} If domain is not allowed or request fails
   */
  post(url: string, options?: HttpRequestOptions): Promise<HttpResponse>;

  /**
   * Send PUT request
   *
   * @param url - Target URL (must be in allowed domains)
   * @param options - Request options
   * @returns HTTP response
   * @throws {Error} If domain is not allowed or request fails
   */
  put(url: string, options?: HttpRequestOptions): Promise<HttpResponse>;

  /**
   * Send DELETE request
   *
   * @param url - Target URL (must be in allowed domains)
   * @param options - Request options
   * @returns HTTP response
   * @throws {Error} If domain is not allowed or request fails
   */
  delete(url: string, options?: HttpRequestOptions): Promise<HttpResponse>;
}

/**
 * HTTP request options
 */
export interface HttpRequestOptions {
  /** Request headers */
  headers?: Record<string, string>;

  /** Request body (for POST/PUT) */
  body?: string;

  /** Follow redirects (default: true) */
  followRedirects?: boolean;

  /** Maximum redirects to follow (default: 5) */
  maxRedirects?: number;

  /** Request timeout in milliseconds (default: 30000) */
  timeout?: number;
}

/**
 * HTTP response
 *
 * @example
 * ```javascript
 * const response = await context.http.get(url);
 *
 * if (response.status === 200) {
 *   const html = response.body;
 *   const downloadUrl = parseDownloadUrl(html);
 * } else if (response.status === 404) {
 *   throw new Error('File not found');
 * }
 * ```
 */
export interface HttpResponse {
  /** HTTP status code (200, 404, etc.) */
  status: number;

  /** Response body as string */
  body: string;

  /** Response headers */
  headers: Record<string, string>;

  /** Cookies from response */
  cookies: Record<string, string>;
}

// ============================================================================
// Credentials
// ============================================================================

/**
 * Credential proxy
 *
 * **Security:** Modules never see the actual password.
 * Credentials are securely stored in the OS keychain and only accessible
 * through authenticated HTTP requests.
 *
 * @example
 * ```javascript
 * if (context.credentials) {
 *   // Access credential keys (not values!)
 *   const hasUsername = 'username' in context.credentials;
 *   const hasApiKey = 'apiKey' in context.credentials;
 *
 *   // Credentials are automatically included in HTTP requests
 *   // via context.http (no manual handling needed)
 * }
 * ```
 */
export interface Credentials {
  /** Credential keys available (values are hidden) */
  [key: string]: string;
}

// ============================================================================
// CAPTCHA Client
// ============================================================================

/**
 * CAPTCHA solving client
 *
 * Requests user to solve CAPTCHA challenges.
 *
 * @example
 * ```javascript
 * // Image CAPTCHA
 * const solution = await context.captcha.request({
 *   type: 'image',
 *   imageUrl: 'https://example.com/captcha.png'
 * });
 * console.log('User entered:', solution.solution);
 *
 * // reCAPTCHA
 * const solution = await context.captcha.request({
 *   type: 'recaptcha',
 *   siteKey: '6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI'
 * });
 * ```
 */
export interface CaptchaClient {
  /**
   * Request CAPTCHA solution from user
   *
   * @param request - CAPTCHA request details
   * @returns CAPTCHA solution
   * @throws {Error} If user cancels or timeout
   */
  request(request: CaptchaRequest): Promise<CaptchaSolution>;
}

/**
 * CAPTCHA request
 */
export interface CaptchaRequest {
  /** CAPTCHA type */
  type: 'image' | 'recaptcha' | 'hcaptcha';

  /** Image URL (for type='image') */
  imageUrl?: string;

  /** Site key (for type='recaptcha' or 'hcaptcha') */
  siteKey?: string;
}

/**
 * CAPTCHA solution
 */
export interface CaptchaSolution {
  /** User's solution/token */
  solution: string;
}

// ============================================================================
// Logger
// ============================================================================

/**
 * Logging interface
 *
 * Logs are written to the audit log and displayed in the module management UI.
 *
 * @example
 * ```javascript
 * context.log.info('Starting download extraction');
 * context.log.warn('Approaching rate limit');
 * context.log.error('Failed to parse URL:', error);
 * ```
 */
export interface Logger {
  /**
   * Log informational message
   */
  info(...args: any[]): void;

  /**
   * Log warning message
   */
  warn(...args: any[]): void;

  /**
   * Log error message
   */
  error(...args: any[]): void;
}

// ============================================================================
// Download Info
// ============================================================================

/**
 * Download information returned by modules
 *
 * @example
 * ```javascript
 * return {
 *   downloadUrl: 'https://cdn.example.com/file.zip',
 *   filename: 'my-file.zip',
 *   filesize: 1024 * 1024 * 100, // 100 MB
 *   headers: {
 *     'Referer': 'https://example.com',
 *     'User-Agent': 'MyDownloader/1.0'
 *   },
 *   metadata: {
 *     downloadHints: {
 *       chunkSize: 1024 * 1024 * 20, // 20 MB chunks
 *       maxConnections: 10,
 *       requestDelay: 0
 *     }
 *   }
 * };
 * ```
 */
export interface DownloadInfo {
  /** Direct download URL */
  downloadUrl: string;

  /** File name (will be used to save the file) */
  filename: string;

  /** File size in bytes (optional, null if unknown) */
  filesize: number | null;

  /** Additional HTTP headers required for download */
  headers: Record<string, string>;

  /** Optional metadata for the downloader */
  metadata?: DownloadMetadata;
}

/**
 * Download metadata
 *
 * Optional hints and information for the download engine.
 */
export interface DownloadMetadata {
  /** Download optimization hints */
  downloadHints?: DownloadHints;

  /** URL expiry time (ISO 8601 string) */
  expiryTime?: string;

  /** Additional custom data */
  [key: string]: any;
}

/**
 * Download optimization hints
 *
 * Provides hints to the download engine for optimal performance.
 */
export interface DownloadHints {
  /** Recommended chunk size in bytes */
  chunkSize?: number;

  /** Maximum parallel connections */
  maxConnections?: number;

  /** Delay between requests in milliseconds */
  requestDelay?: number;

  /** Whether server supports range requests */
  supportsRangeRequests?: boolean;
}

// ============================================================================
// Global Type Aliases
// ============================================================================

/**
 * Module context type alias (for convenience)
 *
 * @example
 * ```javascript
 * /** @param {ModuleContext} context *\/
 * async function login(context) {
 *   await context.http.post(loginUrl, { ... });
 * }
 * ```
 */
export type ModuleContext = ModuleContext;

/**
 * Download info type alias (for convenience)
 */
export type ModuleDownloadInfo = DownloadInfo;

/**
 * HTTP response type alias (for convenience)
 */
export type ModuleHttpResponse = HttpResponse;

// ============================================================================
// Module Manifest (for reference - not used in JS code)
// ============================================================================

/**
 * Module manifest structure (manifest.json)
 *
 * **Note:** This is for reference only. Manifests are JSON files, not TypeScript.
 *
 * @example manifest.json
 * ```json
 * {
 *   "id": "k2s",
 *   "name": "Keep2Share Downloader",
 *   "version": "1.0.0",
 *   "author": "Zoo Downloader Community",
 *   "description": "Download files from Keep2Share (k2s.cc)",
 *   "homepage": "https://github.com/zoo-downloader/modules/k2s",
 *   "urlPatterns": [
 *     "https://k2s.cc/file/*",
 *     "https://*.k2s.cc/file/*"
 *   ],
 *   "permissions": {
 *     "network": {
 *       "allowDomains": ["k2s.cc", "*.k2s.cc", "api.keep2share.cc"]
 *     },
 *     "credentials": {
 *       "required": true,
 *       "credentialKeys": ["username", "password", "apiKey"]
 *     }
 *   }
 * }
 * ```
 */
export interface ModuleManifest {
  /** Unique module ID (lowercase alphanumeric + hyphens) */
  id: string;

  /** Display name */
  name: string;

  /** Semantic version (e.g., "1.0.0") */
  version: string;

  /** Author name or organization */
  author: string;

  /** Description */
  description: string;

  /** Homepage URL (optional) */
  homepage?: string;

  /** URL patterns this module supports */
  urlPatterns: string[];

  /** Permission declarations */
  permissions: {
    /** Network permissions */
    network: {
      /** Allowed domains (supports wildcards) */
      allowDomains: string[];
    };

    /** Credential permissions (optional) */
    credentials?: {
      /** Whether credentials are required */
      required: boolean;

      /** Required credential keys */
      credentialKeys: string[];
    };
  };
}

// ============================================================================
// Default Export
// ============================================================================

/**
 * Default export for convenience
 */
declare module 'zoo-downloader' {
  export {
    SiteModule,
    ModuleContext,
    HttpClient,
    HttpRequestOptions,
    HttpResponse,
    Credentials,
    CaptchaClient,
    CaptchaRequest,
    CaptchaSolution,
    Logger,
    DownloadInfo,
    DownloadMetadata,
    DownloadHints,
    ModuleManifest,
  };
}
