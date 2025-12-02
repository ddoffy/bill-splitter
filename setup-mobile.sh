#!/bin/bash
# Quick development setup script for mobile apps

echo "🚀 Split Bills Mobile Development Setup"
echo "========================================"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed"
    echo "   Install from: https://nodejs.org/"
    exit 1
fi
echo "✅ Node.js $(node --version)"

# npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed"
    exit 1
fi
echo "✅ npm $(npm --version)"

# Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed"
    echo "   Install from: https://rustup.rs/"
    exit 1
fi
echo "✅ Rust $(rustc --version | cut -d' ' -f2)"

echo ""
echo "📦 Installing dependencies..."
npm install

echo ""
echo "🔄 Syncing mobile platforms..."
npx cap sync

echo ""
echo "✅ Setup complete!"
echo ""
echo "📱 Next steps:"
echo ""
echo "1. Start the backend server:"
echo "   cargo run"
echo ""
echo "2. For iOS development:"
echo "   - Install Xcode from the Mac App Store"
echo "   - Install CocoaPods: sudo gem install cocoapods"
echo "   - Open project: npm run open:ios"
echo ""
echo "3. For Android development:"
echo "   - Install Android Studio"
echo "   - Open project: npm run open:android"
echo ""
echo "📖 For detailed instructions, see MOBILE.md"
