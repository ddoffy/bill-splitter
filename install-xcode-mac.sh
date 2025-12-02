#!/bin/bash
# Install Xcode via command line (requires Apple ID)

echo "📦 Installing Xcode"
echo "==================="
echo ""

# Check if mas is installed (Mac App Store CLI)
if ! command -v mas &> /dev/null; then
    echo "Installing 'mas' (Mac App Store CLI)..."
    if command -v brew &> /dev/null; then
        brew install mas
    else
        echo "❌ Homebrew not found. Install it first:"
        echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
fi

echo "✅ mas installed"
echo ""

# Check if signed in to App Store
if ! mas account &> /dev/null; then
    echo "⚠️  Not signed in to App Store"
    echo ""
    echo "Please sign in:"
    echo "1. Open App Store app"
    echo "2. Sign in with your Apple ID"
    echo "3. Re-run this script"
    echo ""
    echo "Or sign in via command line:"
    echo "   mas signin your-email@example.com"
    exit 1
fi

echo "✅ Signed in to App Store: $(mas account)"
echo ""

# Install Xcode (497799835 is Xcode's App Store ID)
echo "⬇️  Installing Xcode from App Store..."
echo "   (This will take 30-60 minutes depending on your connection)"
echo "   File size: ~15GB"
echo ""

mas install 497799835

echo ""
echo "✅ Xcode installed!"
echo ""

# Set Xcode path
echo "🔧 Setting Xcode path..."
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# Accept license
echo "📝 Accepting license..."
sudo xcodebuild -license accept

# Install additional components
echo "📦 Installing additional components..."
sudo xcodebuild -runFirstLaunch

echo ""
echo "✅ Xcode setup complete!"
echo ""
echo "Run: ./setup-mac-headless.sh to continue"
