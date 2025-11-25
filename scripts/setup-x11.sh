#!/bin/bash

# Setup X11 for running Tauri in Docker container under Windows

# Get Windows host IP (for Docker Desktop)
# Try multiple methods to get the host IP
if command -v ip &> /dev/null; then
    export WINDOWS_HOST=$(ip route | grep default | awk '{print $3}')
elif command -v route &> /dev/null; then
    export WINDOWS_HOST=$(route -n | grep '^0.0.0.0' | awk '{print $2}')
elif [ -n "$HOST_IP" ]; then
    export WINDOWS_HOST=$HOST_IP
else
    # Fallback to common Docker Desktop host
    export WINDOWS_HOST="host.docker.internal"
fi

# If WINDOWS_HOST is empty, use the DISPLAY variable as-is
if [ -z "$WINDOWS_HOST" ]; then
    export DISPLAY=${DISPLAY:-:0.0}
else
    export DISPLAY=$WINDOWS_HOST:0.0
fi

# Set locale to avoid locale warnings
export LC_ALL=C.UTF-8
export LANG=C.UTF-8

# Disable GTK accessibility bus warning
export NO_AT_BRIDGE=1

# Enable X11 forwarding with relaxed security (for development only)
export XAUTHORITY=/tmp/.Xauthority
touch $XAUTHORITY
xauth generate $DISPLAY . trusted 2>/dev/null || true

echo "X11 environment configured:"
echo "  DISPLAY=$DISPLAY"
echo "  Windows Host IP: $WINDOWS_HOST"
echo ""
echo "Please make sure X11 server is running on Windows with 'Disable access control' enabled"
