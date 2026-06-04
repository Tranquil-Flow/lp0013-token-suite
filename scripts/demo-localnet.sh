#!/usr/bin/env bash
# scripts/demo-localnet.sh — LP-0013 LOCAL-SEQUENCER lifecycle demo, reproducible from a clean clone.
#
# Builds the in-repo deployable program (onchain-program/), then deploys it to a
# local LEZ sequencer and drives the full *corrected* authority lifecycle:
#
#   create_mint -> create_holding -> mint_to(60) -> mint_to(40)   [accumulates to 100]
#               -> set_mint_authority(None) -> mint_to(post-revoke)   [rejected by the guard]
#
# then reads back the mint + holding PDAs (mint supply == 100, authority == None;
# holding balance == 100).
#
# This is the white-box counterpart to scripts/demo-testnet-live.sh: a local
# sequencer's logs expose the exact guest-panic string ("Program error 2008:
# authority has been revoked") that the public testnet hides.
#
# Prerequisites — the LEZ toolchain, which is HOST-ONLY (it cannot run in hosted
# CI: the RISC0 guest build needs cargo-risczero, and there is no sequencer or
# faucet on a GitHub runner). You need:
#   - cargo + a RISC0 toolchain (`cargo-risczero`)            — builds the guest
#   - lgs / logos-scaffold                                    — runs the local sequencer
#   - wallet (LEZ tag matching your localnet)                 — signs transactions
#   - a funded localnet signer account in your wallet home    — pays/signs the txs
#
# Usage:
#   bash scripts/demo-localnet.sh --check     # prerequisite check only (safe anywhere)
#   bash scripts/demo-localnet.sh             # build + deploy + drive the lifecycle
#
# Environment:
#   NSSA_SEQUENCER_URL    local sequencer URL   (default http://127.0.0.1:3040)
#   NSSA_WALLET_HOME_DIR  wallet home with a funded localnet signer (required for a real run)
#   LP0013_AUTHORITY      signer account id     (defaults to the localnet-seeded account)
#   RISC0_DEV_MODE        set to 0 for a real proof (this script defaults it to 0)
set -euo pipefail

BOLD='\033[1m'; DIM='\033[2m'; GREEN='\033[32m'; CYAN='\033[36m'; YELLOW='\033[33m'; RED='\033[31m'; RESET='\033[0m'

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROGRAM_DIR="${ROOT}/onchain-program"
SEQ_URL="${NSSA_SEQUENCER_URL:-http://127.0.0.1:3040}"
export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"

banner() { echo ""; echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; echo -e "${BOLD}${CYAN}  $1${RESET}"; echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"; echo ""; }
section() { echo ""; echo -e "${BOLD}${YELLOW}▶ $1${RESET}"; echo ""; }
have() { command -v "$1" >/dev/null 2>&1; }

check_prereqs() {
  local missing=0
  section "Toolchain prerequisites (host-only)"
  for tool in cargo wallet; do
    if have "$tool"; then echo -e "  ${GREEN}✓${RESET} $tool"; else echo -e "  ${RED}✗ $tool (missing)${RESET}"; missing=1; fi
  done
  if cargo risczero --version >/dev/null 2>&1 || have cargo-risczero; then
    echo -e "  ${GREEN}✓${RESET} cargo-risczero"
  else
    echo -e "  ${RED}✗ cargo-risczero (missing — needed to build the guest)${RESET}"; missing=1
  fi
  if have lgs; then echo -e "  ${GREEN}✓${RESET} lgs"; elif have logos-scaffold; then echo -e "  ${GREEN}✓${RESET} logos-scaffold"; else echo -e "  ${YELLOW}• lgs/logos-scaffold not found — start a local sequencer yourself${RESET}"; fi
  return $missing
}

if [[ "${1:-}" == "--check" ]]; then
  banner "LP-0013 local-sequencer demo — prerequisite check"
  if check_prereqs; then echo -e "\n${GREEN}All required tools present.${RESET}"; else echo -e "\n${YELLOW}Install the missing tools above before a real run.${RESET}"; fi
  exit 0
fi

banner "LP-0013 — local-sequencer authority lifecycle (corrected guest)"
echo "  Program dir : ${PROGRAM_DIR}"
echo "  Sequencer   : ${SEQ_URL}"
echo "  RISC0_DEV_MODE = ${RISC0_DEV_MODE}"

if ! check_prereqs; then
  echo -e "\n${RED}Missing required tools — see above. This demo needs the host LEZ toolchain.${RESET}"
  echo -e "${DIM}Run 'bash scripts/demo-localnet.sh --check' anywhere to re-check.${RESET}"
  exit 1
fi

if [[ -z "${NSSA_WALLET_HOME_DIR:-}" ]]; then
  echo -e "\n${RED}NSSA_WALLET_HOME_DIR is not set.${RESET} Point it at a wallet home holding a funded"
  echo "localnet signer (the account in LP0013_AUTHORITY). See docs/LEZ_PROOF_LOG.md."
  exit 1
fi

section "1. Build the RISC0 guest (onchain-program)"
echo -e "  ${DIM}\$ make -C onchain-program build${RESET}"
make -C "${PROGRAM_DIR}" build

section "2. Locate the built guest ELF"
ELF="$(find "${PROGRAM_DIR}/methods/guest/target" -name 'admin_authority_spike.bin' 2>/dev/null | head -1 || true)"
if [[ -z "${ELF}" ]]; then
  echo -e "  ${RED}Could not find admin_authority_spike.bin under methods/guest/target.${RESET}"
  echo "  Check the build output above (or run 'make -C onchain-program inspect')."
  exit 1
fi
export LP0013_PROGRAM_BIN="${ELF}"
echo -e "  ${GREEN}LP0013_PROGRAM_BIN=${ELF}${RESET}"

section "3. ProgramId for the built guest"
echo -e "  ${DIM}\$ make -C onchain-program inspect${RESET}"
make -C "${PROGRAM_DIR}" inspect || true

section "4. Deploy + drive the full lifecycle against the local sequencer"
echo "  This runs the in-repo LEZ client (onchain-program live_lifecycle): deploy,"
echo "  create_mint, create_holding, two accumulating mints (60 + 40 -> 100),"
echo "  set_mint_authority(None), and a post-revoke mint rejected by the guard,"
echo "  then reads back the mint + holding PDAs."
echo ""
( cd "${PROGRAM_DIR}" && NSSA_SEQUENCER_URL="${SEQ_URL}" cargo run --quiet --bin live_lifecycle )

banner "Done"
echo "  Expected: mint supply == 100 (60 + 40), authority == None; holding balance == 100;"
echo "  the post-revoke mint rejected. The local sequencer log shows the exact guest"
echo "  panic 'Program error 2008: authority has been revoked' for that rejection."
echo "  See docs/LEZ_PROOF_LOG.md for how to grep the executor log."
