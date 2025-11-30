// Content script for Zoo Downloader - detects download links on web pages

// Common download file extensions
const DOWNLOAD_EXTENSIONS = [
  // Archives
  '.zip', '.rar', '.7z', '.tar', '.gz', '.bz2', '.xz', '.tar.gz', '.tar.bz2',
  // Executables
  '.exe', '.msi', '.dmg', '.pkg', '.deb', '.rpm', '.appimage',
  // Documents
  '.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
  // Media
  '.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm',
  '.mp3', '.flac', '.wav', '.aac', '.ogg', '.m4a',
  // Images
  '.jpg', '.jpeg', '.png', '.gif', '.bmp', '.svg', '.webp', '.ico',
  // Development
  '.apk', '.ipa', '.jar', '.war',
  // Others
  '.iso', '.img', '.bin', '.torrent'
];

// Detect if a URL is a download link
function isDownloadLink(url) {
  try {
    const urlObj = new URL(url, window.location.href);
    const pathname = urlObj.pathname.toLowerCase();

    // Check file extension
    for (const ext of DOWNLOAD_EXTENSIONS) {
      if (pathname.endsWith(ext)) {
        return true;
      }
    }

    // Check for download parameter
    if (urlObj.searchParams.has('download') ||
        urlObj.searchParams.has('dl') ||
        urlObj.searchParams.has('file')) {
      return true;
    }

    // Check for common download paths
    if (pathname.includes('/download/') ||
        pathname.includes('/files/') ||
        pathname.includes('/attachments/')) {
      return true;
    }

    return false;
  } catch (e) {
    return false;
  }
}

// Extract filename from URL
function extractFilename(url) {
  try {
    const urlObj = new URL(url, window.location.href);
    const pathname = urlObj.pathname;
    const parts = pathname.split('/');
    const filename = parts[parts.length - 1];

    // If filename has extension, return it
    if (filename && filename.includes('.')) {
      return decodeURIComponent(filename);
    }

    // Try to get filename from Content-Disposition header (if available)
    const contentDisposition = urlObj.searchParams.get('response-content-disposition');
    if (contentDisposition) {
      const match = contentDisposition.match(/filename[^;=\n]*=((['"]).*?\2|[^;\n]*)/);
      if (match && match[1]) {
        return match[1].replace(/['"]/g, '');
      }
    }

    return filename || 'download';
  } catch (e) {
    return 'download';
  }
}

// Detect all download links on the page
function detectDownloadLinks() {
  const links = [];
  const seenUrls = new Set();

  // Find all <a> tags with href
  document.querySelectorAll('a[href]').forEach(anchor => {
    const href = anchor.href;

    if (href && !seenUrls.has(href) && isDownloadLink(href)) {
      seenUrls.add(href);
      links.push({
        url: href,
        filename: extractFilename(href),
        text: anchor.textContent.trim() || anchor.title || ''
      });
    }
  });

  return links;
}

// Add context menu icon to download links
function addDownloadIcons() {
  document.querySelectorAll('a[href]').forEach(anchor => {
    if (isDownloadLink(anchor.href) && !anchor.dataset.zooDownloader) {
      anchor.dataset.zooDownloader = 'true';

      // Add visual indicator
      const icon = document.createElement('span');
      icon.textContent = '⬇️';
      icon.style.cssText = `
        margin-left: 4px;
        cursor: pointer;
        font-size: 0.9em;
      `;
      icon.title = 'Send to Zoo Downloader';

      // Click handler to send to native app
      icon.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();

        chrome.runtime.sendMessage({
          type: 'SEND_TO_NATIVE',
          url: anchor.href,
          filename: extractFilename(anchor.href)
        });

        // Show notification
        showNotification(`Sent to Zoo Downloader: ${extractFilename(anchor.href)}`);
      });

      anchor.appendChild(icon);
    }
  });
}

// Show notification
function showNotification(message) {
  const notification = document.createElement('div');
  notification.textContent = message;
  notification.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background-color: #4caf50;
    color: white;
    padding: 12px 20px;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.2);
    z-index: 999999;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
  `;

  document.body.appendChild(notification);

  setTimeout(() => {
    notification.style.transition = 'opacity 0.3s';
    notification.style.opacity = '0';
    setTimeout(() => notification.remove(), 300);
  }, 3000);
}

// Listen for messages from popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'DETECT_LINKS') {
    const links = detectDownloadLinks();
    sendResponse({ links });
  }
  return true;
});

// Auto-detect and add icons when page loads
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', addDownloadIcons);
} else {
  addDownloadIcons();
}

// Watch for dynamically added links
const observer = new MutationObserver(() => {
  addDownloadIcons();
});

observer.observe(document.body, {
  childList: true,
  subtree: true
});

// Add context menu for selected links
document.addEventListener('contextmenu', (e) => {
  const target = e.target.closest('a[href]');
  if (target && isDownloadLink(target.href)) {
    // Context menu will be handled by background.js
    chrome.runtime.sendMessage({
      type: 'UPDATE_CONTEXT_MENU',
      url: target.href,
      filename: extractFilename(target.href)
    });
  }
});
