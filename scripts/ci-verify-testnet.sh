#!/usr/bin/env bash
# scripts/ci-verify-testnet.sh — wallet-free public LEZ testnet verifier for LP-0013.
#
# Read-only, no keys, no wallet build, no localnet, no mock. Re-queries the
# corrected 2026-06-04 authority lifecycle and the final mint PDA state from
# https://testnet.lez.logos.co/ using JSON-RPC only.
set -euo pipefail

SEQ="${LP0013_SEQUENCER:-https://testnet.lez.logos.co/}"
PROGRAM_ID="32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce"
MINT_PDA="HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu"

TXS=(
  "deploy_program|5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0|present"
  "create_mint|7d1dcb04b5f339b33f04a120b7334cf9802720d4a917e600becd62476e44da74|present"
  "create_holding|520d080b833c7e4038a1aa214bba43a3fc97328e8f379a093b74ca3e32be5893|present"
  "mint_to(60)|8c865d0184f55ce5a881e24c8c125cd3729c5f90a4b83d0484c8d1610f743f61|present"
  "mint_to(40)|c63168b7f615221ab2425b2ba003d32183f4df2e482eb4203e4e216675993d21|present"
  "set_mint_authority(None)|8c4b08b5c750c57d0dbb4e9f43c32b7c0f2627ce5508da85408e3aaf01f5a331|present"
  "mint_to(post-revoke)|6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1|null"
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
  resp="$(rpc get_transaction_by_hash "[\"$hash\"]" || true)"
  if python3 - <<'PY' "$resp" >/dev/null 2>&1
import json, sys
try:
    data=json.loads(sys.argv[1])
except Exception:
    sys.exit(1)
err=data.get('error')
sys.exit(0 if isinstance(err, dict) and err.get('code') == -32601 else 1)
PY
  then
    resp="$(rpc getTransaction "[\"$hash\"]" || true)"
  fi
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
if isinstance(result, dict) and "transaction" in result:
    result = result.get("transaction")
if expected == "present":
    ok = result is not None and (not isinstance(result, str) or len(result) > 0)
else:
    ok = result is None
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
acct_resp="$(rpc get_account "[\"$MINT_PDA\"]" || true)"
if python3 - <<'PY' "$acct_resp" >/dev/null 2>&1
import json, sys
try:
    data=json.loads(sys.argv[1])
except Exception:
    sys.exit(1)
err=data.get('error')
sys.exit(0 if isinstance(err, dict) and err.get('code') == -32601 else 1)
PY
then
  acct_resp="$(rpc getAccount "[\"$MINT_PDA\"]" || true)"
fi
acct_file="$tmpdir/account.json"
printf '%s' "$acct_resp" > "$acct_file"
if python3 - "$MINT_PDA" "$acct_file" <<'PY'
import json, sys
pda = sys.argv[1]
path = sys.argv[2]
resp = json.load(open(path, "r", encoding="utf-8"))
result = resp.get("result")
if isinstance(result, dict) and "account" in result:
    result = result.get("account")
if not isinstance(result, dict):
    print(f"  FAIL {pda}: missing account result -> {resp}")
    sys.exit(1)
data = result.get("data")
if not isinstance(data, list):
    print(f"  FAIL {pda}: missing byte-array data -> {result}")
    sys.exit(1)
raw = bytes(data)
if len(raw) < 19:
    print(f"  FAIL {pda}: data too short ({len(raw)})")
    sys.exit(1)
authority_type = raw[0]
authority_option = raw[1]
supply = int.from_bytes(raw[2:18], "little")
decimals = raw[18]
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

if [[ "$fail" == "0" ]]; then
  echo "LP-0013 public testnet evidence verified."
else
  echo "LP-0013 public testnet evidence verification failed." >&2
  exit 1
fi
