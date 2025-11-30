#!/bin/bash

# Setup script to configure passwordless sudo for deployment
# Run this ONCE on your server as root or with sudo

echo "🔧 Setting up passwordless sudo for split-bills deployment"
echo ""

# Get the deployment user
DEPLOY_USER="${1:-$USER}"
echo "👤 Deployment user: $DEPLOY_USER"

# Create sudoers file for split-bills service management
SUDOERS_FILE="/etc/sudoers.d/split-bills-deploy"

cat > "$SUDOERS_FILE" << EOF
# Allow $DEPLOY_USER to manage split-bills service without password
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl start split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl stop split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl restart split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl status split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl daemon-reload
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/cp * /etc/systemd/system/split-bills.service
EOF

# Set correct permissions (critical for sudoers files)
chmod 0440 "$SUDOERS_FILE"

# Validate the sudoers file
if visudo -c -f "$SUDOERS_FILE"; then
    echo "✅ Sudoers configuration created successfully at $SUDOERS_FILE"
    echo ""
    echo "The user '$DEPLOY_USER' can now run these commands without password:"
    echo "  - sudo systemctl start/stop/restart/status split-bills"
    echo "  - sudo systemctl daemon-reload"
    echo ""
    echo "🔒 Security: Only specific systemctl commands for split-bills are allowed."
else
    echo "❌ Error: Invalid sudoers configuration!"
    rm -f "$SUDOERS_FILE"
    exit 1
fi
