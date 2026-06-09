#!/usr/bin/env bash
# scripts/demo-video-live.sh — LP-0013 live LEZ demo for recording.
#
# This is the "actual demo" companion to demo-video.sh: it starts a LEZ
# localnet, deploys the LP-0013 SPEL/RISC0 program, submits the authority
# lifecycle transactions, and reads back final on-chain mint state.
#
# Run on the M4 Pro:
#   cd ~/Projects/logos-basecamp/lp-0013-token-authorities/token-suite
#   bash scripts/demo-video-live.sh

set -euo pipefail

BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[32m'
CYAN='\033[36m'
YELLOW='\033[33m'
RESET='\033[0m'

PREFACE_PAUSE=${PREFACE_PAUSE:-4}
SECTION_PAUSE=${SECTION_PAUSE:-3}
COMMAND_PAUSE=${COMMAND_PAUSE:-2}
SCENE_PAUSE=${SCENE_PAUSE:-4}

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIVE_SPIKE_DIR=${LIVE_SPIKE_DIR:-"$HOME/.cache/lp0013-live-spike/admin_authority_spike"}
LP0017_DIR=${LP0017_DIR:-"$HOME/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower"}
PROGRAM_BIN=${LP0013_PROGRAM_BIN:-"$LIVE_SPIKE_DIR/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin"}
WALLET_DIR=${NSSA_WALLET_HOME_DIR:-"$LP0017_DIR/.scaffold/wallet"}

export PATH="$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"
export NSSA_WALLET_HOME_DIR="$WALLET_DIR"
export LP0013_PROGRAM_BIN="$PROGRAM_BIN"
export RISC0_DEV_MODE=0
if [[ -z "${TERM:-}" || "${TERM:-}" == "dumb" ]]; then
  export TERM=xterm-256color
fi

pause() { sleep "${1:-$SECTION_PAUSE}"; }

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
  echo ""
  sleep "$COMMAND_PAUSE"
}

show() {
  echo -e "${DIM}\$ $1${RESET}"
  echo ""
  sleep "$COMMAND_PAUSE"
  eval "$1" 2>&1 | sed 's/^/  /'
  echo ""
}

cleanup() {
  if [[ "${KEEP_LOCALNET:-0}" != "1" && -d "$LP0017_DIR" ]]; then
    (cd "$LP0017_DIR" && lgs localnet stop >/dev/null 2>&1 || true)
  fi
}
trap cleanup EXIT

require_file() {
  if [[ ! -f "$1" ]]; then
    echo -e "${YELLOW}Missing required file:${RESET} $1"
    echo "The live scaffold/cache has not been prepared on this machine."
    exit 1
  fi
}

require_dir() {
  if [[ ! -d "$1" ]]; then
    echo -e "${YELLOW}Missing required directory:${RESET} $1"
    exit 1
  fi
}

clear

echo ""
echo -e "${BOLD}  LP-0013 — Live LEZ Authority Lifecycle${RESET}"
echo -e "${BOLD}  Actual on-chain/localnet demo, RISC0_DEV_MODE=0${RESET}"
echo ""
echo "  This run starts a LEZ sequencer, deploys the LP-0013 RISC0 guest"
echo "  program, executes the mint-authority lifecycle, and reads back state."
echo ""
pause "$PREFACE_PAUSE"

banner "Part 1 — Live demo prerequisites"

section "Checking cached live scaffold and program ELF"
require_dir "$LIVE_SPIKE_DIR"
require_file "$PROGRAM_BIN"
require_dir "$LP0017_DIR"
require_dir "$WALLET_DIR"
show "cargo --version"
show "wallet --help | head -20"
show "ls -lh '$PROGRAM_BIN'"

echo -e "  ${GREEN}✓ LP-0013 program ELF is ready${RESET}"
echo -e "  ${GREEN}✓ Wallet/scaffold environment is ready${RESET}"
echo ""
pause "$SCENE_PAUSE"

banner "Part 2 — Start LEZ localnet"

section "Starting sequencer with real RISC0 proof mode"
echo "  RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "  This means the sequencer uses the real Risc0 zkVM path, not dev-mode bypass."
echo ""
cmd "cd '$LP0017_DIR' && lgs localnet reset --yes"
(
  cd "$LP0017_DIR"
  lgs localnet reset --yes
) > /tmp/lp0013-localnet-reset.log 2>&1
sed 's/^/  /' /tmp/lp0013-localnet-reset.log
echo ""
pause "$SCENE_PAUSE"

section "Sequencer health"
show "tail -20 '$LP0017_DIR/.scaffold/logs/sequencer.log'"
pause "$SCENE_PAUSE"

banner "Part 3 — Deploy LP-0013 program"

section "Deploying the RISC0 guest ELF"
echo "  The program id is derived from the compiled guest binary."
echo "  After deployment, the sequencer recognizes LP-0013 transactions."
echo ""
cmd "wallet deploy-program '$PROGRAM_BIN'"
wallet deploy-program "$PROGRAM_BIN" 2>&1 | sed 's/^/  /'
echo ""
echo "  Waiting for the next block so deployment is indexed..."
sleep 18
echo -e "  ${GREEN}✓ Deploy step submitted${RESET}"
echo ""
pause "$SCENE_PAUSE"

banner "Part 4 — Execute authority lifecycle live"

section "Submitting create, mint, revoke, and rejected post-revoke mint"
echo "  The driver signs real transactions with the seeded local wallet."
echo "  Expected result: three confirmations, then deterministic rejection."
echo ""
cmd "cd '$LIVE_SPIKE_DIR' && RISC0_DEV_MODE=0 cargo run -p admin_authority_spike-examples --bin live_lifecycle"
(
  cd "$LIVE_SPIKE_DIR"
  RISC0_DEV_MODE=0 cargo run -p admin_authority_spike-examples --bin live_lifecycle
) 2>&1 | tee /tmp/lp0013-live-demo-recording.log | sed 's/^/  /'
echo ""
pause "$SCENE_PAUSE"

banner "Part 5 — Evidence in sequencer log"

section "Recent proof/execution log lines"
show "tail -80 '$LP0017_DIR/.scaffold/logs/sequencer.log' | grep -E 'execution time|Block with id|failed execution|Guest panicked|Created block' | tail -30 || true"

echo -e "  ${GREEN}✓ Live demo complete${RESET}"
echo ""
echo "  What the evaluator just saw:"
echo ""
echo "    1. LP-0013 RISC0 guest ELF deployed to LEZ localnet."
echo "    2. create_mint confirmed."
echo "    3. mint_to(100) confirmed."
echo "    4. set_mint_authority(None) confirmed."
echo "    5. post-revoke mint rejected."
echo "    6. post-revoke authority restoration rejected, and final state reads authority=None."
echo ""
echo "  This demonstrates the corrected local-sequencer evidence path."
echo ""
echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
