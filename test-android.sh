#!/bin/bash
# Test and build Android app without Android Studio

set -e

echo "🤖 Android CLI Build & Test"
echo "============================"
echo ""

cd "$(dirname "$0")/android"

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v java &> /dev/null; then
    echo "❌ Java is not installed"
    echo "   Install: sudo apt install openjdk-17-jdk"
    exit 1
fi
echo "✅ Java $(java -version 2>&1 | head -n 1)"

if ! command -v adb &> /dev/null; then
    echo "❌ adb is not installed"
    echo "   Install Android SDK platform-tools"
    exit 1
fi
echo "✅ adb installed"

echo ""
echo "📱 Checking for devices..."
adb devices -l

DEVICE_COUNT=$(adb devices | grep -v "List" | grep "device$" | wc -l)

if [ "$DEVICE_COUNT" -eq 0 ]; then
    echo ""
    echo "⚠️  No devices connected"
    echo ""
    echo "Options to test:"
    echo "1. Connect a physical Android device via USB (enable USB debugging)"
    echo "2. Start an emulator:"
    echo "   - Install Android emulator: sudo apt install android-emulator"
    echo "   - Or use Android Studio's AVD Manager"
    echo "3. Use a wireless device: adb connect <device-ip>:5555"
    echo ""
    read -p "Do you want to build anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo "🔨 Building debug APK..."
./gradlew assembleDebug

APK_PATH="app/build/outputs/apk/debug/app-debug.apk"

if [ -f "$APK_PATH" ]; then
    echo ""
    echo "✅ Build successful!"
    echo "📦 APK location: android/$APK_PATH"
    echo ""
    
    # Get APK size
    APK_SIZE=$(du -h "$APK_PATH" | cut -f1)
    echo "📊 APK size: $APK_SIZE"
    echo ""
    
    if [ "$DEVICE_COUNT" -gt 0 ]; then
        echo "📲 Installing on device..."
        adb install -r "$APK_PATH"
        
        echo ""
        echo "✅ App installed!"
        echo ""
        echo "🚀 Launching app..."
        adb shell am start -n org.ddoffy.splitbills/.MainActivity
        
        echo ""
        echo "📋 View logs with:"
        echo "   adb logcat | grep -i capacitor"
        echo ""
        echo "🔄 To reinstall:"
        echo "   adb install -r android/$APK_PATH"
    else
        echo "💡 To install manually:"
        echo "   adb install -r android/$APK_PATH"
    fi
else
    echo "❌ Build failed - APK not found"
    exit 1
fi
