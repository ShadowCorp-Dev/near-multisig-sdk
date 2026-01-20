#!/bin/bash
set -e

echo "Building basic multisig contract..."
cargo near build non-reproducible-wasm

echo "✓ Build complete!"
echo "WASM file: target/near/basic_multisig.wasm"
