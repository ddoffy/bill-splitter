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

# Install Xcode Command Line Tools (no GUI needed!)
if ! xcode-select -p &> /dev/null; then
    echo "📦 Installing Xcode Command Line Tools..."
    xcode-select --install
    echo "⏳ Waiting for installation to complete..."
    echo "   (This may take a few minutes)"
    until xcode-select -p &> /dev/null; do
        sleep 5
    done
else
    echo "✅ Xcode Command Line Tools installed"
fi

# Accept license
echo "📝 Accepting Xcode license..."
sudo xcodebuild -license accept || true

echo ""
echo "✅ Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy your project to this Mac"
echo "2. Run: ./build-ios-remote.sh"
echo "3. Download the .ipa file"
echo "4. Install on iPhone"
