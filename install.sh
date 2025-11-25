#!/bin/bash

# Exit on any error
set -e

# Get the absolute path of the current directory
PROJECT_DIR=$(pwd)
CURRENT_USER=$(whoami)

echo "🚀 Setting up Split Bills service..."
echo "📂 Project Directory: $PROJECT_DIR"
echo "👤 User: $CURRENT_USER"

# Update the service file with current paths and user
# This ensures it works even if the folder is moved or user is different
echo "📝 Updating service configuration..."
sed -i "s|WorkingDirectory=.*|WorkingDirectory=$PROJECT_DIR|g" split-bills.service
sed -i "s|ExecStart=.*|ExecStart=$PROJECT_DIR/target/release/split-bills|g" split-bills.service
sed -i "s|User=.*|User=$CURRENT_USER|g" split-bills.service

echo "🔨 Building release binary..."
cargo build --release

echo "📦 Installing systemd service..."
if [ -f /etc/systemd/system/split-bills.service ]; then
    echo "Stopping existing service..."
    sudo systemctl stop split-bills
fi

sudo cp split-bills.service /etc/systemd/system/

echo "🔄 Reloading systemd..."
sudo systemctl daemon-reload

echo "▶️ Enabling and starting service..."
sudo systemctl enable split-bills
sudo systemctl restart split-bills

echo "🛡️ Configuring firewall (UFW)..."
if command -v ufw >/dev/null; then
    sudo ufw allow 7777/tcp
    echo "Firewall rule added for port 7777."
else
    echo "UFW not found, skipping firewall step."
fi

# Get local IP address
IP_ADDR=$(hostname -I | awk '{print $1}')

echo "------------------------------------------------"
echo "✅ Installation complete!"
echo "The service is running at: http://$IP_ADDR:7777"
echo "Check status with: sudo systemctl status split-bills"
echo "------------------------------------------------"
