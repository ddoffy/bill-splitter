#!/bin/bash
# Build iOS app on Mac via SSH (no Xcode GUI required)

set -e

echo "🏗️  Building iOS App (Command Line)"
echo "===================================="
echo ""

cd "$(dirname "$0")"

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Run: ./setup-mac-headless.sh"
    exit 1
fi

if ! command -v pod &> /dev/null; then
    echo "❌ CocoaPods not found. Run: ./setup-mac-headless.sh"
    exit 1
fi

if ! xcode-select -p &> /dev/null; then
    echo "❌ Xcode Command Line Tools not found. Run: ./setup-mac-headless.sh"
    exit 1
fi

echo "✅ All prerequisites met"
echo ""

# Install npm dependencies
echo "📦 Installing npm dependencies..."
npm install

# Sync Capacitor
echo "🔄 Syncing Capacitor..."
npx cap sync ios

# Install CocoaPods dependencies
echo "📦 Installing iOS dependencies..."
cd ios/App
pod install
cd ../..

# Build the app
echo "🏗️  Building iOS app..."
cd ios/App

# For development/testing - unsigned build
echo "   Building unsigned (for testing)..."
xcodebuild \
    -workspace App.xcworkspace \
    -scheme App \
    -configuration Debug \
    -sdk iphoneos \
    -destination generic/platform=iOS \
    -allowProvisioningUpdates \
    clean build \
    CODE_SIGN_IDENTITY="" \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 App location: ios/App/build/Debug-iphoneos/App.app"
echo ""
echo "📋 Next steps:"
echo "1. Create IPA for installation"
echo "2. Run: ./package-ios-ipa.sh"
