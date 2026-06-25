#!/usr/bin/env bash
# scripts/demo-testnet-live.sh — LP-0013 PUBLIC TESTNET evidence, reproducible from a clean clone.
#
# The LP-0013 program is deployed and its authority lifecycle is recorded on the
# public LEZ testnet (https://testnet.lez.logos.co/, real consensus, RISC0_DEV_MODE=0).
# This script re-verifies that on-chain evidence directly from the public sequencer.
#
# Two modes:
#
#   verify   (default)  Read-only. Re-queries the canonical program + transaction
#                       hashes + mint-PDA state straight from the public sequencer.
#                       Uses `wallet` when available; otherwise falls back to
#                       scripts/ci-verify-testnet.sh (curl + python3 only).
#                       No build, no faucet, no keys.
#
#   full                Fresh deploy + lifecycle from your own funded account.
#                       Requires the rc3 build tree (see docs/LEZ_PROOF_LOG.md) and a
#                       faucet-funded signer. Documented at the bottom of this script.
#
# Usage:
#   bash scripts/demo-testnet-live.sh            # verify mode
#   bash scripts/demo-testnet-live.sh verify
#   MODE=full bash scripts/demo-testnet-live.sh  # see full-mode instructions
#
# The `wallet` binary must be built from LEZ tag v0.1.2 (the public testnet's version):
#   git clone https://github.com/logos-blockchain/logos-execution-zone && cd logos-execution-zone
#   git checkout tags/v0.1.2
#   PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo install --path wallet --force

set -euo pipefail

BOLD='\033[1m'; DIM='\033[2m'; GREEN='\033[32m'; CYAN='\033[36m'; YELLOW='\033[33m'; RED='\033[31m'; RESET='\033[0m'

MODE="${1:-${MODE:-verify}}"

# ---- Canonical public-testnet evidence (2026-06-04, corrected four-instruction guest) ----
# Deployed + exercised on the public LEZ testnet with create_holding + mutable mint_to.
# Two accumulating mints (60+40 -> 100) prove variable supply works on chain. After
# authority revocation, the post-revoke mint is rejected and the final mint state remains
# authority=None, supply=100. In LEZ a program is content-addressed, so ProgramId ==
# ImageID; base58 form of the program (as `program_owner` on its PDAs / in the explorer):
# 4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF
SEQ_ADDR="https://testnet.lez.logos.co/"
PROGRAM_ID="32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce"
IMAGE_ID="32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce"
MINT_PDA="HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu"
AUTHORITY="B6Sa77taeQgQ3FXHP88wjs15sJw3EyfcRjnSAZKnYchb"
RECIPIENT="4yswbZaRR1HQt4a5HS4uN7nLvAwL1txHTMSXKo1WZH2S"

# step label | tx hash | expected verdict
TXS=(
  "deploy_program|5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0|Some(ProgramDeployment)"
  "create_mint|7d1dcb04b5f339b33f04a120b7334cf9802720d4a917e600becd62476e44da74|Some(Public)"
  "create_holding|520d080b833c7e4038a1aa214bba43a3fc97328e8f379a093b74ca3e32be5893|Some(Public)"
  "mint_to(60)|8c865d0184f55ce5a881e24c8c125cd3729c5f90a4b83d0484c8d1610f743f61|Some(Public)"
  "mint_to(40)|c63168b7f615221ab2425b2ba003d32183f4df2e482eb4203e4e216675993d21|Some(Public)"
  "set_mint_authority(None)|8c4b08b5c750c57d0dbb4e9f43c32b7c0f2627ce5508da85408e3aaf01f5a331|Some(Public)"
  "mint_to(post-revoke)|6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1|None (rejected)"
)

export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

banner() { echo ""; echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; echo -e "${BOLD}${CYAN}  $1${RESET}"; echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; echo ""; }
section() { echo ""; echo -e "${BOLD}${YELLOW}▶ $1${RESET}"; echo ""; }

need_wallet() {
  if ! command -v wallet >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

# Set up an isolated, throwaway wallet home pointed at the public testnet, unless the
# caller already provided one. Read-only chain queries need no keys and no funding.
setup_wallet_home() {
  if [[ -n "${NSSA_WALLET_HOME_DIR:-}" ]]; then
    echo -e "  using provided NSSA_WALLET_HOME_DIR=${NSSA_WALLET_HOME_DIR}"
    return
  fi
  local home; home="$(mktemp -d -t lp0013-verify-XXXX)"
  cat > "$home/wallet_config.json" <<JSON
{
  "sequencer_addr": "${SEQ_ADDR}",
  "seq_poll_timeout": "12s",
  "seq_tx_poll_max_blocks": 5,
  "seq_poll_max_retries": 5,
  "seq_block_poll_max_amount": 100
}
JSON
  export NSSA_WALLET_HOME_DIR="$home"
  echo -e "  created throwaway wallet home: ${home}"
  echo -e "  sequencer_addr = ${SEQ_ADDR}"
}

verify_mode() {
  banner "LP-0013 — public-testnet evidence re-verification (read-only)"
  echo -e "  ${GREEN}${BOLD}✓ Token authority lifecycle:${RESET} ${DIM}create_holding + mutable mint_to."
  echo -e "  Two accumulating mints (60+40 -> 100) prove variable supply works on chain."
  echo -e "  After authority revocation, the post-revoke mint is rejected and final supply remains 100.${RESET}"
  echo ""
  echo "  Network : ${SEQ_ADDR} (real consensus, RISC0_DEV_MODE=0)"
  echo "  Program : ${PROGRAM_ID}"
  echo "  ImageID : ${IMAGE_ID}"
  echo "  MintPDA : ${MINT_PDA}"
  echo ""

  if [[ "${LP0013_FORCE_CURL_VERIFY:-0}" == "1" ]] || ! need_wallet; then
    echo -e "  ${YELLOW}using curl-only JSON-RPC verifier (wallet unavailable or LP0013_FORCE_CURL_VERIFY=1).${RESET}"
    echo -e "  ${DIM}\$ bash scripts/ci-verify-testnet.sh${RESET}"
    bash "$(dirname "$0")/ci-verify-testnet.sh"
    return
  fi

  setup_wallet_home

  section "Sequencer reachable; current block id"
  echo -e "  ${DIM}\$ wallet chain-info current-block-id${RESET}"
  wallet chain-info current-block-id < /dev/null 2>&1 | tail -2 | sed 's/^/  /' || {
    echo -e "  ${RED}could not reach the public sequencer${RESET}"; exit 1; }

  section "Per-transaction verdicts (queried live from the public sequencer)"
  local fail=0
  for entry in "${TXS[@]}"; do
    IFS='|' read -r label hash expected <<< "$entry"
    out="$(wallet chain-info transaction --hash "$hash" < /dev/null 2>&1 || true)"
    # Classify with pure bash pattern matching (no pipe → no SIGPIPE under pipefail).
    if [[ "$out" == *"Transaction is None"* || "$out" == None* ]]; then verdict="None (rejected)"
    elif [[ "$out" == *ProgramDeployment* ]]; then verdict="Some(ProgramDeployment)"
    elif [[ "$out" == *PublicTransaction* || "$out" == *"Public("* ]]; then verdict="Some(Public)"
    else verdict="${out%%$'\n'*}"; fi
    if [[ "$verdict" == "$expected" ]]; then
      printf "  ${GREEN}✓${RESET} %-26s %s  ${DIM}(%s…)${RESET}\n" "$label" "$verdict" "${hash:0:8}"
    else
      printf "  ${RED}✗${RESET} %-26s got '%s' expected '%s'  ${DIM}(%s…)${RESET}\n" "$label" "$verdict" "$expected" "${hash:0:8}"
      fail=1
    fi
  done

  section "Mint PDA state (the revocation invariant, by pure chain read)"
  echo -e "  ${DIM}\$ wallet account get --account-id Public/${MINT_PDA} --raw${RESET}"
  raw_json="$(wallet account get --account-id "Public/${MINT_PDA}" --raw < /dev/null 2>&1 | grep -o '{.*}' | tail -1 || true)"
  echo "  ${raw_json}"
  data_hex="$(sed -n 's/.*"data":"\([0-9a-fA-F]*\)".*/\1/p' <<< "$raw_json")"
  if [[ -n "$data_hex" ]]; then
    if python3 - "$data_hex" <<'PY'
import sys
raw = bytes.fromhex(sys.argv[1])
atype = raw[0]; opt = raw[1]
supply = int.from_bytes(raw[2:18], "little"); decimals = raw[18]
auth = "None" if opt == 0 else "Some(...)"
print(f"  decoded MintDefinition: authority_type={atype}  current_authority={auth}  supply={supply}  decimals={decimals}")
ok = (opt == 0 and supply == 100 and decimals == 6)
print("  " + ("\033[32m✓ revocation invariant holds: authority=None, supply=100 (post-revoke +7 never landed)\033[0m"
              if ok else "\033[31m✗ unexpected mint state\033[0m"))
sys.exit(0 if ok else 1)
PY
    then :; else fail=1; fi
  else
    echo -e "  ${RED}could not read mint PDA data${RESET}"; fail=1
  fi

  echo ""
  if [[ "$fail" == "0" ]]; then
    echo -e "${BOLD}${GREEN}  ✓ All public-testnet evidence re-verified live.${RESET}"
    echo "  Full proof log: docs/LEZ_PROOF_LOG.md"
  else
    echo -e "${BOLD}${RED}  ✗ One or more checks did not match.${RESET}"
    echo "  Note: the public testnet may have been reset since 2026-06-04; see docs/LEZ_PROOF_LOG.md."
    exit 1
  fi
}

full_mode() {
  banner "LP-0013 — full fresh deploy + lifecycle on the public testnet"
  cat <<EOF
  This mode deploys a fresh copy and runs the lifecycle from your own funded account.
  It is documented rather than auto-run because it needs the rc3 build tree and a faucet.

  1) Build the rc3 / testnet-matching guest (docker required for cargo risczero):
       see docs/LEZ_PROOF_LOG.md "Version-pin landmine" — pin spel/spel-framework to
       rev 31e52c52 and nssa* to tag v0.2.0-rc3, then:
       PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 \\
         cargo risczero build --manifest-path methods/guest/Cargo.toml

  2) Point a wallet at the public testnet and fund a signer:
       export NSSA_WALLET_HOME_DIR=\$(mktemp -d)
       wallet config set sequencer_addr ${SEQ_ADDR}
       wallet account new                       # note the Public/<id>
       wallet auth-transfer init --account-id Public/<id>     # initialize it first
       wallet pinata claim --to Public/<id>                   # faucet (solves a small PoW)
       wallet check-health                       # ✅ All looks good!

  3) Run the lifecycle driver against the testnet:
       export LP0013_PROGRAM_BIN=.../docker/admin_authority_spike.bin
       export LP0013_AUTHORITY=<id>              # the funded account from step 2
       export LP0013_POLL_MS=1500 LP0013_POLL_ATTEMPTS=160   # generous; testnet can take minutes
       cargo run -p admin_authority_spike-examples --bin live_lifecycle

  The driver deploys (capturing the hash the CLI discards), then runs create_mint /
  create_holding / mint_to(60) / mint_to(40) [accumulates -> 100] / set_mint_authority(None) /
  post-revoke mint_to (guard-rejected), and reads back the mint PDA (supply=100, authority=None)
  and the holding PDA (balance=100). Then re-verify with:  bash scripts/demo-testnet-live.sh verify
EOF
}

case "$MODE" in
  verify) verify_mode ;;
  full)   full_mode ;;
  *) echo "unknown mode '$MODE' (use: verify | full)"; exit 2 ;;
esac
