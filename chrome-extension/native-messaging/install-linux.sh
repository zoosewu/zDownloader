#!/bin/bash
# Install Native Messaging manifest for Linux

MANIFEST_NAME="com.zoo.downloader.json"
HOST_NAME="com.zoo.downloader"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Chrome native messaging manifest directory
CHROME_MANIFEST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"

# Chromium native messaging manifest directory
CHROMIUM_MANIFEST_DIR="$HOME/.config/chromium/NativeMessagingHosts"

# Prompt for Zoo Downloader executable path
read -p "Enter the full path to zoo-downloader executable: " APP_PATH

# Validate path
if [ ! -f "$APP_PATH" ]; then
    echo "Error: File not found: $APP_PATH"
    exit 1
fi

# Update manifest with correct path
sed "s|/path/to/zoo-downloader|$APP_PATH|g" "$SCRIPT_DIR/$MANIFEST_NAME" > "$SCRIPT_DIR/${MANIFEST_NAME}.tmp"
mv "$SCRIPT_DIR/${MANIFEST_NAME}.tmp" "$SCRIPT_DIR/$MANIFEST_NAME"

# Install for Chrome
if [ -d "$HOME/.config/google-chrome" ]; then
    mkdir -p "$CHROME_MANIFEST_DIR"
    cp "$SCRIPT_DIR/$MANIFEST_NAME" "$CHROME_MANIFEST_DIR/$MANIFEST_NAME"
    echo "Installed for Google Chrome"
fi

# Install for Chromium
if [ -d "$HOME/.config/chromium" ]; then
    mkdir -p "$CHROMIUM_MANIFEST_DIR"
    cp "$SCRIPT_DIR/$MANIFEST_NAME" "$CHROMIUM_MANIFEST_DIR/$MANIFEST_NAME"
    echo "Installed for Chromium"
fi

echo ""
echo "Native Messaging host installed successfully!"
echo "Manifest location: $SCRIPT_DIR/$MANIFEST_NAME"
echo ""
echo "Next steps:"
echo "1. Install the Chrome extension"
echo "2. Update the manifest with your extension ID"
echo "3. Restart Chrome/Chromium"
