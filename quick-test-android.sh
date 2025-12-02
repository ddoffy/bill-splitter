#!/bin/bash
# Quick test script - builds and installs Android app

echo "🔄 Quick Android Test"
echo "===================="
echo ""

cd "$(dirname "$0")"

# Sync latest changes
echo "📦 Syncing web assets..."
npm run sync:android

# Build and install
echo ""
./test-android.sh
