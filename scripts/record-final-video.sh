#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export TERM="${TERM:-xterm-256color}"
BOLD='[1m'; DIM='[2m'; GREEN='[32m'; CYAN='[36m'; YELLOW='[33m'; RESET='[0m'
SECTION_PAUSE="${SECTION_PAUSE:-4}"
COMMAND_PAUSE="${COMMAND_PAUSE:-1}"
SCENE_PAUSE="${SCENE_PAUSE:-5}"
RUN_COMMANDS="${RUN_COMMANDS:-1}"
pause(){ sleep "${1:-$SECTION_PAUSE}"; }
scene(){ printf '
%b════════════════════════════════════════════════════════════%b
' "$BOLD$CYAN" "$RESET"; printf '%b  %s%b
' "$BOLD$CYAN" "$1" "$RESET"; printf '%b════════════════════════════════════════════════════════════%b

' "$BOLD$CYAN" "$RESET"; pause "$SECTION_PAUSE"; }
say(){ printf '%s
' "$*"; }
cmd(){ printf '%b$ %s%b
' "$DIM" "$*" "$RESET"; if [[ "$RUN_COMMANDS" == "1" ]]; then bash -lc "$*"; else printf '  [dry display only]
'; fi; printf '
'; pause "$COMMAND_PAUSE"; }
soft(){ printf '%b$ %s%b
' "$DIM" "$*" "$RESET"; if [[ "$RUN_COMMANDS" == "1" ]]; then bash -lc "$*" || true; else printf '  [dry display only]
'; fi; printf '
'; pause "$COMMAND_PAUSE"; }

clear || true
scene "LP-0013 final resubmission demo — public testnet authority lifecycle"
say "This recording demonstrates the corrected LP-0013 token authority lifecycle on the public LEZ testnet."
say "The key fix is separate create_holding and mutable mint_to: two mints accumulate to supply one hundred, then revocation prevents future minting."
pause "$SCENE_PAUSE"
scene "1. Submission state and canonical evidence"
cmd "git log -1 --oneline"
cmd "python3 scripts/validate-submission-docs.py"
cmd "grep -n 'Program\|ImageID\|MintPDA\|supply=100\|authority=None' RESUBMISSION_STATUS.md docs/LEZ_PROOF_LOG.md | head -80"
scene "2. Public-testnet read-only verification"
say "This command queries the public sequencer. It should show deploy, create_mint, create_holding, mint_to sixty, mint_to forty, authority revocation, and post-revoke rejection."
cmd "bash scripts/demo-testnet-live.sh verify"
scene "3. Final reviewer summary"
say "LP-0013 is ready for resubmission once this fresh video URL is inserted."
say "The final state shown by the verifier is authority=None, supply=100, decimals=6, and the post-revoke mint did not land."
say "Do not cite the older June third run; this video shows the corrected June fourth public-testnet evidence."
printf '\n%bLP-0013 final video script complete.%b\n' "$GREEN" "$RESET"
