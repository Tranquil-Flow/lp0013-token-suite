# LP-0013 — resubmission status (authoritative)

**As of 2026-06-04. Read this before any other doc's testnet claims.**

PR [#57](https://github.com/logos-co/lambda-prize/pull/57) was **closed** by the reviewer (weboko). The on-chain correctness fix is now **landed in source and verified on the public testnet** (2026-06-04): the corrected four-instruction guest (ImageID/ProgramId `32335764…b0a9ce`) was re-deployed to `testnet.lez.logos.co` and the full lifecycle re-captured — two accumulating mints (60+40 → 100, variable supply on chain) and a guard-rejected post-revoke mint. The fresh narrated demo video is recorded at https://youtu.be/rUgsCCPiQfo. All six of the reviewer's points are addressed below. The remaining outward-facing step is the Logos resubmission PR (Evi-gated). Treat this file as the source of truth.

## Reviewer's points (weboko, 2026-06-04)

1. e2e tests vs a LEZ sequencer (standalone) in CI — absent.
2. A reproducible demo script that **runs** the e2e from a clean clone vs a real **local** sequencer, `RISC0_DEV_MODE=0` (not offline re-query).
3. The deployable SPEL program/scaffold must **live in the repo** and be deployable by an evaluator — local-only hashes are not evidence.
4. Real, reproducible CU on LEZ (not single-run executor-ms on a private node).
5. **Demonstrate revoked-authority mint rejection on-chain via the authority guard, NOT via an `AccountAlreadyInitialized` side effect; fix the init-only holding PDA so repeat / variable-supply minting works on chain.**
6. SDK/IDL targeting the deployed program (SPEL IDL with accounts/types/errors + a client that talks to LEZ, not the in-memory harness).

## The real bug (#5) and the fix — DONE in source + verified on chain (2026-06-04)

The deployed SPEL guest declared the recipient holding `#[account(init, pda)]`. In SPEL, `init` maps to a **non-idempotent** claim (`new_claimed`), and the macro forces the attribute-derived claim (the handler can't substitute the idempotent `new_claimed_if_default` that nssa's own token program uses). Therefore:

- A **second** mint to the same holding failed (`init` rejects an already-claimed account) → **variable-supply minting was broken on chain**.
- The prior testnet lifecycle did **one** mint, then revoked, then a post-revoke mint that hit the **already-initialized** holding — so it was rejected by the `init` side effect **before `require_authority` ran**. The authority guard (error 2008) was never actually exercised on chain. The single-mint lifecycle masked this.

**Fix (landed):** split into two instructions (the SPL/associated-token-account pattern):
- `create_holding(#[account(init, pda)] holding, #[account(signer)] payer)` — claims the holding once.
- `mint_to(#[account(mut, pda)] mint, #[account(mut, pda)] recipient_holding, #[account(signer)] authority, amount)` — holding is `mut`, so repeated mints accumulate, and `require_authority` runs before any state change (post-revoke rejection is genuinely the guard error, not a side effect).

Applied to the in-repo buildable program (`onchain-program/methods/guest/src/bin/admin_authority_spike.rs`) and its reading mirror (`spel-spike/admin_authority_guest.rs`). The offline harness was already correct; a new contract test `mint-program::variable_supply_allows_repeated_minting_to_same_holding` locks the contract (mint 50+50=100; post-revoke mint to the **existing** holding → `AuthorityRevoked`). `cargo test --workspace` is green.

## Status per point

| # | Status |
|---|--------|
| 5 | **DONE — verified on chain (2026-06-04).** Guest split into `create_holding` + mutable `mint_to`; `cargo test --workspace` green (incl. the repeated-mint / post-revoke-guard contract test). On the public testnet the two mints `mint_to(60)` + `mint_to(40)` both confirmed and accumulated to `supply=100` / holding `balance=100` (variable supply on chain), and the post-revoke `mint_to` was rejected — because the holding already exists (`mut`), the rejection is the authority guard (`require_authority`, error 2008), not an init side effect; the mint PDA reads `authority=None, supply=100`. Re-verify: `bash scripts/demo-testnet-live.sh verify`. |
| 3 | **DONE.** The buildable, deployable program is in the repo at `onchain-program/` (RISC0 guest + deploy/lifecycle driver; a nested workspace `exclude`d from the parent so the per-push CI gate stays toolchain-light). The reproducible `cargo risczero build` produced ImageID `32335764…b0a9ce`, which deployed and ran on the public testnet. |
| 6 | **DONE.** LEZ client `onchain-program/examples/src/bin/live_lifecycle.rs` (four-instruction surface: `create_holding`, two accumulating mints 60+40→100, holding readback, post-revoke guard rejection) **drove the lifecycle on the public testnet for real** — it talks to the LEZ sequencer, not the in-memory harness. SPEL IDL regenerated for the four-instruction surface (`idl/admin-authority.idl.spel-generated*.json` — rc1≡rc3 byte-identical, full account bodies); the hand-written IDL is reframed as a design reference with disclosed caveats (illustrative discriminators; offline-API-shaped args). |
| 2 | **DONE.** `scripts/demo-localnet.sh` builds `onchain-program/` and drives the full corrected lifecycle against a local sequencer under `RISC0_DEV_MODE=0` (`--check` reports prerequisites); `scripts/demo-testnet-live.sh verify` re-runs the public-testnet evidence read-only from a clean clone. |
| 1 | **Partial / honestly constrained.** The full on-chain lifecycle has been run e2e against the public LEZ sequencer (the 2026-06-04 run). The offline suite + the doc validator + its self-tests run per-push in CI; a manual `workflow_dispatch` job (`testnet-verify` in `.github/workflows/ci.yml`) re-verifies the on-chain evidence read-only. Hosted CI still cannot build the RISC0 guest or run a sequencer/faucet, so the full on-chain e2e is not a per-push gate (documented in `docs/SPEC_COMPLIANCE.md`). |
| 4 | **Framing reconciled.** CU methodology stands (deterministic deployed-ELF executor cycles; the public testnet exposes no per-tx CU, so per-op CU is only observable on a local sequencer whose log you control). The committed numbers in `docs/BENCHMARKS.md` are from the pre-fix semantic guest's localnet sessions, clearly labeled; the corrected four-op profile re-measure on a local sequencer is a documented follow-up. Note: any such number is single-run private-node CU — the inherent platform caveat weboko raised, unresolvable on the public testnet which hides per-tx CU. |
| — | **Ready for Evi-gated resubmission:** fresh narrated video recorded (https://youtu.be/rUgsCCPiQfo); open the new Logos PR after final validator pass and explicit sign-off. |

## Canonical on-chain evidence (2026-06-04, corrected guest)

ProgramId == ImageID `32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce` (base58 `4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF`), on `testnet.lez.logos.co` (`RISC0_DEV_MODE=0`):

| step | tx | verdict |
|---|---|---|
| deploy_program | `5b39deec…85cb4ce0` | `Some(ProgramDeployment)` |
| create_mint | `7d1dcb04…6e44da74` | `Some(Public)` |
| create_holding | `520d080b…32be5893` | `Some(Public)` |
| mint_to(60) | `8c865d01…0f743f61` | `Some(Public)` |
| mint_to(40) | `c63168b7…5993d21` | `Some(Public)` |
| set_mint_authority(None) | `8c4b08b5…01f5a331` | `Some(Public)` |
| mint_to(post-revoke) | `6e92e605…374a972d1` | `Transaction is None` (guard-rejected) |

Mint PDA `HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu` decodes `authority=None, supply=100, decimals=6`; holding PDA `4yswbZaRR1HQt4a5HS4uN7nLvAwL1txHTMSXKo1WZH2S` balance=100. Signer/authority: `B6Sa77taeQgQ3FXHP88wjs15sJw3EyfcRjnSAZKnYchb`. Full log: `docs/LEZ_PROOF_LOG.md`; re-verify read-only with `bash scripts/demo-testnet-live.sh verify`.

### Superseded (do not cite as fix evidence)

The pre-fix **2026-06-03** run used a different ImageID and the single-use `init` holding bug — do not cite its hashes (`07561014…` deploy, `0540648f…` set-authority, `312ea9f1…` post-revoke mint, ProgramId `4153e159…`, ImageID `59e15341…`) as proof of the fix. They are retained in `docs/LEZ_PROOF_LOG.md` as a historical record only.
