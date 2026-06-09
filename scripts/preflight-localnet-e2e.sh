#!/usr/bin/env bash
# Preflight for the LP-0013 standalone LEZ local-sequencer e2e path.
#
# This script is intentionally safe for hosted CI: it reports exactly which
# Logos/RISC0/local-wallet prerequisites are present without starting a
# sequencer, spending funds, or requiring private keys. A real e2e run remains
# `bash scripts/demo-localnet.sh` on a host that passes this preflight.
set -euo pipefail

mode="${1:---report}"
case "$mode" in
  --report|--check-only|--real-run) ;;
  *)
    cat >&2 <<'USAGE'
usage: bash scripts/preflight-localnet-e2e.sh [--report|--check-only|--real-run]

  --report      Print all detected/missing prerequisites; always exits 0.
  --check-only  Require toolchain prerequisites; do not require wallet home.
  --real-run    Require toolchain prerequisites plus NSSA_WALLET_HOME_DIR.
USAGE
    exit 2
    ;;
esac

missing_tools=0
missing_real=0

say() { printf '%s\n' "$*"; }
ok() { printf 'ok: %s\n' "$*"; }
miss_tool() { printf 'missing: %s\n' "$*"; missing_tools=1; }
miss_real() { printf 'missing-real-run: %s\n' "$*"; missing_real=1; }

have() { command -v "$1" >/dev/null 2>&1; }

say "LP-0013 standalone LEZ local-sequencer e2e preflight"
say "mode: $mode"
say "RISC0_DEV_MODE=${RISC0_DEV_MODE:-0}"
say "NSSA_SEQUENCER_URL=${NSSA_SEQUENCER_URL:-http://127.0.0.1:3040}"
say ""

if have cargo; then
  ok "cargo -> $(command -v cargo)"
  cargo --version || true
else
  miss_tool "cargo (Rust toolchain)"
fi

if have rustc; then
  ok "rustc -> $(command -v rustc)"
  rustc --version || true
else
  miss_tool "rustc (Rust toolchain)"
fi

if have cargo && cargo risczero --version >/dev/null 2>&1; then
  ok "cargo risczero -> $(cargo risczero --version 2>/dev/null | head -1)"
elif have cargo-risczero; then
  ok "cargo-risczero -> $(command -v cargo-risczero)"
  cargo-risczero --version || true
else
  miss_tool "cargo-risczero (needed to build the RISC0 guest)"
fi

if have wallet; then
  ok "wallet -> $(command -v wallet)"
  wallet --version || true
else
  miss_tool "wallet (LEZ wallet binary, tag matching localnet/testnet)"
fi

if have lgs; then
  ok "lgs -> $(command -v lgs)"
  lgs --version || true
elif have logos-scaffold; then
  ok "logos-scaffold -> $(command -v logos-scaffold)"
  logos-scaffold --version || true
else
  miss_tool "lgs or logos-scaffold (needed to run/start a standalone local sequencer)"
fi

if [[ -n "${NSSA_WALLET_HOME_DIR:-}" ]]; then
  if [[ -d "$NSSA_WALLET_HOME_DIR" ]]; then
    ok "NSSA_WALLET_HOME_DIR exists -> $NSSA_WALLET_HOME_DIR"
  else
    miss_real "NSSA_WALLET_HOME_DIR is set but is not a directory: $NSSA_WALLET_HOME_DIR"
  fi
else
  miss_real "NSSA_WALLET_HOME_DIR (required only for a real funded localnet run)"
fi

if [[ -n "${LP0013_AUTHORITY:-}" ]]; then
  ok "LP0013_AUTHORITY set"
else
  say "info: LP0013_AUTHORITY unset; demo-localnet.sh will use its default localnet-seeded account"
fi

say ""
say "Summary:"
if [[ "$missing_tools" -eq 0 ]]; then
  ok "all standalone-localnet toolchain prerequisites present"
else
  say "standalone-localnet toolchain prerequisites are incomplete"
fi

if [[ "$missing_real" -eq 0 ]]; then
  ok "real-run wallet prerequisites present"
else
  say "real-run wallet prerequisites are incomplete"
fi

case "$mode" in
  --report)
    exit 0
    ;;
  --check-only)
    exit "$missing_tools"
    ;;
  --real-run)
    if [[ "$missing_tools" -ne 0 || "$missing_real" -ne 0 ]]; then
      exit 1
    fi
    ;;
esac
