# LP-0013 Token Authorities

Implementation for Logos λPrize LP-0013: Token program improvements — authorities.

This workspace provides a self-contained Rust implementation of mint authority lifecycle semantics for variable-supply and fixed-supply tokens, plus runnable examples, an offline CLI demo, a canonical IDL, real SPEL-generated IDL evidence, and a SPEL guest source that ports the same authority checks into the RISC0/LEZ account adapter.

> **Status:** offline Rust authority suite green (31 unit + 3 example tests); canonical hand-written IDL + real `spel generate-idl` output checked in alongside; **live LEZ four-transaction lifecycle proven on local sequencer 2026-05-18 under `RISC0_DEV_MODE=0`** — semantic guest deployed (`b16831c0…04ab5`, block 49551), full lifecycle exercised (`create_mint` `7d582e7b…`, `mint_to(100)` `c474cf82…`, `set_mint_authority(None)` `756ee393…`, post-revoke `mint_to` rejected `27df9483…`), and **independently re-verified ~22 minutes later** with a fresh `set_mint_authority` (`cea5b8c7…`) rejected on chain at `Program error 2008: authority has been revoked` — the canonical `require_authority` panic from the guest body, observed live on the sequencer. Per-operation Risc0 zkVM executor time captured from the sequencer log: `create_mint` 8.38 ms, `mint_to` 7.58 ms, `set_mint_authority` (rotate/revoke) 6.76 ms; rejected post-revoke operations cost ~50% less (4.21–4.43 ms) — deterministic-rejection visible in the CU profile. Full proof log with all txhashes in [`docs/LEZ_PROOF_LOG.md`](docs/LEZ_PROOF_LOG.md); CU breakdown in [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md). Narrated demo video: <https://youtu.be/3hQd2G8O-UM>. **Remaining for submission:** open the PR after Evi sign-off.

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

Proven on host (sequencer at `127.0.0.1:3040` with `RISC0_DEV_MODE=0`):

- real `spel generate-idl` output for the LP-0013 surface, regenerated against the semantic guest and committed at `idl/admin-authority.idl.spel-generated.json`,
- RISC0 guest binary built on macOS arm64 (no recurrence of the LP-0017 cc-rs/ring failure under SPEL `v0.2.0-rc.3`),
- 2026-05-17 archival spike — structural-surface guest deployed and exercised: deploy txhash `2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965`, block 45491; wire-level lifecycle `fd68e225…` / `07de7c91…` / `ec58ace4…` / `e1ecbb81…`,
- 2026-05-18 semantic release-candidate rerun — semantic guest (ImageID `58470667…d0b960`) deployed (deploy txhash `b16831c0ee550014ea9297ba47d47b31d0c1b425ff3219b44358189bb9204ab5`, block 49551) and full lifecycle confirmed on chain: `create_mint` (`7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63`), `mint_to(100)` (`c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f`), `set_mint_authority(None)` (`756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c`), post-revoke `mint_to` rejected on chain (`27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff`), with the decoded mint PDA showing `supply=100, current_authority=None, decimals=6`,
- in-guest authority semantics implemented in `spel-spike/admin_authority_guest.rs` by Borsh-decoding account state, enforcing signer/revoked-authority checks, updating supply/balances, claiming the holding PDA on first mint, and rejecting zero/overflow cases before returning post-states,
- benchmarks recorded in `docs/BENCHMARKS.md`, full host log in `docs/LEZ_PROOF_LOG.md`.

Narrated demo video: <https://youtu.be/3hQd2G8O-UM>.

Still required before final λPrize PR:

- public repository push and Logos PR only after explicit Evi sign-off.

## License

Dual licensed under either:

- MIT, or
- Apache-2.0
