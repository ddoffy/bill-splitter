#!/bin/bash
set -e

echo "🔨 Rebuilding..."
cargo build --release

echo "🔄 Restarting service..."
sudo systemctl restart split-bills

echo "✅ Service status:"
systemctl status split-bills --no-pager
