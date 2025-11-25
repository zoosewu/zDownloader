#!/bin/bash

echo "========================================="
echo "Tauri Dev Mode Launcher for Docker"
echo "========================================="
echo ""

# Setup X11 environment
source ./scripts/setup-x11.sh

echo ""
echo "Testing X11 connection..."
echo "If this hangs, press Ctrl+C and check your X11 server settings"
echo ""

# Try to test X11 connection (but don't fail if xset is not installed)
if command -v xset &> /dev/null; then
    if xset q &>/dev/null; then
        echo "✓ X11 connection successful!"
    else
        echo "✗ X11 connection failed!"
        echo ""
        echo "Please check:"
        echo "1. X11 server is running on Windows (VcXsrv, X410, etc.)"
        echo "2. 'Disable access control' is enabled in X11 server settings"
        echo "3. Windows Firewall allows connections on port 6000"
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
else
    echo "⚠ xset not installed, skipping X11 test"
    echo "  Install with: apt-get update && apt-get install -y x11-apps"
fi

echo ""
echo "Starting Tauri dev mode..."
echo "========================================="
echo ""

# Run Tauri dev
npm run tauri dev
