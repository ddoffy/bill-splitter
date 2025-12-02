#!/bin/bash
# Show quick reference for mobile testing

cat << 'EOF'
╔══════════════════════════════════════════════════════════════╗
║           Split Bills Mobile - Quick Reference               ║
╚══════════════════════════════════════════════════════════════╝

🚀 QUICK START (No IDE Required)
────────────────────────────────────────────────────────────────
1. Connect Android device (USB debugging enabled)
2. Run: ./quick-test-android.sh
3. View logs: ./logs-android.sh
4. Debug: ./debug-android.sh

📱 ANDROID COMMANDS
────────────────────────────────────────────────────────────────
./quick-test-android.sh    → Sync + Build + Install + Launch
./test-android.sh          → Build + Install APK
./logs-android.sh          → View app logs in real-time
./debug-android.sh         → Open Chrome DevTools

npm run test:quick         → Same as quick-test-android.sh
npm run android:build      → Build APK only
npm run android:install    → Install APK to device
npm run android:logs       → View logs

🔧 DEVELOPMENT WORKFLOW
────────────────────────────────────────────────────────────────
1. Edit files in static/    → Change HTML/CSS/JS
2. npm run sync:android     → Sync changes to Android
3. ./quick-test-android.sh  → Build and test
4. ./logs-android.sh        → Monitor in real-time

🍎 iOS (Requires macOS + Xcode)
────────────────────────────────────────────────────────────────
npm run open:ios           → Open in Xcode
npm run sync:ios           → Sync web assets

📦 BUILDING
────────────────────────────────────────────────────────────────
# Debug builds
npm run android:build      → android/app/build/outputs/apk/debug/

# Release builds
npm run android:release    → android/app/build/outputs/apk/release/
cd android && ./gradlew bundleRelease  → For Google Play (AAB)

🔍 DEBUGGING
────────────────────────────────────────────────────────────────
./debug-android.sh         → Chrome DevTools (inspect app)
chrome://inspect           → Manual DevTools URL
adb logcat                 → Raw Android logs
adb shell                  → Access device shell

📱 DEVICE MANAGEMENT
────────────────────────────────────────────────────────────────
adb devices                → List connected devices
adb connect <ip>:5555      → Connect wireless device
adb disconnect             → Disconnect wireless device
adb kill-server            → Restart adb daemon

📲 APP MANAGEMENT
────────────────────────────────────────────────────────────────
adb install -r <apk>       → Install/reinstall APK
adb uninstall org.ddoffy.splitbills  → Uninstall app
adb shell am start -n org.ddoffy.splitbills/.MainActivity  → Launch app
adb shell pm clear org.ddoffy.splitbills  → Clear app data

📁 FILE LOCATIONS
────────────────────────────────────────────────────────────────
static/                    → Web assets (edit here)
android/app/build/outputs/ → Built APKs
capacitor.config.json      → App configuration

🌐 SERVER
────────────────────────────────────────────────────────────────
Current: https://billsplitter.ddoffy.org
Change in: capacitor.config.json → "server.url"
Then run: npm run sync

📚 DOCUMENTATION
────────────────────────────────────────────────────────────────
TESTING_WITHOUT_IDE.md     → Full CLI testing guide
MOBILE_DEV_CONFIG.md       → Development configuration
MOBILE.md                  → Complete mobile guide
README.md                  → Project overview

💡 TIPS
────────────────────────────────────────────────────────────────
• Use ./quick-test-android.sh for fast iterations
• Keep ./logs-android.sh running in another terminal
• Use Chrome DevTools for JavaScript debugging
• Physical device testing is more reliable than emulator
• Enable wireless debugging for cable-free development

🆘 TROUBLESHOOTING
────────────────────────────────────────────────────────────────
No devices found:
  → adb kill-server && adb start-server
  → Re-enable USB debugging on phone
  → Check USB cable

Build failed:
  → cd android && ./gradlew clean
  → Check Java version: java -version

App crashes:
  → ./logs-android.sh to see error
  → ./debug-android.sh for DevTools
  → Check network connection to server

Changes not appearing:
  → npm run sync:android
  → Rebuild: ./quick-test-android.sh
  → Clear app data: adb shell pm clear org.ddoffy.splitbills

EOF
