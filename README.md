# near-multisig-sdk

**Build secure multisig wallets for NEAR in minutes.**

Complete multisig toolkit with web UIs, CLI tools, and smart contracts.

> **⚠️ Educational/Prototype Use:** This toolkit is provided for educational purposes and prototype development. While comprehensive security check has been performed, use in production environments is at your own risk. For high-value deployments, consider professional external audit.

## What You Get

**Three production-ready templates:**
- **Basic** - M-of-N approval (3 of 5 owners must approve)
- **Timelock** - Mandatory delay before execution (48-hour safety period)
- **Weighted** - Token-based voting power (governance)

**Each template includes:**
- ✅ Rust smart contract with full implementation
- ✅ Security-hardened (audited by ShadowCorp)
- ✅ Next.js web UI (connect wallet, approve transactions)
- ✅ Shell scripts for CLI users
- ✅ Complete documentation

**Plus:**
- Security checksums
- GitHub Actions auto-releases
- Build verification tools

## Security

All contracts have undergone security review and include protections against:
- ✅ Transaction ID overflow attacks
- ✅ Promise callback failures
- ✅ Weight calculation overflows
- ✅ Input validation exploits

See [SECURITY.md](SECURITY.md) for full details.

## Installation

```bash
cd near-multisig-sdk
cargo build --release

# Use the CLI from: target/release/near-multisig
```

## 60-Second Start

**1. Create a new multisig project:**

```bash
near-multisig init my-treasury
```

This creates a complete project with smart contract code, build config, and CI/CD workflow.

**2. Build it:**

```bash
cd my-treasury
near-multisig build
```

**3. Deploy it:**

```bash
# Your WASM file is ready at: release/my_treasury.wasm
near deploy --accountId your-account.near --wasmFile release/my_treasury.wasm
```

**4. Initialize with owners:**

```bash
near call your-account.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2
}' --accountId your-account.near
```

Done. You now have a working multisig contract.

**Next steps:** See [INITIALIZATION.md](INITIALIZATION.md) for complete usage guide including how to submit/approve transactions.

## Two Ways to Use

### Option 1: Quick CLI (Developers)

Generate a contract project. Fast and simple.

```bash
near-multisig init my-project
cd my-project
near-multisig build
```

**Creates:** Contract code, build config, GitHub Actions.

**Best for:** Developers who know NEAR CLI.

### Option 2: Full Template (Teams)

Copy complete template with web UI. Everything included.

```bash
# Copy template
cp -r templates/basic my-multisig
cd my-multisig

# Build contract
cd contract && ./build.sh

# Run web UI
cd ../frontend
npm install
npm run dev
# Open http://localhost:3000
```

**Includes:**
- Contract (Rust)
- Frontend (Next.js + wallet)
- Shell scripts (CLI helpers)
- Docs (deployment guide)

**Best for:** Teams wanting a web interface for non-technical users.

**See all templates:** [templates/README.md](templates/README.md)

## Templates Explained

### 1. Basic Multisig

**What:** M out of N owners must approve. Transaction must be explicitly executed after threshold reached.

**Use:** DAO treasuries, team wallets, shared custody.

**Example:** 5-person council, need 3 approvals to spend, then anyone executes.

**CLI:** `near-multisig init dao-treasury`
**Template:** `cp -r templates/basic dao-treasury`

**Web UI features:**
- View pending transactions
- Approve with one click
- Shows approval progress (2/3)

---

### 2. Timelock Multisig

**What:** After M approvals, mandatory delay before anyone can execute.

**Use:** Protocol upgrades, high-value transfers, security-critical operations.

**Example:** 2 approvals needed, then 48-hour delay before execution.

**CLI:** `near-multisig init upgrade --template timelock`
**Template:** `cp -r templates/timelock upgrade`

**Web UI features:**
- Pending tab (needs approvals)
- Scheduled tab (counting down)
- Execute button (after timelock expires)

---

### 3. Weighted Multisig

**What:** Voting power based on token holdings or stake. Executes when weight threshold reached.

**Use:** Token governance, equity voting, proportional control.

**Example:** Alice 50%, Bob 30%, Charlie 20%. Need 60% to pass.

**CLI:** `near-multisig init token-gov --template weighted`
**Template:** `cp -r templates/weighted token-gov`

**Web UI features:**
- Weight-based progress bars
- Shows each voter's weight
- Percentage complete (45/60 = 75%)

---

## Quick Comparison

| Feature | Basic | Timelock | Weighted |
|---------|-------|----------|----------|
| **Approval** | M-of-N count | M-of-N count | Weight threshold |
| **Execution** | Immediate | After delay | Immediate |
| **Best for** | DAOs, teams | Security | Governance |
| **Frontend** | ✅ Simple | ✅ 3 tabs + timer | ✅ Weight bars |
| **Scripts** | ✅ Yes | ✅ Yes | ✅ Yes |

## Commands

### `near-multisig init <name>`

Create a new multisig project.

**Options:**
- `--template` or `-t` - Choose template: `basic` (default), `timelock`, or `weighted`

**What it creates:**
- `src/lib.rs` - Your contract code
- `Cargo.toml` - Build configuration
- `.github/workflows/release.yml` - Auto-release workflow

### `near-multisig build`

Build your contract and generate verification files.

**Creates:**
- `release/your_contract.wasm` - Compiled contract
- `release/SHA256SUMS` - Security checksums
- `release/build-manifest.json` - Build details

### `near-multisig verify <dir>`

Verify checksums match.

```bash
near-multisig verify release/
# ✓ my_treasury.wasm (checksum matches)
```

## GitHub Auto-Releases

Every project includes GitHub Actions workflow. When you push a git tag, it automatically:
1. Builds your contract
2. Generates checksums
3. Creates a GitHub release
4. Uploads the files

**Usage:**

```bash
git init
git add .
git commit -m "Initial release"
git tag v1.0.0
git push origin main --tags
```

Your contract is now published on GitHub with verified checksums.

## Shell Scripts (CLI Alternative)

Don't want a web UI? Use shell scripts instead.

```bash
cd scripts
export NEAR_ACCOUNT=alice.near

# View pending
./multisig.sh view-pending multisig.testnet

# Submit transfer (5 NEAR)
./multisig.sh submit multisig.testnet recipient.near 5

# Approve transaction
./multisig.sh approve multisig.testnet 0
```

**Scripts work with all templates** (basic, timelock, weighted).

**Learn more:** [scripts/README.md](scripts/README.md)

## Real-World Examples

### DAO Treasury (3-of-5)

```bash
# Setup
near-multisig init dao-treasury
cd dao-treasury
near-multisig build

# Deploy
near deploy --accountId treasury.dao.near --wasmFile release/dao_treasury.wasm

# Initialize with 5 owners, need 3 approvals
near call treasury.dao.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near", "dave.near", "eve.near"],
  "num_confirmations": 3
}' --accountId treasury.dao.near
```

### Protocol Upgrade with 48h Timelock

```bash
# Setup with timelock template
near-multisig init protocol-upgrade --template timelock
cd protocol-upgrade
near-multisig build

# Deploy and initialize with 48-hour delay
near call upgrade.protocol.near new '{
  "owners": ["dev-team.near", "security.near"],
  "num_confirmations": 2,
  "timelock_duration": 172800000000000
}' --accountId upgrade.protocol.near

# After 2 approvals, must wait 48 hours before execution
```

### Token Governance (Weighted)

```bash
# Setup
near-multisig init token-gov --template weighted
cd token-gov
near-multisig build

# Initialize with different voting weights
near call gov.token.near new '{
  "owners_with_weights": [
    ["whale.near", 50],
    ["medium1.near", 25],
    ["medium2.near", 15],
    ["small.near", 10]
  ],
  "approval_threshold": 60
}' --accountId gov.token.near

# Need 60% weight to execute (e.g., whale + medium1 = 75%)
```

## How to Verify a Contract

If someone sends you a `.wasm` file and `SHA256SUMS`:

```bash
# Check the checksum
sha256sum -c SHA256SUMS
# ✓ contract.wasm: OK

# This proves the file hasn't been tampered with
```

## Development

**Project structure:**
```
near-multisig-sdk/
├── cli/          # The near-multisig command
├── lib/          # Shared utilities
└── README.md
```

**Build from source:**
```bash
git clone <repo>
cd near-multisig-sdk
cargo build --release
```

**Test it works:**
```bash
./target/release/near-multisig init test-project
cd test-project
../target/release/near-multisig build
../target/release/near-multisig verify release/
```

## What's Next

- [ ] Full reproducible builds (Docker-based)
- [ ] Publish to crates.io for `cargo install`
- [ ] Social recovery template
- [ ] More governance patterns

## License

MIT

---

**Documentation:**
- [INITIALIZATION.md](INITIALIZATION.md) - Complete deployment and usage guide
- [EXAMPLES.md](EXAMPLES.md) - Real-world patterns and advanced usage
- [SECURITY.md](SECURITY.md) - Security audit results and best practices
