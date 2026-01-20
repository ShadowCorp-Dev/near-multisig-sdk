#!/bin/bash
set -e

# Usage: ./approve.sh <multisig-address> <tx-id>

if [ $# -lt 2 ]; then
    echo "Usage: ./approve.sh <multisig-address> <tx-id>"
    echo "Example: ./approve.sh multisig.testnet 0"
    exit 1
fi

MULTISIG="$1"
TX_ID="$2"

# Security: Validate TX_ID is a number (NM-4)
if ! [[ "$TX_ID" =~ ^[0-9]+$ ]]; then
    echo "Error: Invalid transaction ID. Must be a number."
    exit 1
fi

if [ -z "$NEAR_ACCOUNT" ]; then
    echo "Error: NEAR_ACCOUNT environment variable not set"
    echo "Set it with: export NEAR_ACCOUNT=your-account.near"
    exit 1
fi

echo "Approving transaction #$TX_ID on $MULTISIG as $NEAR_ACCOUNT..."
echo ""

near call "$MULTISIG" confirm_transaction "{\"tx_id\": $TX_ID}" --accountId "$NEAR_ACCOUNT"

echo ""
echo "✓ Approval submitted!"
