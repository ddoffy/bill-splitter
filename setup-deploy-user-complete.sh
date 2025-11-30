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
echo "📋 SSH Key Setup"
echo "   Option 1: Paste the SSH PUBLIC KEY directly"
echo "   Option 2: Provide path to public key file"
echo ""
echo "Choose option (1 or 2):"
read -r KEY_OPTION

if [ "$KEY_OPTION" = "2" ]; then
    echo "Enter the path to your public key file:"
    read -r KEY_PATH
    
    if [ -f "$KEY_PATH" ]; then
        SSH_PUBLIC_KEY=$(cat "$KEY_PATH")
        echo "✓ Public key loaded from $KEY_PATH"
    else
        echo "⚠️  File not found: $KEY_PATH"
        echo "Enter the SSH PUBLIC KEY manually:"
        read -r SSH_PUBLIC_KEY
    fi
else
    echo "Paste the SSH PUBLIC KEY:"
    echo "   (Generate with: ssh-keygen -t ed25519 -C 'github-actions-deploy')"
    read -r SSH_PUBLIC_KEY
fi

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
echo "ℹ️  No server restart required!"
echo "   - User and SSH are ready immediately"
echo "   - Sudo permissions are active"
echo "   - You can test SSH connection right away"
echo ""
echo "🧪 Test the setup:"
echo ""
echo "1. Test SSH connection (from your local machine):"
echo "   ssh -i /path/to/private/key $DEPLOY_USER@your-server-hostname"
echo ""
echo "2. Test sudo permissions (after SSH):"
echo "   sudo systemctl status split-bills"
echo "   (Should not ask for password)"
echo ""
echo "3. Test Rust installation (after SSH):"
echo "   source ~/.cargo/env"
echo "   rustc --version"
echo "   cargo --version"
echo ""
echo "4. Test GitHub Actions deployment:"
echo "   - Push a commit to main branch"
echo "   - Check Actions tab in GitHub repository"
echo "   - Verify deployment runs successfully"
echo ""
echo "📋 Next steps for GitHub Actions:"
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
