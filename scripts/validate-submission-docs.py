#!/usr/bin/env python3
"""Validate LP-0013 submission-facing artifacts before public push/PR.

This checks for local completeness and honest status documentation. It validates
that the offline release gates, canonical/genuine IDL artifacts, host proof log,
and semantic SPEL guest source are all present and consistently described.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = [
    "README.md",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "Cargo.toml",
    "Cargo.lock",
    ".github/workflows/ci.yml",
    "docs/SPEC_COMPLIANCE.md",
    "docs/SPEL_STATUS.md",
    "docs/HOST_LOGOS_TOOLCHAIN.md",
    "docs/LEZ_PROOF_LOG.md",
    "docs/BENCHMARKS.md",
    "SUBMISSION.md",
    "RESUBMISSION_STATUS.md",
    "solutions/LP-0013.md",
    "idl/admin-authority.idl.json",
    "idl/admin-authority.idl.spel-generated.json",
    "idl/admin-authority.idl.spel-generated.rc3-testnet.json",
    "spel-spike/admin_authority_guest.rs",
    "spel-spike/generate_idl.rs",
    "spel-spike/live_lifecycle.rs",
    "spel-spike/README.md",
    "scripts/demo.sh",
    "scripts/demo-testnet-live.sh",
    "scripts/demo-localnet.sh",
    "scripts/check-prereqs.sh",
    "examples/variable-supply/Cargo.toml",
    "examples/variable-supply/README.md",
    "examples/fixed-supply/Cargo.toml",
    "examples/fixed-supply/README.md",
    "examples/config-pda-gated/Cargo.toml",
    "examples/config-pda-gated/README.md",
]

README_REQUIRED_PHRASES = [
    "LP-0013",
    "Token program improvements",
    "cargo run -p variable-supply",
    "cargo run -p fixed-supply",
    "cargo run -p config-pda-gated",
    "cargo clippy --workspace --all-targets -- -D warnings",
    "RISC0_DEV_MODE=0",
    "explicit Evi sign-off",
    "MIT",
    "Apache-2.0",
    "testnet.lez.logos.co",
]

COMPLIANCE_REQUIRED_PHRASES = [
    "Variable-size tokens",
    "Fixed-supply",
    "RFP-001",
    "IDL using SPEL framework",
    "not complete",
    "Current Local Verification",
]

SPEL_STATUS_REQUIRED_PHRASES = [
    "spel",
    "lgs",
    "cargo-risczero",
    "v0.2.0-rc1",
    "ed3bbedb4b684645da05455d30a4a0be7cc4dfe0",
    "admin-authority.idl.spel-generated.json",
    "hand-written",
    # rc3 / public-testnet regeneration
    "v0.2.0-rc3",
    "admin-authority.idl.spel-generated.rc3-testnet.json",
    "31e52c52",
]

LEZ_PROOF_LOG_REQUIRED_PHRASES = [
    "macOS 15.6.1",
    "cargo-risczero",
    "spel -- generate-idl",
    "admin-authority.idl.spel-generated.json",
    "127.0.0.1:3040",
    "Semantic LEZ rerun",
    # archival structural-surface tx hashes — kept as historical proof
    "2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965",
    "fd68e225ceb3164f88367600564a026dbfb8f4823f449a6b07c37fc35de79c69",
    "07de7c91b5137fdb88b1f0ad84bb3b30a436cf9e8e368193fc81998713d88811",
    "ec58ace48bbadee7143585b7bc402b33dd5fd767b8dd15dcf13ce1a87eba204d",
    "e1ecbb81da1a828a7068ef05401c96ed7593d29c8fa9537c07bda1dea020a3f3",
    # semantic release-candidate tx hashes — 2026-05-18 rerun
    "b16831c0ee550014ea9297ba47d47b31d0c1b425ff3219b44358189bb9204ab5",
    "7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63",
    "c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f",
    "756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c",
    "27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff",
    "58470667b5d45fcc4317684eb7aaad2b19c0cf666bd8c7f85d2b0e1069d0b960",
    # independent re-verification (2026-05-18, ~22 min after rerun) —
    # canonical authority-revoked rejection observed on chain
    "cea5b8c7a23ed1e2bbb489284d993257786d15728627666a2c7c7581c1fc5eb4",
    "Program error 2008: authority has been revoked",
    # public testnet deploy + lifecycle (2026-06-03) — pre-fix run (superseded; hashes retained as history)
    "https://testnet.lez.logos.co/",
    "v0.2.0-rc3",
    "59e15341b10dfacf6bfeb8436f587e18fb4bf714fc042c79aba9f8878fb0ae2c",
    "07561014a617dc18c3a420db01c9f752755053eb58f44d8db98871646cb968ba",
    "17d90ea633db426a863efc697239aa158293c20822ff07839a2a0b6f2eeb37d2",
    "be393bcf82e489bc5a940904ed0e38ea861b61939f43529132ca4c701f29bbd8",
    "0540648f9f5099296340bcf65d0ac1a4cf89ff226eca7abb27dcdcb0b29f5784",
    "312ea9f120602f9aa2d574d43fefa73ae25d74e1bd228b9f65317fef8fef4798",
    # public testnet deploy + lifecycle (2026-06-04) — CORRECTED guest (load-bearing evidence)
    "32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce",
    "4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF",
    "HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu",
    "5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0",
    "7d1dcb04b5f339b33f04a120b7334cf9802720d4a917e600becd62476e44da74",
    "520d080b833c7e4038a1aa214bba43a3fc97328e8f379a093b74ca3e32be5893",
    "8c865d0184f55ce5a881e24c8c125cd3729c5f90a4b83d0484c8d1610f743f61",
    "c63168b7f615221ab2425b2ba003d32183f4df2e482eb4203e4e216675993d21",
    "8c4b08b5c750c57d0dbb4e9f43c32b7c0f2627ce5508da85408e3aaf01f5a331",
    "6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1",
]

HOST_TOOLCHAIN_REQUIRED_PHRASES = [
    "LP-0017 was previously built on this computer outside the sandbox",
    "Claude Code macOS shell",
    "RISC0_DEV_MODE=0",
    "NSSA_SEQUENCER_URL=http://127.0.0.1:3040",
    "docs/LEZ_PROOF_LOG.md",
    "Do not push",
]

SUBMISSION_REQUIRED_PHRASES = [
    "LP-0013 Token Authorities",
    "Tranquil-Flow/lp0013-token-suite",
    "offline Rust authority suite: proven",
    "admin-authority.idl.spel-generated.json",
    "explicit Evi sign-off",
    "RISC0_DEV_MODE=0",
    "No private keys",
    "testnet.lez.logos.co",
    "public-testnet deploy + authority lifecycle",
]

SOLUTION_REQUIRED_PHRASES = [
    "LP-0013",
    "Tranquil-Flow/lp0013-token-suite",
    "32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce",
    "5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0",
    "6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1",
    "scripts/demo-testnet-live.sh verify",
    "Recorded narrated demo video",
    "Pending human recording",
    "superseded historical evidence only",
]

EXPECTED_IDL_INSTRUCTIONS = ["create_mint", "create_holding", "mint_to", "set_mint_authority"]
EXPECTED_IDL_ACCOUNTS = ["AuthorityInfo", "MintDefinition", "TokenHolding"]
EXPECTED_SPEL_GENERATED_INSTRUCTIONS = [
    "create_mint",
    "create_holding",
    "mint_to",
    "set_mint_authority",
]


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_text(relative: str) -> str:
    path = ROOT / relative
    try:
        return path.read_text()
    except FileNotFoundError:
        fail(f"missing required file: {relative}")


def require_files() -> list[str]:
    checked = []
    for relative in REQUIRED_FILES:
        path = ROOT / relative
        if not path.exists():
            fail(f"missing required file: {relative}")
        if path.is_file() and path.stat().st_size == 0:
            fail(f"required file is empty: {relative}")
        checked.append(relative)
    return checked


def require_phrases(relative: str, phrases: list[str]) -> None:
    text = read_text(relative)
    for phrase in phrases:
        if phrase not in text:
            fail(f"{relative} missing phrase: {phrase}")


def validate_idl() -> None:
    try:
        idl = json.loads(read_text("idl/admin-authority.idl.json"))
    except json.JSONDecodeError as exc:
        fail(f"idl/admin-authority.idl.json is invalid JSON: {exc}")

    if idl.get("name") != "admin_authority":
        fail("IDL name must be admin_authority")
    if idl.get("metadata", {}).get("generation") != "hand-written":
        fail("hand-written IDL must declare hand-written generation")
    if (
        idl.get("metadata", {}).get("tooling_status")
        != "spel-available-generated-idl-shipped-alongside"
    ):
        fail("hand-written IDL must declare that the spel-generated IDL is shipped alongside")
    if not idl.get("metadata", {}).get("caveats"):
        fail("hand-written IDL must disclose its discriminator/arg caveats")

    instruction_names = [item.get("name") for item in idl.get("instructions", [])]
    if instruction_names != EXPECTED_IDL_INSTRUCTIONS:
        fail(f"IDL instructions mismatch: {instruction_names}")

    account_names = [item.get("name") for item in idl.get("accounts", [])]
    for account in EXPECTED_IDL_ACCOUNTS:
        if account not in account_names:
            fail(f"IDL missing account type: {account}")


def validate_spel_generated_idl() -> None:
    relative = "idl/admin-authority.idl.spel-generated.json"
    try:
        idl = json.loads(read_text(relative))
    except json.JSONDecodeError as exc:
        fail(f"{relative} is invalid JSON: {exc}")

    if idl.get("name") != "admin_authority":
        fail(f"{relative} name must be admin_authority")

    instruction_names = [item.get("name") for item in idl.get("instructions", [])]
    if instruction_names != EXPECTED_SPEL_GENERATED_INSTRUCTIONS:
        fail(f"{relative} instructions mismatch: {instruction_names}")


def validate_spel_generated_rc3_idl() -> None:
    """The rc3 / testnet-matching generation must emit the full account bodies.
    Under the corrected guest (which annotates #[account_type]) this generation
    is byte-identical to the rc1 generation — a cross-revision stability check,
    not a richer artifact — but both must still carry the account bodies."""
    relative = "idl/admin-authority.idl.spel-generated.rc3-testnet.json"
    try:
        idl = json.loads(read_text(relative))
    except json.JSONDecodeError as exc:
        fail(f"{relative} is invalid JSON: {exc}")

    if idl.get("name") != "admin_authority":
        fail(f"{relative} name must be admin_authority")

    instruction_names = [item.get("name") for item in idl.get("instructions", [])]
    if instruction_names != EXPECTED_SPEL_GENERATED_INSTRUCTIONS:
        fail(f"{relative} instructions mismatch: {instruction_names}")

    account_names = [item.get("name") for item in idl.get("accounts", [])]
    for account in EXPECTED_IDL_ACCOUNTS:
        if account not in account_names:
            fail(f"{relative} missing account body: {account} (rc3 must emit account bodies)")


def main() -> int:
    checked = require_files()
    require_phrases("README.md", README_REQUIRED_PHRASES)
    require_phrases("docs/SPEC_COMPLIANCE.md", COMPLIANCE_REQUIRED_PHRASES)
    require_phrases("docs/SPEL_STATUS.md", SPEL_STATUS_REQUIRED_PHRASES)
    require_phrases("docs/HOST_LOGOS_TOOLCHAIN.md", HOST_TOOLCHAIN_REQUIRED_PHRASES)
    require_phrases("docs/LEZ_PROOF_LOG.md", LEZ_PROOF_LOG_REQUIRED_PHRASES)
    require_phrases("SUBMISSION.md", SUBMISSION_REQUIRED_PHRASES)
    require_phrases("solutions/LP-0013.md", SOLUTION_REQUIRED_PHRASES)
    validate_idl()
    validate_spel_generated_idl()
    validate_spel_generated_rc3_idl()

    print("submission docs validated")
    for relative in checked:
        print(f"ok: {relative}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
