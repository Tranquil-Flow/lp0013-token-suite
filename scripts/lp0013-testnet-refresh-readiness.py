#!/usr/bin/env python3
"""LP-0013 private schema-aware readiness probe / guarded submitter.

Default mode is DRY-RUN ONLY: read-only RPC probes + tx artifact validation.
Execution mode requires both --execute and LP0013_I_UNDERSTAND_PUBLIC_SEND=YES.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

READ_METHOD_SETS = {
    "camelCase": {
        "health": "checkHealth",
        "programs": "getProgramIds",
        "nonces": "getAccountsNonces",
        "account": "getAccount",
        "tx": "getTransaction",
        "send": "sendTransaction",
    },
    "snake_case": {
        "health": "check_health",
        "programs": "get_program_ids",
        "nonces": "get_accounts_nonces",
        "account": "get_account",
        "tx": "get_transaction",
        "send": "send_tx",
    },
}

SUBMIT_CANDIDATES = [
    "sendTransaction",
    "send_tx",
    "sendTx",
    "submitTransaction",
    "submit_transaction",
]

EXPECTED_LABELS = [
    "deploy_program",
    "create_mint",
    "create_holding",
    "mint_to_60",
    "mint_to_40",
    "set_mint_authority_none",
    "mint_to_post_revoke",
]


def rpc(endpoint: str, method: str, params: list[Any], timeout: int = 25) -> dict[str, Any]:
    payload = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    req = urllib.request.Request(
        endpoint,
        data=json.dumps(payload, separators=(",", ":")).encode(),
        headers={"content-type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            raw = r.read().decode()
            status = r.status
    except urllib.error.HTTPError as e:
        raw = e.read().decode(errors="replace")
        status = e.code
    except Exception as e:  # noqa: BLE001 - report exact probe failure
        return {"ok": False, "transport_error": repr(e), "method": method}
    try:
        body = json.loads(raw)
    except Exception:
        return {"ok": False, "http_status": status, "raw": raw[:1000], "method": method}
    body["http_status"] = status
    return body


def is_method_not_found(resp: dict[str, Any]) -> bool:
    return resp.get("error", {}).get("code") == -32601


def has_result(resp: dict[str, Any]) -> bool:
    return resp.get("http_status") == 200 and "result" in resp


def git_sha(cwd: Path) -> str | None:
    try:
        return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=str(cwd), text=True).strip()
    except Exception:
        return None


def discover(endpoint: str, authority: str) -> dict[str, Any]:
    out: dict[str, Any] = {"read_schemas": {}, "submit_candidates": {}}
    for name, methods in READ_METHOD_SETS.items():
        health = rpc(endpoint, methods["health"], [])
        programs = rpc(endpoint, methods["programs"], [])
        nonces = rpc(endpoint, methods["nonces"], [[authority]])
        out["read_schemas"][name] = {
            "health_ok": has_result(health),
            "programs_ok": has_result(programs),
            "nonces_ok": has_result(nonces),
            "nonce_result": nonces.get("result"),
            "health_error": health.get("error"),
            "programs_error": programs.get("error"),
            "nonces_error": nonces.get("error"),
        }
    for method in SUBMIT_CANDIDATES:
        # invalid null probes method existence without submitting a transaction.
        resp = rpc(endpoint, method, [None])
        status = "present_invalid_params" if resp.get("error", {}).get("code") == -32602 else (
            "not_found" if is_method_not_found(resp) else ("result_unexpected" if "result" in resp else "other_error")
        )
        out["submit_candidates"][method] = {"status": status, "error": resp.get("error"), "result": resp.get("result")}
    return out


def choose_schema(discovery: dict[str, Any]) -> str | None:
    for name, state in discovery["read_schemas"].items():
        if state["health_ok"] and state["programs_ok"] and state["nonces_ok"]:
            return name
    return None


def summarize_account(resp: dict[str, Any]) -> dict[str, Any]:
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    acct = resp.get("result")
    if not isinstance(acct, dict):
        return {"ok": False, "result": acct}
    data = acct.get("data")
    return {
        "ok": True,
        "balance": acct.get("balance"),
        "nonce": acct.get("nonce"),
        "data_len": len(data) if isinstance(data, list) else None,
        "program_owner": acct.get("program_owner"),
    }


def poll_tx(endpoint: str, tx_method: str, tx_hash: str, attempts: int, sleep_s: float) -> dict[str, Any]:
    for i in range(attempts):
        resp = rpc(endpoint, tx_method, [tx_hash])
        if resp.get("result") is not None:
            return {"included": True, "attempt": i + 1, "response": resp}
        time.sleep(sleep_s)
    return {"included": False, "attempts": attempts}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--endpoint", default=os.environ.get("LP0013_RPC", "https://testnet.lez.logos.co/"))
    ap.add_argument("--tx-json", default=os.environ.get("LP0013_TX_JSON", "/tmp/lp0013_lifecycle_txs.json"))
    ap.add_argument("--repo", default=os.environ.get("LP0013_REPO", str(Path.cwd())))
    ap.add_argument("--execute", action="store_true", help="PUBLIC SENDS; requires LP0013_I_UNDERSTAND_PUBLIC_SEND=YES")
    ap.add_argument("--poll-attempts", type=int, default=180)
    ap.add_argument("--poll-sleep", type=float, default=2.0)
    args = ap.parse_args()

    tx_path = Path(args.tx_json)
    repo = Path(args.repo)
    if not tx_path.exists():
        print(json.dumps({"ok": False, "blockers": [f"missing tx artifact: {tx_path}"]}, indent=2))
        return 2
    artifact = json.loads(tx_path.read_text())
    authority = artifact.get("authority")
    mint_pda = artifact.get("mint_pda")
    holding_pda = artifact.get("holding_pda")
    txs = artifact.get("txs", [])

    blockers: list[str] = []
    warnings: list[str] = []
    if not authority:
        blockers.append("tx artifact missing authority")
    if len(txs) != len(EXPECTED_LABELS):
        blockers.append(f"expected {len(EXPECTED_LABELS)} txs, saw {len(txs)}")
    labels = [t.get("label") for t in txs]
    for label in EXPECTED_LABELS:
        if label not in labels:
            blockers.append(f"missing tx label: {label}")
    for t in txs:
        if not t.get("hash") or not t.get("transaction"):
            blockers.append(f"tx {t.get('label')} missing hash or transaction string")

    discovery = discover(args.endpoint, authority or "") if authority else {"read_schemas": {}, "submit_candidates": {}}
    schema_name = choose_schema(discovery)
    if not schema_name:
        blockers.append("no read schema fully healthy")
        methods = READ_METHOD_SETS["camelCase"]
    else:
        methods = READ_METHOD_SETS[schema_name]

    present_submit = [m for m, v in discovery.get("submit_candidates", {}).items() if v.get("status") == "present_invalid_params"]
    send_method = methods.get("send") if methods.get("send") in present_submit else (present_submit[0] if present_submit else None)
    if not send_method:
        blockers.append("no submit method detected by invalid-param presence probe")

    # Read-only state checks.
    nonce_resp = rpc(args.endpoint, methods["nonces"], [[authority]]) if schema_name and authority else {}
    nonce = None
    try:
        nonce = nonce_resp.get("result", [None])[0]
    except Exception:
        pass
    if nonce != 0:
        blockers.append(f"authority nonce is {nonce}, expected 0 for prebuilt tx artifact")

    accounts = {}
    for label, addr in [("authority", authority), ("mint_pda", mint_pda), ("holding_pda", holding_pda)]:
        if addr and schema_name:
            accounts[label] = summarize_account(rpc(args.endpoint, methods["account"], [addr]))
    for label in ("mint_pda", "holding_pda"):
        st = accounts.get(label, {})
        if st.get("ok") and st.get("data_len") not in (0, None):
            blockers.append(f"{label} already has data_len={st.get('data_len')}; lifecycle artifact would not start clean")

    existing = {}
    if schema_name:
        for t in txs:
            h = t.get("hash")
            if h:
                r = rpc(args.endpoint, methods["tx"], [h])
                existing[t.get("label", h)] = r.get("result") is not None
        if any(existing.values()):
            warnings.append("one or more candidate hashes already included; rerun tx generation with fresh nonce/account")

    summary: dict[str, Any] = {
        "ok": not blockers,
        "mode": "EXECUTE" if args.execute else "DRY_RUN_NO_SENDS",
        "timestamp_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "command_line": " ".join(sys.argv),
        "repo": str(repo),
        "repo_sha": git_sha(repo),
        "endpoint": args.endpoint,
        "schema": schema_name,
        "send_method": send_method,
        "authority": authority,
        "mint_pda": mint_pda,
        "holding_pda": holding_pda,
        "nonce": nonce,
        "accounts": accounts,
        "txs": [{"label": t.get("label"), "expect": t.get("expect"), "hash": t.get("hash"), "transaction_len": len(t.get("transaction", ""))} for t in txs],
        "candidate_hashes_already_included": existing,
        "discovery": discovery,
        "warnings": warnings,
        "blockers": blockers,
    }

    if args.execute:
        if os.environ.get("LP0013_I_UNDERSTAND_PUBLIC_SEND") != "YES":
            summary["ok"] = False
            summary["blockers"].append("execution refused: set LP0013_I_UNDERSTAND_PUBLIC_SEND=YES")
            print(json.dumps(summary, indent=2, sort_keys=True))
            return 3
        if blockers:
            print(json.dumps(summary, indent=2, sort_keys=True))
            return 4
        results = []
        for t in txs:
            label = t["label"]
            expect = t.get("expect")
            h = t["hash"]
            send_resp = rpc(args.endpoint, send_method, [t["transaction"]])
            entry = {"label": label, "expected_hash": h, "expect": expect, "send_response": send_resp}
            if "result" not in send_resp:
                entry["status"] = "send_failed"
                results.append(entry)
                summary["execution_results"] = results
                summary["ok"] = False
                summary["blockers"].append(f"send failed at {label}")
                print(json.dumps(summary, indent=2, sort_keys=True))
                return 5
            entry["returned_hash"] = send_resp.get("result")
            if expect == "confirm":
                entry["poll"] = poll_tx(args.endpoint, methods["tx"], h, args.poll_attempts, args.poll_sleep)
                entry["status"] = "confirmed" if entry["poll"].get("included") else "not_confirmed"
                if entry["status"] != "confirmed":
                    summary["ok"] = False
                    summary["blockers"].append(f"{label} not confirmed")
                    results.append(entry)
                    summary["execution_results"] = results
                    print(json.dumps(summary, indent=2, sort_keys=True))
                    return 6
            else:
                entry["poll"] = poll_tx(args.endpoint, methods["tx"], h, max(10, args.poll_attempts // 3), args.poll_sleep)
                entry["status"] = "rejected_or_not_included" if not entry["poll"].get("included") else "unexpectedly_included"
                if entry["status"] == "unexpectedly_included":
                    summary["ok"] = False
                    summary["blockers"].append(f"{label} unexpectedly included")
                    results.append(entry)
                    summary["execution_results"] = results
                    print(json.dumps(summary, indent=2, sort_keys=True))
                    return 7
            results.append(entry)
        # final readback after execute
        summary["execution_results"] = results
        summary["post_accounts"] = {
            "mint_pda": summarize_account(rpc(args.endpoint, methods["account"], [mint_pda])),
            "holding_pda": summarize_account(rpc(args.endpoint, methods["account"], [holding_pda])),
        }

    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
