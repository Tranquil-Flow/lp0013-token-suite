# LP-0013 Token Authorities — Submission Draft

> **Final status (2026-06-04): corrected guest deployed + verified on the public testnet.** The four-instruction guest is deployed on `testnet.lez.logos.co` with `RISC0_DEV_MODE=0`. The public-testnet lifecycle shows two accumulating mints (`60 + 40 = 100`) and a post-revocation mint rejected by the authority guard while the holding already exists (`mut`). Authoritative state → [`RESUBMISSION_STATUS.md`](RESUBMISSION_STATUS.md).

## Summary

This submission implements token mint authority lifecycle support for Logos λPrize LP-0013.

It provides a self-contained Rust workspace proving the core authority model, token mint state transitions, instruction-level semantics, SDK ergonomics, CLI demos, runnable examples, CI, a canonical IDL artifact, real SPEL-generated IDL evidence, and a SPEL guest source that ports the same authority semantics into the RISC0/LEZ account adapter. The offline suite is green (including a repeated-mint / post-revoke-guard contract test). **The corrected guest is deployed and its authority lifecycle is verified on the public LEZ testnet** (`https://testnet.lez.logos.co/`, real consensus, `RISC0_DEV_MODE=0`, 2026-06-04): two accumulating mints (60+40 → 100) demonstrate variable supply on chain, and the post-revoke mint is rejected by the authority guard — the holding already exists (`mut`), so the rejection is `require_authority`, not an `init` side effect. Proof log: `docs/LEZ_PROOF_LOG.md`; requirements matrix: `docs/LP0013_REQUIREMENTS_MATRIX.md`; re-verify read-only with `bash scripts/demo-testnet-live.sh verify`.

## Repository

Intended public repository:

```text
https://github.com/Tranquil-Flow/lp0013-token-suite
```

Public implementation repository for LP-0013 evaluation.

## Prize

- Prize: LP-0013 Token program improvements — authorities
- Focus: mint authority support for variable supply and fixed supply tokens
- RFP dependency: RFP-001 gated config authority / approval-style support

## What is implemented

### Core authority model

Crate: `admin-authority-core`

- `AuthorityType::MintTokens`
- `AuthorityInfo`
- deterministic authorization checks
- atomic authority rotation
- atomic authority revocation
- deterministic rejection after revocation
- documented error enum via `TokenError`

### Token state transitions

Crate: `mint-core`

- `MintDefinition`
- `TokenHolding`
- variable supply minting
- fixed supply represented by revoked mint authority
- zero amount rejection
- supply overflow rejection
- balance overflow rejection
- unchanged state on failed authorization

### Instruction layer

Crate: `mint-program`

- `Instruction::CreateMint`
- `Instruction::MintTo`
- `Instruction::SetMintAuthority`
- `ProgramState` in-memory harness
- explicit clone/commit points so failed instructions do not partially mutate state

### SDK

Crate: `mint-sdk`

- `TokenClient::create_variable_mint`
- `TokenClient::create_fixed_mint`
- `TokenClient::mint_to`
- `TokenClient::rotate_authority`
- `TokenClient::revoke_authority`
- `TokenClient::supply`
- `TokenClient::balance`
- `TokenClient::current_authority`

### CLI and examples

Crate: `mint-cli`

```bash
cargo run -p mint-cli -- demo-variable
cargo run -p mint-cli -- demo-fixed
```

Runnable examples:

```bash
cargo run -p variable-supply
cargo run -p fixed-supply
cargo run -p config-pda-gated
```

The config-PDA-gated example demonstrates the RFP-001-style gate in the offline harness by deriving a deterministic config authority and proving that only the matching gate can mint before revocation.

### IDL / SPEL status

Authoritative IDL — real `spel generate-idl` output for the corrected four-instruction guest (exactly what `make idl` emits):

```text
idl/admin-authority.idl.spel-generated.json              # rc1 pins (v0.2.0-rc1 / spel ed3bbedb)
idl/admin-authority.idl.spel-generated.rc3-testnet.json  # rc3 / testnet pins (v0.2.0-rc3 / spel 31e52c52)
```

Hand-written design reference (adds discriminators / execution / errors the generator omits; see its `metadata.caveats`):

```text
idl/admin-authority.idl.json
```

Spike sources for reproducibility:

```text
spel-spike/admin_authority_guest.rs
spel-spike/generate_idl.rs
spel-spike/live_lifecycle.rs
spel-spike/README.md
```

Status docs:

```text
docs/SPEL_STATUS.md
docs/LEZ_PROOF_LOG.md
```

The spel-generated IDL is authoritative for the on-chain surface. Both pin generations are **byte-identical** under the corrected guest (a cross-revision stability check across `v0.2.0-rc1` / spel `ed3bbedb` and `v0.2.0-rc3` = `cf3639d8` / spel `31e52c52`) and both emit the full `AuthorityInfo` / `MintDefinition` / `TokenHolding` account bodies, with `current_authority` as `option<array<u8,32>>`. The generator omits instruction discriminators (LEZ dispatches by enum-variant index), the `execution` block, declared errors, and the instruction-arg `Option<T>` inner type (`initial_authority`/`new_authority` stay `{"defined":"Option"}`, which is why the lifecycle driver passes a strongly-typed enum to `nssa`). The hand-written `idl/admin-authority.idl.json` documents those omitted pieces as a design reference, its `metadata.caveats` disclosing that its 8-byte discriminators are illustrative and its args model the offline mint-program call surface. See `docs/SPEL_STATUS.md` for the full diff.

Claims:

- offline Rust authority suite: proven
- real SPEL-generated IDL (authoritative): `make idl` output for the corrected four-instruction guest; the rc1 and rc3 pin generations are byte-identical (cross-revision stability) and emit full account bodies
- hand-written IDL: a test-guarded design reference documenting the discriminators / execution / errors the generator omits, with disclosed caveats
- **public-testnet deploy + authority lifecycle (2026-06-04, corrected guest): proven on `https://testnet.lez.logos.co/`** under real consensus and `RISC0_DEV_MODE=0` — ProgramId/ImageID `32335764…b0a9ce` (base58 `4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF`), deploy `5b39deec…85cb4ce0`, `create_mint` `7d1dcb04…6e44da74`, `create_holding` `520d080b…32be5893`, `mint_to(60)` `8c865d01…0f743f61`, `mint_to(40)` `c63168b7…5993d21` [accumulates → 100], `set_mint_authority(None)` `8c4b08b5…01f5a331`, post-revoke `mint_to` `6e92e605…374a972d1` never included; mint-PDA readback `authority=None, supply=100, decimals=6` and holding `balance=100`. The two accumulating mints demonstrate variable supply on chain, and the post-revoke rejection is the authority guard (`require_authority`), not an init side effect — the holding already exists (`mut`). Re-verify read-only: `bash scripts/demo-testnet-live.sh verify`.
- LEZ deploy + four-transaction lifecycle (archival structural-surface, 2026-05-17, local-sequencer corroboration): proved at the wire/framework layer on the local sequencer (`127.0.0.1:3040`, `RISC0_DEV_MODE=0`) — deploy + create_mint + mint_to + set_mint_authority + post-revoke mint_to, all txhashes captured in `docs/LEZ_PROOF_LOG.md` and `docs/BENCHMARKS.md`
- LEZ semantic release-candidate rerun (2026-05-18, local-sequencer corroboration): semantic guest deployed (deploy `b16831c0…04ab5`, block 49551) and full lifecycle exercised — `create_mint` (`7d582e7b…b6f9d63`), `mint_to(100)` (`c474cf82…d3c74f`), `set_mint_authority(None)` (`756ee393…7351c`), post-revoke `mint_to` rejected (`27df9483…2bff`), decoded mint PDA shows `supply=100, current_authority=None, decimals=6`
- Independent post-rerun re-verification (2026-05-18 localnet, ~22 min later): re-submitted `set_mint_authority` (`cea5b8c7…fc5eb4`) was rejected on chain with `Program error 2008: authority has been revoked` — the canonical `require_authority` panic from the guest body, observed live on the local sequencer (whose logs, unlike the public testnet's, expose the exact guest error string); complements the testnet state-level invariant above
- in-guest authority semantics: source ported in `spel-spike/admin_authority_guest.rs` — `mint_to` enforces nonzero amount, current-authority authorization, revoked-authority rejection, supply/balance overflow checks, and post-state writes, with the holding account claimed as a program-owned PDA on first mint; `set_mint_authority` enforces current-authority authorization and persists rotation/revocation

## Local verification

Run:

```bash
bash scripts/check-prereqs.sh
bash scripts/demo.sh
bash scripts/preflight-localnet-e2e.sh --report
```

`check-prereqs.sh` validates submission artifacts and runs:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Why this should be competitive

- Small, focused authority model with fixed-supply and variable-supply sharing one code path.
- Revocation is represented by `None`, making post-revocation behavior deterministic and easy to audit.
- Failed instructions leave state unchanged.
- The SDK and examples are easy for evaluators to run.
- The compliance docs explicitly separate proven offline behavior, host wire-level LEZ evidence, and the semantic guest source port.
- The implementation avoids a shell-out wallet dependency in the tested Rust API, sidestepping the failure mode that affected the earlier LP-0013 attempt.

## Demo video

Fresh narrated demo: https://youtu.be/rUgsCCPiQfo. This recording demonstrates the corrected public-testnet lifecycle and is the final video evidence for LP-0013.

## λPrize PR

Submitted upstream as <https://github.com/logos-co/lambda-prize/pull/77>.

## Suggested PR checklist

- [x] Public repo URL is correct.
- [x] README quick start works from a fresh clone.
- [x] `bash scripts/check-prereqs.sh` passes.
- [x] `bash scripts/demo.sh` passes.
- [x] `docs/SPEC_COMPLIANCE.md` is current.
- [x] `docs/SPEL_STATUS.md` reflects the corrected four-instruction guest status.
- [x] `idl/admin-authority.idl.spel-generated.json` is the authoritative IDL for the corrected four-instruction surface; `idl/admin-authority.idl.json` is clearly marked as a hand-written design reference with disclosed caveats.
- [x] `RISC0_DEV_MODE=0` proof logs are captured for public testnet and standalone local sequencer.
- [x] Standalone local-sequencer e2e is included in CI config (`local-sequencer-e2e-preflight` plus manual self-hosted `local-sequencer-e2e`) and has a real 2026-06-09 prepared-host run recorded in `docs/LEZ_PROOF_LOG.md`. Hosted CI passed for this CI/preflight evidence path; final branch CI should be checked from GitHub Actions after each push.
- [x] Demo video re-recorded against the public-testnet lifecycle and link updated: https://youtu.be/rUgsCCPiQfo.
- [x] No private keys, seeds, credentials, or private chat excerpts are committed.
- [x] No AI attribution is present in commit messages.
