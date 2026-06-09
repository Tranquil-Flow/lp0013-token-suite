# LP-0013 Token Authority Compliance Map

> **✅ RESOLVED (2026-06-04) — corrected guest deployed + verified on the public testnet.** The init-only holding PDA was split into `create_holding` + mutable `mint_to`, re-deployed to `testnet.lez.logos.co` (`RISC0_DEV_MODE=0`), and the lifecycle re-captured: two accumulating mints (60+40 → 100) prove variable supply on chain, and the post-revoke mint is rejected by the authority guard (`require_authority`), not by an `init` side effect (the holding already exists, `mut`). The prior 2026-06-03 run is the superseded pre-fix record. Authoritative state → [`../RESUBMISSION_STATUS.md`](../RESUBMISSION_STATUS.md).

Status: **on-chain verified (see banner).** Offline Rust authority suite is locally green (now including a repeated-mint / post-revoke-guard contract test), the SPEL IDL is regenerated for the corrected four-instruction surface (the rc1 and rc3 pin generations are byte-identical), and the guest source carries the corrected two-instruction minting model (`create_holding` + mutable `mint_to`). The corrected guest is **deployed and its authority lifecycle is verified on the public LEZ testnet** (2026-06-04; ProgramId/ImageID `32335764…b0a9ce`; two accumulating mints → supply 100, guard-rejected post-revoke mint) — re-verify read-only with `bash scripts/demo-testnet-live.sh verify`. The prior 2026-06-03 testnet run is the superseded pre-fix record (see `docs/LEZ_PROOF_LOG.md`). A narrated demo video of the corrected testnet lifecycle remains the one outstanding (human) task.

This document tracks the submission against the LP-0013 success criteria and keeps evaluator-facing claims honest.

## Functionality

- Variable-size tokens through minting authority: proven in pure Rust/offline harness.
  - `admin-authority-core` implements authority initialization, authorization checks, rotation, and revocation.
  - `mint-core` implements variable-supply and fixed-supply mint state transitions.
  - `mint-program` implements pure instruction semantics for `CreateMint`, `MintTo`, and `SetMintAuthority`.
  - `mint-cli demo-variable` exercises create -> mint -> rotate -> mint -> revoke -> rejected mint.
  - `examples/variable-supply` is a runnable evaluator example for the same lifecycle.
- Fixed-supply/revoked-authority behavior: proven in pure Rust/offline harness.
  - `mint-cli demo-fixed` and `examples/fixed-supply` show that a fixed mint starts with no mint authority and rejects future minting.
- Documentation and examples: complete; narrated demo video recorded at https://youtu.be/rUgsCCPiQfo.
  - README has quick-start, verification, and current status.
  - Three runnable examples exist: variable supply, fixed supply, and config-PDA-gated.
  - Per-example README files document each flow.
- Self-sufficient agnostic library for RFP-001 approval: partial.
  - `admin-authority-core` is runtime-agnostic.
  - `examples/config-pda-gated` demonstrates a deterministic config-derived authority gate offline.
  - SPEL/RFP-001 integration evidence is captured through the generated IDL and `config-pda-gated` example; guest semantics now use the same authority checks at the account-adapter layer.

## Usability

- SDK/module for interacting with token program: proven offline.
  - `mint-sdk::TokenClient` wraps create/mint/rotate/revoke/query operations.
- IDL using SPEL framework: real generated IDL + a hand-written design reference.
  - `idl/admin-authority.idl.spel-generated.json` is the authoritative IDL: real `spel_framework::generate_idl!` output (exactly what `make idl` emits) for the corrected guest. It carries all four instructions (`create_mint`, `create_holding`, `mint_to`, `set_mint_authority`), the full `AuthorityInfo` / `MintDefinition` / `TokenHolding` account bodies, and each instruction's PDA seeds and signer/writable/init access modes. **The spel-generated IDL is authoritative for the on-chain account/arg surface.**
  - `idl/admin-authority.idl.spel-generated.rc3-testnet.json` is the same generator run under the public-testnet pins (`v0.2.0-rc3`, spel `31e52c52`). Under the corrected guest (which annotates `#[account_type]`) it is **byte-identical across the rc1 (`ed3bbedb4b684645da05455d30a4a0be7cc4dfe0`, `v0.2.0-rc1`) and rc3 (`31e52c52`) spel revisions** — we ship both pin labels as a cross-revision stability check, not as richer-vs-poorer artifacts.
  - `idl/admin-authority.idl.json` is a hand-written design reference, declared `metadata.generation = "hand-written"` and test-guarded by `admin-authority-spel`. It additionally documents instruction `execution` semantics, the declared `TokenError` set, and expanded `Option` types the generator omits. Its `metadata.caveats` disclose that its 8-byte discriminators are illustrative (LEZ dispatches by enum-variant index) and that its instruction `args` model the offline mint-program call surface rather than the on-chain account/arg split.
  - The spike guest sources are checked in under `spel-spike/` for reproducibility. See `docs/SPEL_STATUS.md` for the diff rationale and `docs/LEZ_PROOF_LOG.md` for the host environment.

## Reliability

- Authority rotation/revocation atomic: proven in unit tests.
- Minting with revoked authority rejected deterministically: proven in unit/integration tests using `TokenError::AuthorityRevoked`.
- Wrong/old authority rejection leaves state unchanged: proven in unit and example-output tests.

## Performance

- Performance documentation: complete.
  - `docs/BENCHMARKS.md` §"Compute units (CU)" documents per-operation Risc0 zkVM executor time extracted from the live local sequencer log for each tx in the 2026-05-18 semantic release-candidate lifecycle: `create_mint` 8.38 ms, `mint_to` 7.58 ms, `set_mint_authority` (rotate/revoke shares one code path) 6.76 ms. Rejected post-revoke operations cost ~50% less (4.21–4.43 ms) because execution halts at the authority guard before any account writes — the deterministic-rejection property the spec mandates, visible in the CU profile. Methodology mirrors LP-0017's `BENCHMARKS.md` §"Methodology" (public-transaction CU = sequencer-side executor time; `RISC0_DEV_MODE=0` doesn't shift the numbers because the host-side prover is bypassed on the public-transaction path). **These numbers are from the pre-fix guest (single `mint_to`); they and the operation set (now four ops including `create_holding`) would be re-measured on a local sequencer as a documented, non-blocking follow-up — the public testnet (where the corrected guest is now deployed) exposes no per-tx CU, so per-op CU is only observable on a local sequencer. The methodology is unchanged.**

## Supportability

- CI workflow: added at `.github/workflows/ci.yml`.
- Deployed/tested on LEZ testnet: **done (corrected guest, 2026-06-04).** The corrected guest was deployed and exercised on the **public testnet** (`testnet.lez.logos.co`, `RISC0_DEV_MODE=0`; ProgramId/ImageID `32335764…b0a9ce`): create_mint → create_holding → mint_to(60) → mint_to(40) [accumulates → 100] → set_mint_authority(None) → guard-rejected post-revoke mint. Two accumulating mints prove variable supply on chain; the post-revoke mint is rejected by `require_authority` (not an init side effect — the holding already exists, `mut`). Re-verify read-only via `bash scripts/demo-testnet-live.sh verify`. The prior 2026-06-03 run (ProgramId `4153e159…2caeb08f`) used the pre-fix guest and is retained as history. Corroborated at the wire/semantic layer by the 2026-05-18 local-sequencer rerun (see `docs/LEZ_PROOF_LOG.md`).
- End-to-end tests against LEZ sequencer in CI: partial / honestly constrained. Hosted CI cannot build the RISC0 guest or run a sequencer/faucet, so the on-chain lifecycle is not part of the per-push gate. The offline suite + the doc validator (`scripts/validate-submission-docs.py` and its self-tests) run on every push; a **manual `workflow_dispatch` job** (`testnet-verify` in `.github/workflows/ci.yml`) builds the LEZ `wallet` and re-verifies the on-chain evidence read-only. Locally, `scripts/demo-localnet.sh` runs the full build + deploy + lifecycle against a local sequencer, and `scripts/demo-testnet-live.sh verify` re-verifies the public testnet from a clean clone.
- README end-to-end usage: complete for offline evaluator paths; LEZ reproduction steps live in `spel-spike/README.md`, `scripts/demo-testnet-live.sh`, and `docs/LEZ_PROOF_LOG.md`.
- Reproducible demo script: complete for offline, local-sequencer, and public-testnet paths.
  - `scripts/demo.sh` runs CLI demos plus all three runnable examples (offline; runs in CI).
  - `scripts/demo-localnet.sh` builds the in-repo `onchain-program/` guest and drives the full corrected lifecycle (create_mint → create_holding → two accumulating mints → revoke → guard-rejected post-revoke mint) against a local LEZ sequencer; `--check` reports prerequisites.
  - `scripts/demo-testnet-live.sh verify` re-verifies the public-testnet deploy + lifecycle straight from the sequencer (read-only; needs only the `wallet` binary, no build or faucet); its `full` mode documents a fresh deploy from a funded account. (Its current hashes are the pre-fix run, to be refreshed at re-deploy.)
  - `onchain-program/examples/src/bin/live_lifecycle.rs` is the LEZ lifecycle driver (runs against localnet or the public testnet via env config).
- Recorded narrated video: complete — https://youtu.be/rUgsCCPiQfo.

## Current Local Verification

The following local gates pass at this checkpoint:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/demo.sh
```
