#!/bin/bash
set -e

echo "Building weighted multisig contract..."

# Build the contract
cargo near build

echo ""
echo "✓ Build complete!"
echo "WASM file: target/near/weighted_multisig.wasm"
echo ""
echo "Deploy with:"
echo "  near deploy --accountId your-multisig.testnet --wasmFile target/near/weighted_multisig.wasm"
