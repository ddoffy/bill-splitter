#!/bin/bash
# Start server and show access instructions

echo "🚀 Starting Split Bills Server..."
echo ""

# Start the server in background
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Get local IP addresses
echo "✅ Server is running!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📱 ACCESS FROM YOUR DEVICES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Local access
echo "💻 On this computer:"
echo "   http://localhost:7777"
echo "   http://127.0.0.1:7777"
echo ""

# Network access
echo "📱 On your iPhone/other devices (same WiFi):"
IP_ADDRESSES=$(ip -4 addr | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v 127.0.0.1)

if [ -n "$IP_ADDRESSES" ]; then
    while IFS= read -r ip; do
        echo "   http://$ip:7777"
    done <<< "$IP_ADDRESSES"
else
    echo "   Run: ip addr"
    echo "   Find your IP (usually starts with 192.168 or 10.0)"
    echo "   Then: http://YOUR_IP:7777"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 TESTING ON IPHONE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Make sure your iPhone is on the SAME WiFi network"
echo "2. Open Safari on your iPhone"
echo "3. Type the URL shown above (http://YOUR_IP:7777)"
echo "4. Tap Share → Add to Home Screen"
echo "5. Launch from home screen for full-screen experience"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🛑 TO STOP SERVER"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Press Ctrl+C or run: kill $SERVER_PID"
echo ""

# Keep running and show logs
wait $SERVER_PID
