#!/usr/bin/env bash
# scripts/ci-verify-testnet.sh — wallet-free public LEZ testnet verifier for LP-0013.
#
# Read-only, no keys, no wallet build, no localnet, no mock. Re-queries the
# current 2026-06-27 v0.2.0 authority lifecycle and final account state from
# https://testnet.lez.logos.co/ using JSON-RPC only.
set -euo pipefail

SEQ="${LP0013_SEQUENCER:-https://testnet.lez.logos.co/}"
PROGRAM_ID="338865e9549b18fb736020eaef87d5e20075b4250e10c00e08ea918c4871554a"
MINT_PDA="4gMBXeUskbUTzxoP8fJJEXj3jxTQz91m6ZW7fMsLMJq6"
HOLDING_PDA="366n7Nj21EzD27BXRKE2hFDWPtJ1E2Fcx9RmqQoGRD7h"

TXS=(
  "deploy_program|793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce|present"
  "create_mint|55908821088c98e898c4ef99e9a36e02856092f7afd0155f3457c25c5cf67746|present"
  "create_holding|8a37a8fb7200856c57d199ce081f2b744ed3cbaeec8326c83092f5ca05ac668f|present"
  "mint_to(60)|daf5aa91f35dff8250794c0dcfe932de473c651bd25c946d76f09a42cfdb6a97|present"
  "mint_to(40)|ed07b29c004a796d504814ddf1a9a0cfda373d1618398b620e330ccb529b3cce|present"
  "set_mint_authority(None)|719123f918df2aee42c4e69d36ba8860807b2a69c97a2927097d8313a508550e|present"
  "mint_to(post-revoke)|016043771c0cc60efaf158ec120a9bf341326967c881285878469503ddd3d4fa|null"
)

rpc() {
  local method="$1" params="$2"
  curl -s --max-time 30 -X POST "$SEQ" -H 'content-type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}"
}

echo "Verifying LP-0013 corrected authority lifecycle on $SEQ (read-only)"
echo "  program=$PROGRAM_ID"
fail=0
tmpdir="$(mktemp -d -t lp0013-ci-verify-XXXX)"
trap 'rm -rf "$tmpdir"' EXIT
for entry in "${TXS[@]}"; do
  IFS='|' read -r label hash expected <<< "$entry"
  resp="$(rpc getTransaction "[\"$hash\"]" || true)"
  resp_file="$tmpdir/tx-${hash}.json"
  printf '%s' "$resp" > "$resp_file"
  if python3 - "$expected" "$resp_file" <<'PY' 2>/dev/null
import json, sys
expected = sys.argv[1]
path = sys.argv[2]
try:
    result = json.load(open(path, "r", encoding="utf-8")).get("result")
except Exception:
    sys.exit(1)
ok = (isinstance(result, str) and len(result) > 0) if expected == "present" else (result is None)
sys.exit(0 if ok else 1)
PY
  then
    echo "  ok   $label  ($hash)"
  else
    echo "  FAIL $label  ($hash) -> $(printf '%s' "$resp" | head -c 200)"
    fail=1
  fi
done

echo "Verifying final mint PDA state"
acct_resp="$(rpc getAccount "[\"$MINT_PDA\"]" || true)"
acct_file="$tmpdir/mint.json"
printf '%s' "$acct_resp" > "$acct_file"
if python3 - "$MINT_PDA" "$acct_file" <<'PY'
import json, sys
pda = sys.argv[1]
path = sys.argv[2]
resp = json.load(open(path, "r", encoding="utf-8"))
result = resp.get("result")
if not isinstance(result, dict):
    print(f"  FAIL {pda}: missing account result -> {resp}")
    sys.exit(1)
data = result.get("data")
if not isinstance(data, list):
    print(f"  FAIL {pda}: missing byte-array data -> {result}")
    sys.exit(1)
raw = bytes(data)
if len(raw) >= 19:
    authority_type = raw[0]
    authority_option = raw[1]
    supply = int.from_bytes(raw[2:18], "little")
    decimals = raw[18]
elif len(raw) == 18:
    authority_type = 0
    authority_option = raw[0]
    supply = int.from_bytes(raw[1:17], "little")
    decimals = raw[17]
else:
    print(f"  FAIL {pda}: data too short ({len(raw)})")
    sys.exit(1)
print(f"  mint_pda={pda}")
print(f"  decoded MintDefinition: authority_type={authority_type} current_authority={'None' if authority_option == 0 else 'Some(...)'} supply={supply} decimals={decimals}")
if authority_option == 0 and supply == 100 and decimals == 6:
    print("  ok   revocation invariant holds: authority=None, supply=100, decimals=6")
else:
    print("  FAIL unexpected mint state")
    sys.exit(1)
PY
then
  :
else
  fail=1
fi

echo "Verifying final holding PDA state"
holding_resp="$(rpc getAccount "[\"$HOLDING_PDA\"]" || true)"
holding_file="$tmpdir/holding.json"
printf '%s' "$holding_resp" > "$holding_file"
if python3 - "$HOLDING_PDA" "$holding_file" <<'PY'
import json, sys
pda = sys.argv[1]
path = sys.argv[2]
resp = json.load(open(path, "r", encoding="utf-8"))
result = resp.get("result")
if not isinstance(result, dict):
    print(f"  FAIL {pda}: missing account result -> {resp}")
    sys.exit(1)
data = result.get("data")
if not isinstance(data, list) or len(data) < 48:
    print(f"  FAIL {pda}: missing/short holding data -> {result}")
    sys.exit(1)
raw = bytes(data)
balance = int.from_bytes(raw[32:48], "little")
print(f"  holding_pda={pda}")
print(f"  decoded TokenHolding: balance={balance}")
if balance == 100:
    print("  ok   holding balance invariant holds: balance=100")
else:
    print("  FAIL unexpected holding balance")
    sys.exit(1)
PY
then
  :
else
  fail=1
fi

if [[ "$fail" == "0" ]]; then
  echo "LP-0013 public testnet evidence verified."
else
  echo "LP-0013 public testnet evidence verification failed." >&2
  exit 1
fi
