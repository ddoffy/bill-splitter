#!/bin/bash
# Open Chrome DevTools for mobile debugging

echo "🌐 Opening Chrome DevTools for Mobile Debugging"
echo "==============================================="
echo ""

# Check if Chrome/Chromium is installed
if command -v google-chrome &> /dev/null; then
    CHROME="google-chrome"
elif command -v chromium &> /dev/null; then
    CHROME="chromium"
elif command -v chromium-browser &> /dev/null; then
    CHROME="chromium-browser"
else
    echo "❌ Chrome/Chromium not found"
    echo ""
    echo "Install Chrome:"
    echo "  wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb"
    echo "  sudo dpkg -i google-chrome-stable_current_amd64.deb"
    echo ""
    echo "Or Chromium:"
    echo "  sudo apt install chromium-browser"
    exit 1
fi

echo "✅ Using: $CHROME"
echo ""

# Check for connected devices
DEVICE_COUNT=$(adb devices | grep -v "List" | grep "device$" | wc -l)

if [ "$DEVICE_COUNT" -eq 0 ]; then
    echo "⚠️  No Android devices connected"
    echo ""
    echo "Connect a device first:"
    echo "  1. Enable USB Debugging on your Android device"
    echo "  2. Connect via USB"
    echo "  3. Run: adb devices"
    exit 1
fi

echo "📱 Found $DEVICE_COUNT device(s)"
adb devices -l
echo ""

echo "🚀 Opening Chrome DevTools..."
echo ""
echo "Instructions:"
echo "  1. Make sure the Split Bills app is running on your device"
echo "  2. In Chrome DevTools, you'll see your device listed"
echo "  3. Click 'inspect' next to 'org.ddoffy.splitbills'"
echo "  4. Use full Chrome DevTools: Console, Network, Elements, etc."
echo ""

# Open Chrome inspect page
$CHROME chrome://inspect &

echo "✅ DevTools opened!"
echo ""
echo "💡 Tips:"
echo "  - Use Console to see JavaScript logs"
echo "  - Use Network tab to debug API calls"
echo "  - Use Elements to inspect HTML/CSS"
echo "  - Changes in DevTools are temporary (not saved to files)"
