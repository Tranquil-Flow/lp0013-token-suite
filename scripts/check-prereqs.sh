#!/usr/bin/env bash
set -euo pipefail

missing=0

check_cmd() {
  local cmd="$1"
  if command -v "$cmd" >/dev/null 2>&1; then
    printf 'ok: %s -> %s\n' "$cmd" "$(command -v "$cmd")"
  else
    printf 'missing: %s\n' "$cmd" >&2
    missing=1
  fi
}

check_cmd cargo
check_cmd rustc

if command -v cargo >/dev/null 2>&1; then
  cargo --version
  rustc --version
fi

printf '\nChecking submission docs...\n'
python3 scripts/validate-submission-docs.py

printf '\nChecking workspace gates...\n'
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace

if cargo clippy --version >/dev/null 2>&1; then
  cargo clippy --workspace --all-targets -- -D warnings
else
  printf 'warn: cargo clippy unavailable; install clippy for final submission gate\n' >&2
fi

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

printf '\nPrerequisite check passed.\n'
