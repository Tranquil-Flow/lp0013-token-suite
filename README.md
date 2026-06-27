# LP-0013 Token Authorities

> **✅ RESOLVED (2026-06-27) — corrected guest deployed + verified on the public testnet.** The init-only holding PDA was split into `create_holding` + mutable `mint_to`, and the corrected four-instruction guest was re-deployed to `testnet.lez.logos.co` under `RISC0_DEV_MODE=0`. The lifecycle now shows **two accumulating mints (60+40 → 100, variable supply on chain)** and a post-revoke mint **rejected by the authority guard** (`require_authority`), not by an `init` side effect — the holding already exists (`mut`). The prior 2026-06-03 testnet hashes are the superseded pre-fix run, retained as history. Re-verify read-only: `bash scripts/demo-testnet-live.sh verify`. Authoritative state → [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

Implementation for Logos λPrize LP-0013: Token program improvements — authorities.

This workspace provides a self-contained Rust implementation of mint authority lifecycle semantics for variable-supply and fixed-supply tokens, plus runnable examples, an offline CLI demo, a canonical IDL, real SPEL-generated IDL evidence, and a SPEL guest source that ports the same authority checks into the RISC0/LEZ account adapter.

> **Status (2026-06-09): final video linked; corrected guest deployed + verified on the public testnet.** The offline Rust authority suite is green across the workspace (now including a repeated-mint / post-revoke-guard contract test, `mint-program::variable_supply_allows_repeated_minting_to_same_holding`), and the canonical hand-written IDL + `spel generate-idl` output are checked in. **The on-chain correctness fix is landed and proven** (see [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md)): the prior on-chain mint program declared the recipient holding `#[account(init)]`, which is single-use, so (a) a second mint to the same holding would fail — variable supply was broken on chain — and (b) the prior post-revoke `mint_to` was rejected by that `init` side effect *before* the authority guard ran, so the guard (`Program error 2008`) was never genuinely exercised on chain. The fix splits minting into `create_holding` (claims the holding once) + `mint_to` (mutable holding, repeatable; guard runs first). The corrected guest (ImageID/ProgramId `338865e9…871554a`) was re-deployed to `testnet.lez.logos.co` on 2026-06-27 and the full lifecycle re-captured: two accumulating mints (60+40 → 100) demonstrate variable supply on chain, and the post-revoke `mint_to` is rejected by the guard with the mint PDA reading `authority=None, supply=100` (the +7 never landed). Re-verify read-only with `bash scripts/demo-testnet-live.sh verify`. The earlier 2026-06-03 hashes (ProgramId `4153e159…`, deploy `07561014…`) are retained only as historical pre-fix evidence; the 2026-06-27 corrected run is the fix evidence. The final narrated video is linked in [`SUBMISSION.md`](SUBMISSION.md) and [`solutions/LP-0013.md`](solutions/LP-0013.md). The deployable scaffold lives in this repo at `onchain-program/`; the LEZ client `onchain-program/examples/src/bin/live_lifecycle.rs` drove the two-mint accumulation + post-revoke-guard flow on testnet; and `spel generate-idl` was re-run for the four-instruction surface. The CU framing (deterministic deployed-ELF executor cycles; testnet exposes no per-tx CU) stands; per-op numbers are re-measured on a local sequencer (see `docs/BENCHMARKS.md`). Full plan: [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

## What is included

- `admin-authority-core` — runtime-agnostic authority state, authorization checks, rotation, and revocation.
- `mint-core` — pure token mint and holding state transitions.
- `mint-program` — instruction-level runtime harness for create/mint/rotate/revoke flows.
- `mint-sdk` — evaluator-facing Rust client API.
- `mint-cli` — offline demo CLI.
- `examples/variable-supply` — runnable variable-supply authority lifecycle.
- `examples/fixed-supply` — runnable fixed-supply/revoked-authority behavior.
- `examples/config-pda-gated` — runnable RFP-001-style config-PDA-gated authority flow.
- `docs/SPEC_COMPLIANCE.md` — honest status map against LP-0013 criteria.
- `docs/LP0013_REQUIREMENTS_MATRIX.md` — evaluator-facing checklist mapping every LP-0013 requirement to concrete artifacts, commands, and proof logs.

## Submission posture

This is public, non-confidential work intended for a λPrize submission. Do not commit private keys, seed phrases, credentials, unpublished private chat excerpts, or personal data.

This project does not claim Logos endorsement, audit, certification, operation, or guarantee.

## Prerequisites

- Rust toolchain with Cargo and rustfmt.
- Clippy for the final lint gate.

Check the local environment:

```bash
bash scripts/check-prereqs.sh
```

## Quick start

Run the full deterministic offline demo:

```bash
bash scripts/demo.sh
```

Run individual CLI demos:

```bash
cargo run -p mint-cli -- demo-variable
cargo run -p mint-cli -- demo-fixed
```

Run individual evaluator examples:

```bash
cargo run -p variable-supply
cargo run -p fixed-supply
cargo run -p config-pda-gated
```

## Verification

The standard local gate is:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

`bash scripts/check-prereqs.sh` runs the same gate and prints toolchain versions.

## Current status

Proven locally:

- create variable-supply mint with authority,
- mint by current authority,
- reject wrong/old authority,
- rotate authority atomically,
- revoke authority atomically,
- reject minting after revocation,
- create fixed-supply mint with revoked authority,
- exercise an RFP-001-style config PDA gate in an offline deterministic example.

> **✅ On-chain evidence (2026-06-27, corrected guest).** The corrected guest (`create_mint`, `create_holding`, mutable `mint_to`, `set_mint_authority`) is deployed and exercised on the **public LEZ testnet** (`testnet.lez.logos.co`, real consensus, `RISC0_DEV_MODE=0`). The lifecycle mints twice into one holding — `mint_to(60)` + `mint_to(40)` accumulating to `supply=100` / `balance=100` — then revokes mint authority to `None`; the post-revocation `mint_to` is not included, and the live mint-PDA readback remains `authority=None, supply=100`. Reproduce read-only with `bash scripts/demo-testnet-live.sh verify` (full log: `docs/LEZ_PROOF_LOG.md`).

Narrated demo video: https://youtu.be/rUgsCCPiQfo. This recording demonstrates the corrected public-testnet lifecycle and is the final video evidence for LP-0013.

Strict supportability checklist: `docs/LP0013_REQUIREMENTS_MATRIX.md`. The standalone local-sequencer e2e path is included in CI as `local-sequencer-e2e-preflight` (hosted prerequisite/syntax gate) plus `local-sequencer-e2e` (manual self-hosted LEZ/RISC0 runner). A real prepared-host run passed on 2026-06-09 with `RISC0_DEV_MODE=0`; see `docs/LEZ_PROOF_LOG.md`.

Final λPrize PR status:

- narrated public-testnet demo video recorded and linked above;
- public repository updated;
- upstream Logos PR opened at https://github.com/logos-co/lambda-prize/pull/77.

## License

Dual licensed under either:

- MIT, or
- Apache-2.0
