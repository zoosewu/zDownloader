// Background service worker for Zoo Downloader extension

const APP_HTTP_URL = 'http://127.0.0.1:45677';
let isConnected = false;
let contextMenuUrl = null;

// Check connection to desktop app via HTTP
async function checkConnection() {
  try {
    const response = await fetch(`${APP_HTTP_URL}/health`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (response.ok) {
      const data = await response.json();
      isConnected = true;
      console.log('Connected to Zoo Downloader:', data);
      return true;
    } else {
      isConnected = false;
      return false;
    }
  } catch (error) {
    console.log('Zoo Downloader app not running or not reachable');
    isConnected = false;
    return false;
  }
}

// Send download request to desktop app via HTTP
async function sendToApp(url, filename, referer) {
  try {
    const response = await fetch(`${APP_HTTP_URL}/add-download`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        url: url,
        filename: filename,
        referer: referer,
      }),
    });

    if (response.ok) {
      const data = await response.json();
      if (data.success) {
        console.log('Download added successfully:', data);
        return { success: true, message: data.message };
      } else {
        console.error('Failed to add download:', data.message);
        return { success: false, message: data.message };
      }
    } else {
      const error = `HTTP error: ${response.status}`;
      console.error(error);
      return { success: false, message: error };
    }
  } catch (error) {
    console.error('Failed to send download request:', error);
    return { success: false, message: error.message };
  }
}

// Show browser notification
function showNotification(message, isError = false) {
  chrome.notifications.create({
    type: 'basic',
    iconUrl: 'icons/icon128.png',
    title: 'Zoo Downloader',
    message: message,
    priority: isError ? 2 : 1
  });
}

// Create context menu
function createContextMenu() {
  chrome.contextMenus.create({
    id: 'zoo-download',
    title: 'Send to Zoo Downloader',
    contexts: ['link']
  });
}

// Handle context menu clicks
chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId === 'zoo-download' && info.linkUrl) {
    const filename = extractFilename(info.linkUrl);

    const result = await sendToApp(info.linkUrl, filename, tab.url);

    if (result.success) {
      showNotification(`Sent to Zoo Downloader: ${filename}`);
    } else {
      showNotification(`Failed to send: ${result.message}`, true);
    }
  }
});

// Extract filename from URL
function extractFilename(url) {
  try {
    const urlObj = new URL(url);
    const pathname = urlObj.pathname;
    const parts = pathname.split('/');
    const filename = parts[parts.length - 1];
    return filename && filename.includes('.') ? decodeURIComponent(filename) : 'download';
  } catch (e) {
    return 'download';
  }
}

// Handle messages from content script and popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'CHECK_CONNECTION') {
    checkConnection().then((connected) => {
      sendResponse({ connected });
    });
  } else if (message.type === 'SEND_TO_NATIVE') {
    sendToApp(message.url, message.filename, sender.tab?.url).then((result) => {
      if (result.success) {
        showNotification(`Sent to Zoo Downloader: ${message.filename}`);
      } else {
        showNotification(`Failed to send: ${result.message}`, true);
      }
      sendResponse({ success: result.success });
    });
  } else if (message.type === 'UPDATE_CONTEXT_MENU') {
    contextMenuUrl = message.url;
    sendResponse({ success: true });
  }

  return true; // Keep channel open for async response
});

// Listen for download events from Chrome's download manager
chrome.downloads.onCreated.addListener((downloadItem) => {
  // Optionally intercept downloads and send to Zoo Downloader instead
  // This can be enabled by user preference

  // For now, just log the download
  console.log('Download created:', downloadItem);
});

// Handle extension installation
chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === 'install') {
    console.log('Zoo Downloader extension installed');
    createContextMenu();

    // Show welcome notification
    showNotification('Zoo Downloader extension installed! Right-click on download links to send to the app.');
  } else if (details.reason === 'update') {
    console.log('Zoo Downloader extension updated');
  }

  // Check connection to app
  checkConnection();
});

// Handle extension startup
chrome.runtime.onStartup.addListener(() => {
  console.log('Zoo Downloader extension started');
  createContextMenu();
  checkConnection();
});

// Initialize on load
createContextMenu();
checkConnection();

// Keep service worker alive and check connection periodically
chrome.alarms.create('keepAlive', { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'keepAlive') {
    // Periodically check connection
    checkConnection();
  }
});
