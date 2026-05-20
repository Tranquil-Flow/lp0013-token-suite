# LP-0013 Token Authority Compliance Map

Status: release-candidate documentation; offline Rust authority suite is locally green, SPEL IDL evidence is captured, and the guest source now carries the authority semantics port. Narrated demo video remains outstanding.

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
- Documentation and examples: complete except narrated video.
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
- IDL using SPEL framework: dual-artifact, additive evidence.
  - `idl/admin-authority-idl.json` is the canonical hand-written IDL for the proven instruction/account surface, declared `metadata.generation = "hand-written"` and test-guarded by `admin-authority-spel`.
  - `idl/admin-authority-idl.spel-generated.json` is real `spel generate-idl` output regenerated on host (2026-05-17) against the same SPEL revision LP-0017 uses (`ed3bbedb4b684645da05455d30a4a0be7cc4dfe0`, LEZ tag `v0.2.0-rc1`).
  - The two artifacts agree on the instruction set (`create_mint`, `mint_to`, `set_mint_authority`). The canonical hand-written file is the documented superset: the current SPEL-generated artifact does not emit account/type bodies, discriminators, `execution` block, error codes, or expanded type signatures.
  - The spike guest sources are checked in under `spel-spike/` for reproducibility. See `docs/SPEL_STATUS.md` for the diff rationale and `docs/LEZ_PROOF_LOG.md` for the host environment.

## Reliability

- Authority rotation/revocation atomic: proven in unit tests.
- Minting with revoked authority rejected deterministically: proven in unit/integration tests using `TokenError::AuthorityRevoked`.
- Wrong/old authority rejection leaves state unchanged: proven in unit and example-output tests.

## Performance

- Performance documentation: complete.
  - `docs/BENCHMARKS.md` §"Compute units (CU)" documents per-operation Risc0 zkVM executor time extracted from the live local sequencer log for each tx in the 2026-05-18 semantic release-candidate lifecycle: `create_mint` 8.38 ms, `mint_to` 7.58 ms, `set_mint_authority` (rotate/revoke shares one code path) 6.76 ms. Rejected post-revoke operations cost ~50% less (4.21–4.43 ms) because execution halts at the authority guard before any account writes — the deterministic-rejection property the spec mandates, visible in the CU profile. Methodology mirrors LP-0017's `BENCHMARKS.md` §"Methodology" (public-transaction CU = sequencer-side executor time; `RISC0_DEV_MODE=0` doesn't shift the numbers because the host-side prover is bypassed on the public-transaction path).

## Supportability

- CI workflow: added at `.github/workflows/ci.yml`.
- Deployed/tested on LEZ local devnet: host spike complete; semantic release-candidate rerun completed 2026-05-18 with the semantic guest deployed, `create_mint` / `mint_to(100)` / `set_mint_authority(None)` confirmed and post-revoke `mint_to` rejected (see `docs/LEZ_PROOF_LOG.md`).
- End-to-end tests against LEZ sequencer in CI: not complete; this remains a tooling/infrastructure gap because local `lgs`/SPEL are host-only.
- README end-to-end usage: complete for offline evaluator paths; host LEZ reproduction steps live in `spel-spike/README.md` and `docs/LEZ_PROOF_LOG.md`.
- Reproducible demo script: complete for offline evaluator paths.
  - `scripts/demo.sh` runs CLI demos plus all three runnable examples.
  - `spel-spike/live_lifecycle.rs` is the host-only LEZ lifecycle driver for release-candidate reruns.
- Recorded narrated video: not complete.

## Current Local Verification

The following local gates pass at this checkpoint:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/demo.sh
```
