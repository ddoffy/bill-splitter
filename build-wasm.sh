#!/bin/bash
# Build WASM module for offline bill calculation

set -e

echo "🦀 Building Split Bills WASM Module"
echo "===================================="
echo ""

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

echo "✅ wasm-pack installed"
echo ""

cd "$(dirname "$0")/wasm"

# Build WASM module
echo "🏗️  Building WASM module..."
wasm-pack build --target web --out-dir ../static/wasm --out-name split_bills

echo ""
echo "✅ WASM build complete!"
echo ""
echo "📦 Output files in static/wasm/:"
ls -la ../static/wasm/

echo ""
echo "📊 File sizes:"
du -h ../static/wasm/*.wasm

echo ""
echo "🎉 WASM module ready for use!"
echo ""
echo "The following files are generated:"
echo "  • static/wasm/split_bills.js       - JS bindings"
echo "  • static/wasm/split_bills_bg.wasm  - WASM binary"
echo ""
echo "Usage in JavaScript:"
echo "  import init, { calculate_split } from './wasm/split_bills.js';"
echo "  await init();"
echo "  const result = calculate_split(JSON.stringify(request));"
