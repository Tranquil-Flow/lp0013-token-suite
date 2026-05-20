#!/usr/bin/env bash
set -euo pipefail

printf '== LP-0013 offline authority demo ==\n'

printf '\n-- mint-cli demo-variable --\n'
cargo run -p mint-cli -- demo-variable

printf '\n-- mint-cli demo-fixed --\n'
cargo run -p mint-cli -- demo-fixed

printf '\n-- example: variable-supply --\n'
cargo run -p variable-supply

printf '\n-- example: fixed-supply --\n'
cargo run -p fixed-supply

printf '\n-- example: config-pda-gated --\n'
cargo run -p config-pda-gated

printf '\nOffline authority demo completed.\n'
