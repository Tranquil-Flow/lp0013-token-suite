# LP-0013 Token Authorities

> **✅ RESOLVED (2026-06-04) — corrected guest deployed + verified on the public testnet.** The init-only holding PDA was split into `create_holding` + mutable `mint_to`, and the corrected four-instruction guest was re-deployed to `testnet.lez.logos.co` under `RISC0_DEV_MODE=0`. The lifecycle now shows **two accumulating mints (60+40 → 100, variable supply on chain)** and a post-revoke mint **rejected by the authority guard** (`require_authority`), not by an `init` side effect — the holding already exists (`mut`). The prior 2026-06-03 testnet hashes are the superseded pre-fix run, retained as history. Re-verify read-only: `bash scripts/demo-testnet-live.sh verify`. Authoritative state → [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

Implementation for Logos λPrize LP-0013: Token program improvements — authorities.

This workspace provides a self-contained Rust implementation of mint authority lifecycle semantics for variable-supply and fixed-supply tokens, plus runnable examples, an offline CLI demo, a canonical IDL, real SPEL-generated IDL evidence, and a SPEL guest source that ports the same authority checks into the RISC0/LEZ account adapter.

> **Status (2026-06-04): corrected guest deployed + verified on the public testnet.** The offline Rust authority suite is green across the workspace (now including a repeated-mint / post-revoke-guard contract test, `mint-program::variable_supply_allows_repeated_minting_to_same_holding`), and the canonical hand-written IDL + `spel generate-idl` output are checked in. **The on-chain correctness fix is landed and proven** (see [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md)): the prior on-chain mint program declared the recipient holding `#[account(init)]`, which is single-use, so (a) a second mint to the same holding would fail — variable supply was broken on chain — and (b) the prior post-revoke `mint_to` was rejected by that `init` side effect *before* the authority guard ran, so the guard (`Program error 2008`) was never genuinely exercised on chain. The fix splits minting into `create_holding` (claims the holding once) + `mint_to` (mutable holding, repeatable; guard runs first). The corrected guest (ImageID/ProgramId `32335764…b0a9ce`) was re-deployed to `testnet.lez.logos.co` on 2026-06-04 and the full lifecycle re-captured: two accumulating mints (60+40 → 100) demonstrate variable supply on chain, and the post-revoke `mint_to` is rejected by the guard with the mint PDA reading `authority=None, supply=100` (the +7 never landed). Re-verify read-only with `bash scripts/demo-testnet-live.sh verify`. The earlier 2026-06-03 hashes (ProgramId `4153e159…`, deploy `07561014…`) are the superseded pre-fix run and must not be cited as fix evidence. **Remaining (human):** re-record the narrated video against the corrected lifecycle, then resubmit. (Already done: the deployable scaffold lives in this repo at `onchain-program/`; the LEZ client `onchain-program/examples/src/bin/live_lifecycle.rs` drove the two-mint accumulation + post-revoke-guard flow on testnet; and `spel generate-idl` was re-run for the four-instruction surface.) The CU framing (deterministic deployed-ELF executor cycles; testnet exposes no per-tx CU) stands; per-op numbers are re-measured on a local sequencer (see `docs/BENCHMARKS.md`). Full plan: [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

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

> **✅ On-chain evidence (2026-06-04, corrected guest).** The corrected guest (two-instruction `create_holding` + mutable `mint_to`, ImageID/ProgramId `32335764…b0a9ce`) is deployed and exercised on the **public LEZ testnet** (`testnet.lez.logos.co`, real consensus, `RISC0_DEV_MODE=0`). The lifecycle mints twice into one holding — `mint_to(60)` (`8c865d01…`) + `mint_to(40)` (`c63168b7…`) accumulating to `supply=100` / `balance=100` (variable supply on chain) — then `set_mint_authority(None)` (`8c4b08b5…`), then a post-revoke `mint_to` (`6e92e605…`) that is **never included**: because the holding already exists (`mut`, not `init`), the rejection is the authority guard (`require_authority`, error 2008), and the live mint-PDA readback shows `authority=None, supply=100` (the +7 never landed). Reproduce read-only with `bash scripts/demo-testnet-live.sh verify` (full log: `docs/LEZ_PROOF_LOG.md`). The earlier 2026-06-03 public-testnet run (ProgramId `4153e159…`/ImageID `59e15341…`) and the 2026-05-17/05-18 local-sequencer spikes were against the **pre-fix single-`init` guest** (one mint, init-side-effect rejection) and are retained in `docs/LEZ_PROOF_LOG.md` as historical records only. Also valid independent of the fix: the rc3 pins (`v0.2.0-rc3` = `cf3639d8`) build cleanly on macOS arm64; public-tx execution is sequencer-side and charges no gas; `spel generate-idl` output is checked in for the corrected four-instruction surface (`idl/admin-authority.idl.spel-generated*.json`). Authoritative status + full plan: [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

Narrated demo video: https://youtu.be/rUgsCCPiQfo. This fresh recording demonstrates the corrected public-testnet lifecycle and replaces the superseded local-sequencer video (<https://youtu.be/3hQd2G8O-UM>, 2026-05-20), which predates both the public-testnet run and this fix.

Final λPrize PR status:

- narrated public-testnet demo video recorded and linked above;
- public repository update and Logos PR publication remain gated on explicit Evi sign-off.

## License

Dual licensed under either:

- MIT, or
- Apache-2.0
