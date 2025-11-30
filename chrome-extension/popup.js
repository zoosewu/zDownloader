// Popup script for Zoo Downloader extension
let detectedLinks = [];

// Check native app connection status
async function checkConnection() {
  try {
    const response = await chrome.runtime.sendMessage({ type: 'CHECK_CONNECTION' });
    updateStatus(response.connected);
  } catch (error) {
    console.error('Connection check failed:', error);
    updateStatus(false);
  }
}

function updateStatus(connected) {
  const statusEl = document.getElementById('status');
  if (connected) {
    statusEl.className = 'status connected';
    statusEl.textContent = 'Status: Connected to Zoo Downloader';
  } else {
    statusEl.className = 'status disconnected';
    statusEl.textContent = 'Status: Not connected (App not running)';
  }
}

// Detect links on current page
async function detectLinks() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

  chrome.tabs.sendMessage(tab.id, { type: 'DETECT_LINKS' }, (response) => {
    if (chrome.runtime.lastError) {
      console.error('Error detecting links:', chrome.runtime.lastError);
      return;
    }

    detectedLinks = response.links || [];
    document.getElementById('linkCount').textContent = detectedLinks.length;
    document.getElementById('sendAllLinks').disabled = detectedLinks.length === 0;
  });
}

// Send all detected links to native app
async function sendAllLinks() {
  if (detectedLinks.length === 0) return;

  const button = document.getElementById('sendAllLinks');
  button.disabled = true;
  button.textContent = 'Sending...';

  try {
    for (const link of detectedLinks) {
      await chrome.runtime.sendMessage({
        type: 'SEND_TO_NATIVE',
        url: link.url,
        filename: link.filename
      });
    }

    button.textContent = `Sent ${detectedLinks.length} links!`;
    setTimeout(() => {
      button.textContent = 'Send All Links to Downloader';
      button.disabled = false;
    }, 2000);
  } catch (error) {
    console.error('Error sending links:', error);
    button.textContent = 'Error sending links';
    button.disabled = false;
  }
}

// Initialize popup
document.addEventListener('DOMContentLoaded', () => {
  checkConnection();
  detectLinks();

  document.getElementById('detectLinks').addEventListener('click', detectLinks);
  document.getElementById('sendAllLinks').addEventListener('click', sendAllLinks);
});
