#!/bin/bash
# Host IPA file for iPhone download

set -e

IPA_FILE="${1:-SplitBills.ipa}"

if [ ! -f "$IPA_FILE" ]; then
    echo "❌ IPA file not found: $IPA_FILE"
    echo ""
    echo "Usage: $0 [ipa-file]"
    echo "Example: $0 SplitBills.ipa"
    exit 1
fi

# Get file info
IPA_SIZE=$(du -h "$IPA_FILE" | cut -f1)
IPA_FULL_PATH=$(realpath "$IPA_FILE")

# Get local IP
LOCAL_IP=$(ip -4 addr | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v 127.0.0.1 | head -1)

echo "📤 Hosting iOS App for Download"
echo "================================"
echo ""
echo "📦 File: $IPA_FILE"
echo "📊 Size: $IPA_SIZE"
echo ""
echo "🌐 Server starting..."
echo ""

# Create temporary directory with just the IPA
TEMP_DIR=$(mktemp -d)
cp "$IPA_FILE" "$TEMP_DIR/"
cd "$TEMP_DIR"

# Create a simple HTML page
cat > index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Download Split Bills</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            text-align: center;
        }
        .download-btn {
            background: #007AFF;
            color: white;
            padding: 15px 30px;
            border-radius: 10px;
            text-decoration: none;
            display: inline-block;
            margin: 20px 0;
            font-size: 18px;
        }
        .instructions {
            background: #f5f5f5;
            padding: 20px;
            border-radius: 10px;
            margin: 20px 0;
            text-align: left;
        }
        .warning {
            background: #fff3cd;
            padding: 15px;
            border-radius: 10px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <h1>📱 Split Bills iOS App</h1>
    
    <div class="warning">
        ⚠️ This is an unsigned development build. You'll need AltStore or similar to install.
    </div>
    
    <a href="SplitBills.ipa" class="download-btn">⬇️ Download IPA</a>
    
    <div class="instructions">
        <h3>📋 Installation Steps:</h3>
        <ol>
            <li><strong>Install AltStore:</strong>
                <ul>
                    <li>Download from <a href="https://altstore.io/">altstore.io</a></li>
                    <li>Install on your computer</li>
                    <li>Install AltStore app on your iPhone</li>
                </ul>
            </li>
            <li><strong>Download this IPA:</strong>
                <ul>
                    <li>Tap the download button above</li>
                    <li>Save to Files app</li>
                </ul>
            </li>
            <li><strong>Install via AltStore:</strong>
                <ul>
                    <li>Open AltStore on iPhone</li>
                    <li>Tap "+" button</li>
                    <li>Select the downloaded IPA</li>
                    <li>Wait for installation</li>
                </ul>
            </li>
            <li><strong>Trust the app:</strong>
                <ul>
                    <li>Settings → General → VPN & Device Management</li>
                    <li>Trust your Apple ID</li>
                </ul>
            </li>
        </ol>
    </div>
    
    <p><small>Built on $(date)</small></p>
</body>
</html>
EOF

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SERVER RUNNING"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📱 On your iPhone (same WiFi):"
echo ""
if [ -n "$LOCAL_IP" ]; then
    echo "   http://$LOCAL_IP:8000"
else
    echo "   http://YOUR_IP:8000"
    echo "   (Run 'ip addr' to find your IP)"
fi
echo ""
echo "💻 On this computer:"
echo "   http://localhost:8000"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Instructions:"
echo "1. Open the URL above on your iPhone Safari"
echo "2. Tap the download button"
echo "3. Install using AltStore (see webpage for details)"
echo ""
echo "Press Ctrl+C to stop server"
echo ""

# Start Python HTTP server
python3 -m http.server 8000

# Cleanup
cd - > /dev/null
rm -rf "$TEMP_DIR"
