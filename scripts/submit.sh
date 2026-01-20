#!/bin/bash
set -e

# Usage: ./submit.sh <multisig-address> <receiver> <amount-in-near>

if [ $# -lt 3 ]; then
    echo "Usage: ./submit.sh <multisig-address> <receiver> <amount-in-near>"
    echo "Example: ./submit.sh multisig.testnet recipient.near 5"
    exit 1
fi

MULTISIG="$1"
RECEIVER="$2"
AMOUNT_NEAR="$3"

# Security: Validate inputs to prevent command injection (NM-4)
if ! [[ "$AMOUNT_NEAR" =~ ^[0-9]+\.?[0-9]*$ ]]; then
    echo "Error: Invalid amount. Must be a valid number."
    exit 1
fi

if [ -z "$NEAR_ACCOUNT" ]; then
    echo "Error: NEAR_ACCOUNT environment variable not set"
    echo "Set it with: export NEAR_ACCOUNT=your-account.near"
    exit 1
fi

# Convert NEAR to yoctoNEAR (1 NEAR = 10^24 yoctoNEAR)
AMOUNT_YOCTO=$(echo "$AMOUNT_NEAR * 1000000000000000000000000" | bc)

echo "Submitting transaction:"
echo "  From: $MULTISIG"
echo "  To: $RECEIVER"
echo "  Amount: $AMOUNT_NEAR NEAR"
echo "  Signed by: $NEAR_ACCOUNT"
echo ""

near call "$MULTISIG" submit_transaction "{
  \"receiver_id\": \"$RECEIVER\",
  \"actions\": [{
    \"Transfer\": {
      \"amount\": \"$AMOUNT_YOCTO\"
    }
  }]
}" --accountId "$NEAR_ACCOUNT"

echo ""
echo "✓ Transaction submitted!"
echo "Use ./view-pending.sh $MULTISIG to see pending transactions"
