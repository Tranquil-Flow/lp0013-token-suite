#!/usr/bin/env bash
# scripts/demo-video.sh — Self-running LP-0013 demo for video recording.
#
# Usage:
#   bash scripts/demo-video.sh
#
# Designed for screen recording: large text, timed pauses, clear section
# headers.  You just hit Record, run this, and narrate over it.
#
# Requirements: cargo, rustc (lgs/spel/wallet NOT required — the on-chain
# section uses pre-recorded evidence from docs/LEZ_PROOF_LOG.md).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# ── Timing ────────────────────────────────────────────────────────────
# Seconds to pause between sections so you can narrate.
# Increase these if you read slowly; decrease if you're brisk.
PREFACE_PAUSE=${PREFACE_PAUSE:-4}
SECTION_PAUSE=${SECTION_PAUSE:-3}
COMMAND_PAUSE=${COMMAND_PAUSE:-2}
SCENE_PAUSE=${SCENE_PAUSE:-4}

# ── Helpers ───────────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[32m'
CYAN='\033[36m'
YELLOW='\033[33m'
RESET='\033[0m'

banner() {
  echo ""
  echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${BOLD}${CYAN}  $1${RESET}"
  echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

section() {
  echo ""
  echo -e "${BOLD}${YELLOW}▶ $1${RESET}"
  echo ""
}

cmd() {
  echo -e "${DIM}\$ $1${RESET}"
  sleep "$COMMAND_PAUSE"
  echo ""
}

pause() {
  sleep "${1:-$SECTION_PAUSE}"
}

show() {
  # Run a command and display output
  echo -e "${DIM}\$ $1${RESET}"
  echo ""
  sleep "$COMMAND_PAUSE"
  eval "$1" 2>&1 | sed "s/^/  /"
  echo ""
}

# ── Setup ─────────────────────────────────────────────────────────────
clear
echo ""
echo -e "${BOLD}  LP-0013 — Token Authorities${RESET}"
echo -e "${BOLD}  Logos Lambda Prize Submission${RESET}"
echo ""
echo -e "  Mint authority lifecycle for LEZ tokens:"
echo -e "  variable supply, authority rotation, permanent revocation."
echo ""
pause "$PREFACE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Part 1 — Local verification gate"
# ══════════════════════════════════════════════════════════════════════

section "Running prerequisite checks"
show "cargo --version"
show "rustc --version"
echo ""

section "Formatting check"
show "cargo fmt --all -- --check"
echo -e "  ${GREEN}✓ All formatted${RESET}"
pause

section "Running 31 unit tests"
show "cargo test --workspace 2>&1 | tail -20"
pause "$SCENE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Part 2 — Offline authority lifecycle demo"
# ══════════════════════════════════════════════════════════════════════

section "2a. Variable-supply token"
echo "  A mint with a designated authority that can mint more tokens."
echo "  We create, mint 100 units, rotate authority, mint 25 more,"
echo "  then permanently revoke — and show post-revoke mints are rejected."
echo ""
pause "$SCENE_PAUSE"

show "cargo run -p mint-cli -- demo-variable 2>&1"
pause "$SCENE_PAUSE"

section "2b. Fixed-supply token"
echo "  A token created with no authority from the start."
echo "  Every mint attempt is rejected immediately."
echo ""
pause "$SCENE_PAUSE"

show "cargo run -p mint-cli -- demo-fixed 2>&1"
pause "$SCENE_PAUSE"

section "2c. Config-PDA gated authority (RFP-001)"
echo "  A program-derived address acts as the authority."
echo "  This proves the authority library composes with"
echo "  the LEZ PDA mechanism."
echo ""
pause "$SCENE_PAUSE"

show "cargo run -p config-pda-gated 2>&1"
pause "$SCENE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Part 3 — IDL artifacts"
# ══════════════════════════════════════════════════════════════════════

section "Hand-written canonical IDL"
echo "  Three instructions: create_mint, mint_to, set_mint_authority."
echo "  Includes discriminators, declared errors, expanded Option types."
echo ""
show "cat idl/admin-authority.idl.json"
pause "$SCENE_PAUSE"

section "SPEL-generated IDL"
echo "  Real 'spel generate-idl' output from the same LEZ tag"
echo "  and SPEL revision used by LP-0017."
echo ""
show "cat idl/admin-authority.idl.spel-generated.json"
pause "$SCENE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Part 4 — On-chain proof (pre-recorded)"
# ══════════════════════════════════════════════════════════════════════

echo "  The same authority logic running on a live LEZ sequencer"
echo "  with real RISC0 proof execution under RISC0_DEV_MODE=0."
echo ""
pause "$SCENE_PAUSE"

section "RISC0_DEV_MODE=0 confirmed"
echo "  The sequencer was configured with risc0_dev_mode = false."
echo "  Every transaction was processed by the real Risc0 zkVM."
echo ""
pause "$SECTION_PAUSE"

section "Lifecycle transactions (2026-05-18 semantic rerun)"
echo ""
echo -e "  ${GREEN}[1] create_mint${RESET}"
echo "     tx: 7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63"
echo "     result: confirmed — mint PDA initialized, decimals=6, authority=Some(...)"
echo ""
echo -e "  ${GREEN}[2] mint_to(100)${RESET}"
echo "     tx: c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f"
echo "     result: confirmed — supply=100, holding balance=100"
echo ""
echo -e "  ${GREEN}[3] set_mint_authority(None)${RESET}"
echo "     tx: 756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c"
echo "     result: confirmed — current_authority persisted as None"
echo ""
echo -e "  ${YELLOW}[4] mint_to(post-revoke)${RESET}"
echo "     tx: 27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff"
echo "     result: rejected — not confirmed on chain"
echo ""
pause "$SCENE_PAUSE"

section "Independent re-verification (~22 minutes later)"
echo "  Re-submitted set_mint_authority against the revoked mint."
echo ""
echo "  Sequencer log:"
echo ""
echo -e "    ${YELLOW}Guest panicked: Program error 2008:${RESET}"
echo -e "    ${YELLOW}authority has been revoked${RESET}"
echo ""
echo "  That's the require_authority check, firing inside the"
echo "  guest body, observed live on chain."
echo ""
pause "$SCENE_PAUSE"

section "Final mint PDA state readback"
echo "  MintDefinition {"
echo "    authority: AuthorityInfo {"
echo "      authority_type: 0 (MintTokens),"
echo "      current_authority: None,"
echo "    },"
echo "    supply: 100,"
echo "    decimals: 6,"
echo "  }"
echo ""
echo "  Matches the expected post-revocation state exactly."
echo ""
pause "$SCENE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Part 5 — Compute cost"
# ══════════════════════════════════════════════════════════════════════

echo "  Risc0 zkVM executor time per operation (local LEZ devnet):"
echo ""
printf "  %-28s %10s\n" "create_mint" "8.38 ms"
printf "  %-28s %10s\n" "mint_to" "7.58 ms"
printf "  %-28s %10s\n" "set_mint_authority (rotate/revoke)" "6.76 ms"
echo ""
printf "  ${DIM}%-28s %10s${RESET}\n" "post-revoke mint_to (rejected)" "4.43 ms"
printf "  ${DIM}%-28s %10s${RESET}\n" "post-revoke set_authority (rejected)" "4.21 ms"
echo ""
echo "  Rejected operations cost ~50% less — execution halts at the"
echo "  authority guard before any state write. Deterministic rejection."
echo ""
pause "$SCENE_PAUSE"

# ══════════════════════════════════════════════════════════════════════
banner "Closing"
# ══════════════════════════════════════════════════════════════════════

echo "  Proven:"
echo ""
echo "    1. Offline Rust authority semantics — unit-tested and runnable."
echo "    2. IDL generated by the real SPEL framework, shipped alongside"
echo "       the hand-written canonical version."
echo "    3. Same semantics on the LEZ sequencer — four-transaction"
echo "       lifecycle confirmed, post-revocation guard fired live"
echo "       in the guest body."
echo ""
echo "  Outside scope: end-to-end CI against a LEZ sequencer,"
echo "  and any claim of Logos endorsement or audit."
echo ""
echo "  Thanks for evaluating LP-0013."
echo ""
echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

