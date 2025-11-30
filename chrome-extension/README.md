# Zoo Downloader Chrome Extension

This Chrome extension integrates with the Zoo Downloader desktop application to automatically detect download links on web pages and send them to the app for downloading.

## Features

- 🔍 **Automatic Link Detection**: Automatically detects download links on web pages
- 📥 **One-Click Download**: Click the icon next to download links to send them to Zoo Downloader
- 🎯 **Context Menu**: Right-click on any link to send it to Zoo Downloader
- 📋 **Batch Download**: Detect and send all download links on a page at once
- 🔗 **Native Integration**: Seamlessly communicates with the desktop app

## Installation

### Step 1: Install the Desktop App

Make sure you have Zoo Downloader desktop application installed and running.

### Step 2: Install the Chrome Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in the top-right corner)
3. Click "Load unpacked"
4. Select the `chrome-extension` folder from your Zoo Downloader installation

### Step 3: Start Zoo Downloader

Make sure the Zoo Downloader desktop application is running. The extension communicates with the app via HTTP on port 45677 (localhost only).

**No additional configuration needed!** The extension will automatically detect and connect to the running app.

### Step 4: Restart Chrome (Optional)

Close and reopen Chrome if you had the extension loaded before starting the app.

## Usage

### Automatic Link Detection

When you browse a webpage, the extension automatically detects download links and adds a ⬇️ icon next to them. Click the icon to send the link to Zoo Downloader.

### Context Menu

Right-click on any link and select "Send to Zoo Downloader" from the context menu.

### Extension Popup

Click the extension icon in the toolbar to:
- View the connection status with the desktop app
- See how many download links were detected on the current page
- Send all detected links to Zoo Downloader at once

## Supported File Types

The extension detects the following file types:

- **Archives**: .zip, .rar, .7z, .tar, .gz, .bz2, .xz
- **Executables**: .exe, .msi, .dmg, .pkg, .deb, .rpm, .appimage
- **Documents**: .pdf, .doc, .docx, .xls, .xlsx, .ppt, .pptx
- **Media**: .mp4, .mkv, .avi, .mov, .wmv, .mp3, .flac, .wav
- **Images**: .jpg, .png, .gif, .bmp, .svg, .webp
- **Development**: .apk, .ipa, .jar, .war
- **Others**: .iso, .img, .bin, .torrent

## Troubleshooting

### Extension shows "Not connected"

1. Make sure Zoo Downloader desktop app is running
2. Check that the app is listening on port 45677 (check app logs)
3. Ensure no firewall is blocking localhost connections
4. Try restarting both Chrome and the app

### HTTP connection fails

1. Verify the app is running and listening on http://127.0.0.1:45677
2. Check browser console for CORS errors (F12 → Console)
3. Ensure no other application is using port 45677
4. Try accessing http://127.0.0.1:45677/health in your browser directly

### Downloads don't start

1. Check that the desktop app is running
2. Verify connection status in the extension popup
3. Check the browser console for errors (F12 → Console)
4. Check the desktop app logs for errors

## Development

### Project Structure

```
chrome-extension/
├── manifest.json          # Extension manifest
├── background.js          # Background service worker (HTTP client)
├── content.js             # Content script for link detection
├── popup.html             # Extension popup UI
├── popup.js               # Popup logic
├── icons/                 # Extension icons
└── README.md              # Installation guide
```

### Building

No build step is required. The extension can be loaded directly in Chrome.

### Testing

1. Load the extension in Chrome
2. Navigate to a page with download links
3. Check that links are detected
4. Click on a download icon or use the context menu
5. Verify that the download appears in the Zoo Downloader app

## Privacy & Security

- The extension only detects download links on the pages you visit
- No data is sent to any external servers
- Communication is only between the extension and your local Zoo Downloader app via HTTP (localhost only)
- The extension requires `<all_urls>` permission to detect links on all websites
- HTTP server only listens on 127.0.0.1 (localhost) and is not accessible from the internet

## License

This extension is part of Zoo Downloader and is licensed under the same terms.

## Support

For issues and questions, please visit the Zoo Downloader GitHub repository.
