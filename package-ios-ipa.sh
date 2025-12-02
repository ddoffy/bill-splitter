#!/bin/bash
# Package iOS app as IPA and prepare for distribution

set -e

echo "📦 Packaging iOS App as IPA"
echo "============================"
echo ""

cd "$(dirname "$0")"

APP_PATH="ios/App/build/Debug-iphoneos/App.app"
PAYLOAD_DIR="Payload"
IPA_NAME="SplitBills.ipa"

if [ ! -d "$APP_PATH" ]; then
    echo "❌ App not found at $APP_PATH"
    echo "   Run: ./build-ios-remote.sh first"
    exit 1
fi

echo "📦 Creating IPA package..."

# Clean previous build
rm -rf "$PAYLOAD_DIR" "$IPA_NAME"

# Create Payload directory
mkdir -p "$PAYLOAD_DIR"

# Copy app to Payload
cp -r "$APP_PATH" "$PAYLOAD_DIR/"

# Create IPA (it's just a zip file)
zip -r "$IPA_NAME" "$PAYLOAD_DIR"

# Clean up
rm -rf "$PAYLOAD_DIR"

# Get file size
IPA_SIZE=$(du -h "$IPA_NAME" | cut -f1)

echo ""
echo "✅ IPA created successfully!"
echo ""
echo "📦 File: $IPA_NAME"
echo "📊 Size: $IPA_SIZE"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 DISTRIBUTION OPTIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Option 1: Upload to Cloud"
echo "   scp $IPA_NAME user@your-server:/path/to/web/directory/"
echo "   Then download on iPhone"
echo ""
echo "Option 2: Transfer to Linux machine"
echo "   scp $IPA_NAME user@linux-ip:~/"
echo "   Then host with: python3 -m http.server 8000"
echo ""
echo "Option 3: Use GitHub Releases"
echo "   Upload as release asset, download on iPhone"
echo ""
echo "Option 4: Use Dropbox/Google Drive"
echo "   Upload, share link, download on iPhone"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📱 INSTALLING ON IPHONE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⚠️  This is an unsigned development build"
echo ""
echo "Method 1: AltStore (Recommended)"
echo "  1. Install AltStore on your computer"
echo "  2. Install AltStore on iPhone"
echo "  3. Sideload the IPA via AltStore"
echo "  4. Valid for 7 days (free account)"
echo ""
echo "Method 2: Apple Configurator (Mac only)"
echo "  1. Install Apple Configurator 2"
echo "  2. Connect iPhone via USB"
echo "  3. Add app → select IPA"
echo ""
echo "Method 3: Xcode (requires GUI)"
echo "  1. Open Xcode"
echo "  2. Window → Devices and Simulators"
echo "  3. Connect iPhone"
echo "  4. Click '+' → select IPA"
echo ""
echo "Method 4: Sign properly (needs Apple Developer)"
echo "  Run: ./sign-ios-ipa.sh"
echo ""
