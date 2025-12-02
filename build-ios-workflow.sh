#!/bin/bash
# Complete workflow: sync code to Mac, build, and download IPA

set -e

# Configuration
MAC_USER="${MAC_USER:-your-mac-username}"
MAC_HOST="${MAC_HOST:-your-mac-ip}"
MAC_PROJECT_DIR="${MAC_PROJECT_DIR:-~/split-bills}"
LOCAL_PROJECT_DIR="$(dirname "$0")"

echo "🔄 Remote iOS Build Workflow"
echo "============================="
echo ""
echo "Configuration:"
echo "  Mac User: $MAC_USER"
echo "  Mac Host: $MAC_HOST"
echo "  Mac Project Dir: $MAC_PROJECT_DIR"
echo ""
echo "💡 Set environment variables if different:"
echo "   export MAC_USER=your-username"
echo "   export MAC_HOST=192.168.1.100"
echo "   export MAC_PROJECT_DIR=~/my-project"
echo ""

# Check if we can reach the Mac
echo "📡 Testing SSH connection..."
if ! ssh -o ConnectTimeout=5 "$MAC_USER@$MAC_HOST" "echo 'Connected'" &> /dev/null; then
    echo "❌ Cannot connect to $MAC_USER@$MAC_HOST"
    echo ""
    echo "Make sure:"
    echo "  1. Mac is on and connected to network"
    echo "  2. SSH is enabled (System Preferences → Sharing → Remote Login)"
    echo "  3. You can connect: ssh $MAC_USER@$MAC_HOST"
    exit 1
fi
echo "✅ Connected to Mac"
echo ""

# Sync code to Mac
echo "📤 Syncing code to Mac..."
rsync -avz --progress \
    --exclude 'node_modules' \
    --exclude 'target' \
    --exclude 'ios' \
    --exclude 'android' \
    --exclude '.git' \
    --exclude 'sessions.db' \
    "$LOCAL_PROJECT_DIR/" \
    "$MAC_USER@$MAC_HOST:$MAC_PROJECT_DIR/"

echo ""
echo "✅ Code synced"
echo ""

# Setup environment on Mac (first time only)
echo "🔧 Checking Mac environment..."
if ! ssh "$MAC_USER@$MAC_HOST" "xcodebuild -version" &> /dev/null; then
    echo ""
    echo "❌ Xcode not found on Mac"
    echo ""
    echo "Please install Xcode on your Mac first:"
    echo ""
    echo "From your Mac (via SSH):"
    echo "   ssh $MAC_USER@$MAC_HOST"
    echo "   cd $MAC_PROJECT_DIR"
    echo "   ./install-xcode-mac.sh"
    echo ""
    echo "Or manually:"
    echo "   1. Open App Store on Mac"
    echo "   2. Search and install 'Xcode'"
    echo "   3. Wait for installation (~15GB, 30-60 min)"
    echo "   4. Run: sudo xcodebuild -license accept"
    echo "   5. Run this script again"
    echo ""
    exit 1
fi

echo "✅ Xcode found on Mac"
echo ""

echo "🔧 Setting up Mac environment (if needed)..."
ssh "$MAC_USER@$MAC_HOST" "cd $MAC_PROJECT_DIR && chmod +x *.sh && ./setup-mac-headless.sh" || {
    echo "❌ Setup failed"
    exit 1
}

echo ""
echo "🏗️  Building iOS app on Mac..."
ssh "$MAC_USER@$MAC_HOST" "cd $MAC_PROJECT_DIR && ./build-ios-remote.sh"

echo ""
echo "📦 Packaging IPA..."
ssh "$MAC_USER@$MAC_HOST" "cd $MAC_PROJECT_DIR && ./package-ios-ipa.sh"

echo ""
echo "📥 Downloading IPA from Mac..."
scp "$MAC_USER@$MAC_HOST:$MAC_PROJECT_DIR/SplitBills.ipa" ./

IPA_SIZE=$(du -h SplitBills.ipa | cut -f1)

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ BUILD COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📦 IPA File: ./SplitBills.ipa"
echo "📊 Size: $IPA_SIZE"
echo ""
echo "📤 Next steps:"
echo ""
echo "1. Host the IPA for download:"
echo "   python3 -m http.server 8000"
echo "   Access from iPhone: http://YOUR_IP:8000/SplitBills.ipa"
echo ""
echo "2. Or upload to cloud storage:"
echo "   - Dropbox/Google Drive"
echo "   - GitHub Release"
echo "   - Your server"
echo ""
echo "3. Install on iPhone using AltStore:"
echo "   - Download AltStore: https://altstore.io/"
echo "   - Install IPA via AltStore"
echo ""
