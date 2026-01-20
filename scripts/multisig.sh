#!/bin/bash

# Main wrapper script for multisig operations

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_help() {
    cat << EOF
Multisig Helper Script

Usage: ./multisig.sh <command> [args]

Commands:
  view-pending <multisig>              View pending transactions
  submit <multisig> <receiver> <amount> Submit new transfer transaction
  approve <multisig> <tx-id>           Approve a pending transaction

Examples:
  ./multisig.sh view-pending multisig.testnet
  ./multisig.sh submit multisig.testnet recipient.near 5
  ./multisig.sh approve multisig.testnet 0

Environment:
  NEAR_ACCOUNT - Your NEAR account (required for submit/approve)

EOF
}

if [ $# -eq 0 ]; then
    show_help
    exit 1
fi

COMMAND="$1"
shift

case "$COMMAND" in
    view-pending)
        "$SCRIPT_DIR/view-pending.sh" "$@"
        ;;
    submit)
        "$SCRIPT_DIR/submit.sh" "$@"
        ;;
    approve)
        "$SCRIPT_DIR/approve.sh" "$@"
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $COMMAND"
        echo ""
        show_help
        exit 1
        ;;
esac
