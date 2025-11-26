#!/bin/bash
set -e

# Check if the service exists
if ! systemctl list-unit-files | grep -q split-bills.service; then
    echo "⚠️ Service not found. Running installation first..."
    chmod +x install.sh
    ./install.sh
    exit 0
fi

echo "🔨 Rebuilding..."
cargo build --release

echo "🔄 Restarting service..."
sudo systemctl restart split-bills

echo "✅ Service status:"
systemctl status split-bills --no-pager
