/**
 * URL Detection Utilities
 *
 * Provides client-side URL validation and detection utilities
 * for quick filtering before calling backend detection services.
 */

/**
 * Check if a string is a valid URL
 */
export const isValidUrl = (url: string): boolean => {
  try {
    const urlObj = new URL(url);
    return urlObj.protocol === 'http:' || urlObj.protocol === 'https:';
  } catch {
    return false;
  }
};

/**
 * Extract filename from URL
 */
export const extractFilename = (url: string): string | null => {
  try {
    const urlObj = new URL(url);
    const pathname = urlObj.pathname;
    const parts = pathname.split('/');
    const last = parts[parts.length - 1];
    return last || null;
  } catch {
    return null;
  }
};

/**
 * Get file extension from URL
 */
export const getFileExtension = (url: string): string | null => {
  const filename = extractFilename(url);
  if (!filename) return null;

  const dotIndex = filename.lastIndexOf('.');
  if (dotIndex === -1) return null;

  return filename.substring(dotIndex);
};

/**
 * Check if URL has a common downloadable file extension
 */
export const hasCommonFileExtension = (url: string): boolean => {
  const ext = getFileExtension(url);
  if (!ext) return false;

  const commonExts = [
    '.zip',
    '.pdf',
    '.exe',
    '.mp4',
    '.mkv',
    '.rar',
    '.7z',
    '.tar',
    '.gz',
    '.iso',
    '.dmg',
    '.apk',
  ];

  return commonExts.includes(ext.toLowerCase());
};

/**
 * Quick client-side check if URL might be a download link
 * This is used for fast filtering before calling backend detection
 */
export const mightBeDownloadLink = (url: string): boolean => {
  if (!isValidUrl(url)) return false;
  return hasCommonFileExtension(url);
};

/**
 * URL Detector utilities object
 */
export const urlDetector = {
  isValidUrl,
  extractFilename,
  getFileExtension,
  hasCommonFileExtension,
  mightBeDownloadLink,
};
