#!/bin/bash

# Test script to verify deployment user setup
# Run this on your server as the deployment user

echo "🧪 Testing deployment setup..."
echo ""

# Test 1: User info
echo "1️⃣ User Information:"
echo "   Current user: $(whoami)"
echo "   Home directory: $HOME"
echo "   Shell: $SHELL"
echo ""

# Test 2: SSH setup
echo "2️⃣ SSH Configuration:"
if [ -d ~/.ssh ]; then
    echo "   ✓ SSH directory exists"
    echo "   Permissions: $(ls -ld ~/.ssh | awk '{print $1}')"
    if [ -f ~/.ssh/authorized_keys ]; then
        echo "   ✓ authorized_keys exists"
        echo "   Keys configured: $(wc -l < ~/.ssh/authorized_keys)"
    else
        echo "   ✗ authorized_keys missing"
    fi
else
    echo "   ✗ SSH directory missing"
fi
echo ""

# Test 3: Rust installation
echo "3️⃣ Rust Installation:"
if [ -f ~/.cargo/env ]; then
    source ~/.cargo/env
    if command -v rustc &> /dev/null; then
        echo "   ✓ Rust version: $(rustc --version)"
        echo "   ✓ Cargo version: $(cargo --version)"
    else
        echo "   ✗ Rust not found in PATH"
    fi
else
    echo "   ✗ Rust not installed (~/.cargo/env missing)"
fi
echo ""

# Test 4: Sudo permissions
echo "4️⃣ Sudo Permissions:"
if sudo -n systemctl status split-bills &> /dev/null; then
    echo "   ✓ Can run systemctl without password"
elif sudo -n systemctl list-units &> /dev/null; then
    echo "   ✓ Sudo works without password"
    echo "   ⚠️  split-bills service not installed yet (normal for first setup)"
else
    echo "   ✗ Cannot run sudo without password"
    echo "   Run: sudo visudo -f /etc/sudoers.d/split-bills-deploy"
fi
echo ""

# Test 5: Git
echo "5️⃣ Git Installation:"
if command -v git &> /dev/null; then
    echo "   ✓ Git version: $(git --version)"
else
    echo "   ✗ Git not installed"
fi
echo ""

# Test 6: Project directory
echo "6️⃣ Project Directory:"
PROJECT_DIRS=("/opt/split-bills" "$HOME/split-bills")
for dir in "${PROJECT_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "   ✓ Found: $dir"
        echo "     Owner: $(ls -ld "$dir" | awk '{print $3":"$4}')"
        echo "     Writable: $(test -w "$dir" && echo "Yes" || echo "No")"
        if [ -d "$dir/.git" ]; then
            echo "     Git repo: Yes"
            cd "$dir" && echo "     Branch: $(git branch --show-current)"
        fi
    fi
done
echo ""

# Summary
echo "📊 Summary:"
PASS=0
FAIL=0

[ -d ~/.ssh ] && ((PASS++)) || ((FAIL++))
[ -f ~/.cargo/env ] && ((PASS++)) || ((FAIL++))
command -v git &> /dev/null && ((PASS++)) || ((FAIL++))
sudo -n systemctl list-units &> /dev/null && ((PASS++)) || ((FAIL++))

echo "   Passed: $PASS checks"
echo "   Failed: $FAIL checks"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ All checks passed! Ready for deployment."
else
    echo "⚠️  Some checks failed. Review the output above."
fi
