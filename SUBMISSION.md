# LP-0013 Token Authorities — Submission Draft

## Summary

This submission implements token mint authority lifecycle support for Logos λPrize LP-0013.

It provides a self-contained Rust workspace proving the core authority model, token mint state transitions, instruction-level semantics, SDK ergonomics, CLI demos, runnable examples, CI, a canonical IDL artifact, real SPEL-generated IDL evidence, and a SPEL guest source that ports the same authority semantics into the RISC0/LEZ account adapter. The current tree is locally green for the offline suite; host-side SPEL/LEZ evidence is recorded in `docs/LEZ_PROOF_LOG.md`.

## Repository

Intended public repository:

```text
https://github.com/Tranquil-Flow/lp0013-token-suite
```

Do not push or open a Logos PR without explicit Evi sign-off.

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

Canonical IDL (hand-written, test-guarded superset):

```text
idl/admin-authority-idl.json
```

Real SPEL-generated IDL (host spike, 2026-05-17):

```text
idl/admin-authority-idl.spel-generated.json
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

The hand-written IDL ships as the canonical artifact because it carries detail the current SPEL revision does not yet emit (instruction discriminators, `execution` block, declared errors, expanded `Option<T>` and nested account-type bodies). The SPEL-generated IDL is checked in alongside as evidence that the same surface re-emerges from the real tool against the same LEZ pin (`v0.2.0-rc1`) and SPEL revision (`ed3bbedb4b684645da05455d30a4a0be7cc4dfe0`) that LP-0017 uses.

Claims:

- offline Rust authority suite: proven
- canonical hand-written IDL: documented and test-guarded
- real SPEL-generated IDL: regenerated from the semantic guest and committed alongside as additive evidence
- LEZ deploy + four-transaction lifecycle (archival structural-surface, 2026-05-17): proved at the wire/framework layer on the local sequencer (`127.0.0.1:3040`, `RISC0_DEV_MODE=0`) — deploy + create_mint + mint_to + set_mint_authority + post-revoke mint_to, all txhashes captured in `docs/LEZ_PROOF_LOG.md` and `docs/BENCHMARKS.md`
- LEZ semantic release-candidate rerun (2026-05-18): semantic guest deployed (deploy `b16831c0…04ab5`, block 49551) and full lifecycle exercised — `create_mint` (`7d582e7b…b6f9d63`), `mint_to(100)` (`c474cf82…d3c74f`), `set_mint_authority(None)` (`756ee393…7351c`), post-revoke `mint_to` rejected (`27df9483…2bff`), decoded mint PDA shows `supply=100, current_authority=None, decimals=6`
- Independent post-rerun re-verification (same day, ~22 min later): re-submitted `set_mint_authority` (`cea5b8c7…fc5eb4`) was rejected on chain with `Program error 2008: authority has been revoked` — the canonical `require_authority` panic from the guest body, observed live on the sequencer; this is direct on-chain semantic proof of the post-revocation guard, complementing the framework-layer rejection captured during the first rerun
- in-guest authority semantics: source ported in `spel-spike/admin_authority_guest.rs` — `mint_to` enforces nonzero amount, current-authority authorization, revoked-authority rejection, supply/balance overflow checks, and post-state writes, with the holding account claimed as a program-owned PDA on first mint; `set_mint_authority` enforces current-authority authorization and persists rotation/revocation

## Local verification

Run:

```bash
bash scripts/check-prereqs.sh
bash scripts/demo.sh
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

Narrated terminal demo (RISC0_DEV_MODE=0 live LEZ lifecycle): <https://youtu.be/3hQd2G8O-UM>

## λPrize PR

Submitted upstream as <https://github.com/logos-co/lambda-prize/pull/57>.

## Suggested PR checklist

- [ ] Public repo URL is correct.
- [ ] README quick start works from a fresh clone.
- [ ] `bash scripts/check-prereqs.sh` passes.
- [ ] `bash scripts/demo.sh` passes.
- [ ] `docs/SPEC_COMPLIANCE.md` is current.
- [ ] `docs/SPEL_STATUS.md` reflects the host-side spike and release-candidate semantic guest status.
- [ ] `idl/admin-authority-idl.json` is clearly marked as the canonical hand-written superset, with SPEL-generated evidence checked in alongside.
- [ ] `RISC0_DEV_MODE=0` proof logs are captured.
- [x] Demo video link is included — <https://youtu.be/3hQd2G8O-UM>.
- [ ] No private keys, seeds, credentials, or private chat excerpts are committed.
- [ ] No AI attribution is present in commit messages.
