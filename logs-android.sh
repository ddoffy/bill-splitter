#!/bin/bash
# View Android app logs in real-time

echo "📱 Android App Logs (Capacitor)"
echo "==============================="
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Clear previous logs
adb logcat -c

# Show logs filtered for Capacitor and our app
adb logcat | grep -E "(Capacitor|SplitBills|chromium)" --color=always
