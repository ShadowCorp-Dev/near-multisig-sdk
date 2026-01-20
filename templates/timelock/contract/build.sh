#!/bin/bash
set -e

echo "Building timelock multisig contract..."

# Build the contract
cargo near build

echo ""
echo "✓ Build complete!"
echo "WASM file: target/near/timelock_multisig.wasm"
echo ""
echo "Deploy with:"
echo "  near deploy --accountId your-multisig.testnet --wasmFile target/near/timelock_multisig.wasm"
