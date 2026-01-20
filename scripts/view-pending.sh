#!/bin/bash
set -e

# Usage: ./view-pending.sh <multisig-address>

if [ $# -eq 0 ]; then
    echo "Usage: ./view-pending.sh <multisig-address>"
    echo "Example: ./view-pending.sh multisig.testnet"
    exit 1
fi

MULTISIG="$1"

echo "Fetching pending transactions from $MULTISIG..."
echo ""

near view "$MULTISIG" get_pending_transactions --args '{}' | jq '.'
