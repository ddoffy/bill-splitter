#!/bin/bash
# Setup iOS development environment on Mac via SSH (headless)

set -e

echo "🍎 iOS Development Setup (Headless Mac)"
echo "========================================"
echo ""

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ This script must run on macOS"
    exit 1
fi

echo "✅ Running on macOS"
echo ""

# Install Homebrew if not present
if ! command -v brew &> /dev/null; then
    echo "📦 Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "✅ Homebrew installed"
fi

# Install Node.js
if ! command -v node &> /dev/null; then
    echo "📦 Installing Node.js..."
    brew install node
else
    echo "✅ Node.js $(node --version)"
fi

# Install CocoaPods
if ! command -v pod &> /dev/null; then
    echo "📦 Installing CocoaPods..."
    sudo gem install cocoapods
else
    echo "✅ CocoaPods $(pod --version)"
fi

# Check for Xcode
if ! xcodebuild -version &> /dev/null; then
    echo "❌ Xcode is required but not installed"
    echo ""
    echo "📋 To install Xcode (choose one option):"
    echo ""
    echo "Option 1: Install from App Store (easiest, but large ~15GB)"
    echo "   1. Open App Store"
    echo "   2. Search for 'Xcode'"
    echo "   3. Click 'Get' or 'Install'"
    echo "   4. Wait for installation (takes 30-60 minutes)"
    echo "   5. Re-run this script"
    echo ""
    echo "Option 2: Download from Apple Developer (faster with good connection)"
    echo "   1. Go to: https://developer.apple.com/download/all/"
    echo "   2. Sign in with Apple ID (free)"
    echo "   3. Download latest Xcode .xip file"
    echo "   4. Double-click to extract"
    echo "   5. Move Xcode.app to /Applications/"
    echo "   6. Run: sudo xcode-select --switch /Applications/Xcode.app"
    echo "   7. Run: sudo xcodebuild -license accept"
    echo "   8. Re-run this script"
    echo ""
    echo "Option 3: Use command line (requires Apple ID)"
    echo "   Run: mas install 497799835  # Requires 'mas' tool"
    echo ""
    exit 1
fi

echo "✅ Xcode $(xcodebuild -version | head -1)"

# Accept license if needed
echo "📝 Accepting Xcode license..."
sudo xcodebuild -license accept 2>/dev/null || {
    echo "⚠️  Please accept Xcode license:"
    sudo xcodebuild -license
}

# Verify setup
echo ""
echo "🔍 Verifying installation..."
echo ""

if xcodebuild -version &> /dev/null; then
    echo "✅ Xcode: $(xcodebuild -version | head -1)"
else
    echo "❌ Xcode verification failed"
    exit 1
fi

if command -v node &> /dev/null; then
    echo "✅ Node.js: $(node --version)"
else
    echo "❌ Node.js verification failed"
    exit 1
fi

if command -v pod &> /dev/null; then
    echo "✅ CocoaPods: $(pod --version)"
else
    echo "❌ CocoaPods verification failed"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SETUP COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Next steps:"
echo "1. Copy your project to this Mac"
echo "2. Run: ./build-ios-remote.sh"
echo "3. Download the .ipa file"
echo "4. Install on iPhone"
echo ""
