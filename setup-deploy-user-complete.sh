#!/bin/bash

# Script to create a deployment user for GitHub Actions CI/CD
# Run this on your server as root or with sudo

set -e

DEPLOY_USER="${1:-github-deploy}"
DEPLOY_HOME="/home/$DEPLOY_USER"
PROJECT_PATH="${2:-/opt/split-bills}"

echo "🚀 Creating deployment user for CI/CD"
echo "👤 User: $DEPLOY_USER"
echo "📂 Project path: $PROJECT_PATH"
echo ""

# Create the deploy user if it doesn't exist
if id "$DEPLOY_USER" &>/dev/null; then
    echo "✓ User $DEPLOY_USER already exists"
else
    echo "Creating user $DEPLOY_USER..."
    sudo useradd -m -s /bin/bash "$DEPLOY_USER"
    echo "✓ User $DEPLOY_USER created"
fi

# Setup SSH directory
echo "Setting up SSH access..."
sudo mkdir -p "$DEPLOY_HOME/.ssh"
sudo chmod 700 "$DEPLOY_HOME/.ssh"
sudo touch "$DEPLOY_HOME/.ssh/authorized_keys"
sudo chmod 600 "$DEPLOY_HOME/.ssh/authorized_keys"
sudo chown -R "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_HOME/.ssh"
echo "✓ SSH directory configured"

# Add SSH public key (you'll need to paste it when prompted)
echo ""
echo "📋 Paste the SSH PUBLIC KEY that will be used for deployment:"
echo "   (This is the public key pair of DEPLOY_SSH_KEY secret)"
echo "   Generate with: ssh-keygen -t ed25519 -C 'github-actions-deploy'"
read -r SSH_PUBLIC_KEY

if [ -n "$SSH_PUBLIC_KEY" ]; then
    echo "$SSH_PUBLIC_KEY" | sudo tee -a "$DEPLOY_HOME/.ssh/authorized_keys" > /dev/null
    echo "✓ SSH key added"
else
    echo "⚠️  No SSH key provided. You can add it later to $DEPLOY_HOME/.ssh/authorized_keys"
fi

# Install required packages
echo ""
echo "📦 Installing required packages..."
sudo apt-get update
sudo apt-get install -y git curl build-essential

# Install Rust for the deploy user
echo ""
echo "🦀 Installing Rust for $DEPLOY_USER..."
sudo -u "$DEPLOY_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
echo "✓ Rust installed"

# Create project directory
echo ""
echo "📁 Setting up project directory..."
if [ ! -d "$PROJECT_PATH" ]; then
    sudo mkdir -p "$PROJECT_PATH"
fi
sudo chown -R "$DEPLOY_USER:$DEPLOY_USER" "$PROJECT_PATH"
echo "✓ Project directory created: $PROJECT_PATH"

# Setup passwordless sudo for systemctl
echo ""
echo "🔒 Configuring passwordless sudo for service management..."
SUDOERS_FILE="/etc/sudoers.d/split-bills-deploy"

sudo tee "$SUDOERS_FILE" > /dev/null << EOF
# Allow $DEPLOY_USER to manage split-bills service without password
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl start split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl stop split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl restart split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl status split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl daemon-reload
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/systemctl enable split-bills
$DEPLOY_USER ALL=(ALL) NOPASSWD: /bin/cp * /etc/systemd/system/split-bills.service
EOF

sudo chmod 0440 "$SUDOERS_FILE"

if sudo visudo -c -f "$SUDOERS_FILE"; then
    echo "✓ Sudoers configuration created"
else
    echo "❌ Error: Invalid sudoers configuration!"
    sudo rm -f "$SUDOERS_FILE"
    exit 1
fi

# Clone repository (if not exists)
echo ""
echo "Would you like to clone the repository now? (y/n)"
read -r CLONE_REPO

if [ "$CLONE_REPO" = "y" ] || [ "$CLONE_REPO" = "Y" ]; then
    echo "Enter the Git repository URL (e.g., git@github.com:ddoffy/bill-splitter.git):"
    read -r REPO_URL
    
    if [ -n "$REPO_URL" ]; then
        sudo -u "$DEPLOY_USER" git clone "$REPO_URL" "$PROJECT_PATH" || echo "⚠️  Repository already exists or clone failed"
    fi
fi

# Summary
echo ""
echo "✅ Deployment user setup complete!"
echo ""
echo "📋 Next steps:"
echo ""
echo "1. Add these secrets to your GitHub repository:"
echo "   Settings → Secrets and variables → Actions → New repository secret"
echo ""
echo "   DEPLOY_HOST: your-server-hostname-or-ip"
echo "   DEPLOY_USER: $DEPLOY_USER"
echo "   DEPLOY_SSH_KEY: <paste the PRIVATE key from the key pair you generated>"
echo "   DEPLOY_PORT: 22 (or your SSH port)"
echo ""
echo "2. Add these variables:"
echo "   Settings → Secrets and variables → Actions → Variables tab"
echo ""
echo "   DEPLOY_ENABLED: true"
echo "   DEPLOY_PATH: $PROJECT_PATH"
echo ""
echo "3. If using Cloudflare Tunnel, configure it with:"
echo "   cloudflared tunnel create split-bills"
echo "   cloudflared tunnel route dns split-bills your-hostname"
echo ""
echo "4. Test SSH access from your local machine:"
echo "   ssh -i /path/to/private/key $DEPLOY_USER@your-server"
echo ""
echo "5. On first deployment, the deploy.sh will run install.sh automatically"
echo ""
