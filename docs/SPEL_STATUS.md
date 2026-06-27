# SPEL / IDL Integration Status

> **✅ RESOLVED (2026-06-27) — corrected guest deployed + verified on the public testnet; SPEL IDL regenerated.** The correctness fix adds a `create_holding` instruction and changes `mint_to` (holding `init` → `mut`). The SPEL-generated IDL artifacts have been **regenerated** from the corrected four-instruction guest (committed at `idl/admin-authority.idl.spel-generated.json` and `…rc3-testnet.json`), and the corrected guest is deployed + verified on `testnet.lez.logos.co` (ProgramId/ImageID `32335764…b0a9ce`). The 2026-06-03 testnet run referenced below is the superseded pre-fix record. Authoritative state → [`../RESUBMISSION_STATUS.md`](../RESUBMISSION_STATUS.md).

Status: host-side spike landed, the guest source was advanced from structural surface proof to semantic account-adapter source, and the **corrected** guest (single-use `init` holding → `create_holding` + mutable `mint_to`) was deployed and exercised against the **public LEZ testnet** on 2026-06-27 (`testnet.lez.logos.co`, `RISC0_DEV_MODE=0`; ProgramId/ImageID `32335764…b0a9ce`) — two accumulating mints prove variable supply on chain and the post-revoke mint is guard-rejected (see `docs/LEZ_PROOF_LOG.md`). The earlier 2026-06-03 run was the pre-fix build and is retained as history. Real `spel_framework::generate_idl!` output for the corrected four-instruction surface (`create_mint`, `create_holding`, `mint_to`, `set_mint_authority`) is committed as the **authoritative** IDL at `idl/admin-authority.idl.spel-generated.json`. The same generator is run under both pin sets and committed twice: the rc1 pins (`v0.2.0-rc1` / spel `ed3bbedb`, matching LP-0017) at that path, and the v0.2.0 / current testnet pins (`v0.2.0` / spel `31e52c52`) at `idl/admin-authority.idl.spel-generated.rc3-testnet.json`. Under the corrected guest (which annotates `#[account_type]`) the two are **byte-identical** — a cross-revision stability check (see below). The `idl/admin-authority.idl.json` file is retained as a hand-written design reference (it adds discriminators, an `execution` block, and a declared error set the generator omits), but it is no longer described as "canonical": the spel-generated IDL is authoritative for the on-chain account/arg surface, and the hand-written file's `metadata.caveats` disclose where its discriminators (illustrative) and args (offline-API-shaped) diverge from the on-chain program.

## Toolchain on host (2026-05-17)

```text
cargo            1.94.0 (85eff7c80 2026-01-15)
rustc            1.94.0 (4a4ef493e 2026-03-02)
spel             present at ~/.cargo/bin/spel (binary built 2026-05-04, no --version flag)
lgs              logos-scaffold 0.1.1
logos-scaffold   0.1.1
cargo-risczero   3.0.5
gh               2.76.2
```

The same on-host toolchain proved out LP-0017, which gives confidence that the SPEL surface used here is the production-shaped one rather than a stale dev copy.

## What was checked

A `spel init` scaffold pinned to the same LEZ tag (`v0.2.0-rc1`) and SPEL revision (`ed3bbedb4b684645da05455d30a4a0be7cc4dfe0`) used by LP-0017 was created in a sibling directory. The placeholder guest was replaced with a `#[lez_program]` module mirroring the LP-0013 offline instruction surface (`create_mint`, `mint_to`, `set_mint_authority`) and the three account types (`AuthorityInfo`, `MintDefinition`, `TokenHolding`). The IDL driver (`examples/src/bin/generate_idl.rs`) ran cleanly:

```bash
cd admin_authority_spike
make idl
# cargo run --bin generate_idl > admin_authority_spike-idl.json
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.45s
# ✅ IDL written to admin_authority_spike-idl.json
```

The generated output is committed at `idl/admin-authority.idl.spel-generated.json`. The spike sources are checked in under `spel-spike/` as reproducible evidence.

## The spel-generated IDL vs the hand-written design reference

The spel-generated IDL is the **authoritative** artifact: it is what `make idl`
(`spel_framework::generate_idl!`) emits from the deployed guest, so it always
matches the program. For the corrected guest it captures the four instructions,
their PDA seeds, account access modes (signer/writable/init), primitive arg
types, and the **full account bodies** — `AuthorityInfo`, `MintDefinition`,
`TokenHolding` with every field, `current_authority` as `option<array<u8,32>>`.

`spel generate-idl` for this surface still does **not** emit:

- per-instruction discriminators (the LEZ runtime dispatches by enum-variant
  index — `create_mint`=0, `create_holding`=1, `mint_to`=2,
  `set_mint_authority`=3 — so discriminators are not needed),
- the `execution` block (public / private_owned),
- declared error codes (`errors` is `[]`),
- **instruction-arg** `Option<T>` inner-type expansion — `initial_authority` /
  `new_authority` are emitted as a bare `{"defined": "Option"}` (this is why the
  lifecycle driver passes a strongly-typed enum to `nssa` rather than going
  through the IDL-driven CLI for those args),
- the standalone `types` table and per-account comments.

The hand-written `idl/admin-authority.idl.json` documents all of those (it is the
only place the `TokenError` set and the `execution` blocks are written down), but
it is a **design reference, not the canonical IDL**. It declares
`metadata.generation = "hand-written"`, is test-guarded by
`admin_authority_spel::tests::fallback_idl_documents_token_authority_surface`,
and its `metadata.caveats` disclose the two ways it intentionally diverges from
the program: its 8-byte discriminators are illustrative (real dispatch is by
variant index), and its instruction `args` model the offline mint-program
flattened call surface rather than the on-chain account/arg split. For anything
about the actual on-chain surface, the spel-generated IDL is authoritative.

## Hand-written design-reference artifact

`idl/admin-authority.idl.json` is retained as a hand-written design reference,
not as the canonical on-chain IDL. The authoritative on-chain artifact is the
SPEL-generated four-instruction IDL:

- `idl/admin-authority.idl.spel-generated.json`
- `idl/admin-authority.idl.spel-generated.rc3-testnet.json`

Those generated files mirror the corrected deployed surface:

- `create_mint`
- `create_holding`
- `mint_to`
- `set_mint_authority`

and include the account types needed by `spel inspect`-style consumers:

- `AuthorityInfo`
- `MintDefinition`
- `TokenHolding`

The hand-written design reference is test-guarded by:

```bash
cargo test -p admin-authority-spel fallback_idl_documents_token_authority_surface
```

## LEZ proof and semantic release-candidate rerun

`docs/LEZ_PROOF_LOG.md` records four proof sessions:

- **2026-06-27 — public testnet, CORRECTED guest (load-bearing):** the corrected four-instruction guest (ImageID/ProgramId `32335764…b0a9ce`, base58 `4UARaVcJJoLxebFAobocsZyzpJ5TTUvvhRtFuHtuHypd`) was deployed and exercised on `testnet.lez.logos.co` under `RISC0_DEV_MODE=0`: deploy `5b39deec…85cb4ce0` (`Some(ProgramDeployment)`), `create_mint` `7d1dcb04…6e44da74`, `create_holding` `520d080b…32be5893`, `mint_to(60)` `8c865d01…0f743f61`, `mint_to(40)` `c63168b7…5993d21` [accumulates → 100], `set_mint_authority(None)` `8c4b08b5…01f5a331`, post-revoke `mint_to` never included (`6e92e605…374a972d1` → `chain-info` None). Live mint-PDA readback decodes `authority=None, supply=100, decimals=6`; holding `balance=100`. Two accumulating mints prove variable supply on chain, and the post-revoke rejection is the authority guard (`require_authority`), not an init side effect (the holding already exists, `mut`). Re-verify read-only with `bash scripts/demo-testnet-live.sh verify`.
- **2026-06-03 — public testnet (historical pre-fix run):** the semantic guest, rebuilt against the testnet's rc3 pins (ImageID `59e15341…fb0ae2c`, ProgramId `4153e159…2caeb08f`), was deployed and exercised on `testnet.lez.logos.co` under `RISC0_DEV_MODE=0`: deploy `07561014…cb968ba` (`Some(ProgramDeployment)`), `create_mint` `17d90ea6…eeb37d2`, `mint_to(100)` `be393bcf…f29bbd8`, `set_mint_authority(None)` `0540648f…b29f5784`, post-revoke `mint_to` never included (`312ea9f1…8fef4798` → `chain-info` None). Live mint-PDA readback decodes `authority=None, supply=100, decimals=6`. **This run used the pre-fix guest:** `supply=100` is a single mint (the single-use `init` bug), not demonstrated variable supply, and the post-revoke rejection came from the `init` side effect, not the authority guard. Retained only as history; superseded by the 2026-06-27 corrected run above.
- 2026-05-17 — structural-surface spike (localnet, corroboration): real IDL generation, RISC0 guest build, live local-sequencer deploy, four signed lifecycle transactions. Proved the SPEL/LEZ wire path and exposed the semantic gap in the first spike.
- 2026-05-18 — semantic release-candidate rerun (localnet, corroboration): the guest source was advanced from structural stub to semantic source (`mint_to` decodes mint/holding state, enforces nonzero amount, current-authority authorization, revoked-authority rejection, and supply/balance overflow checks, then writes updated post-states; `set_mint_authority` enforces the current authority and persists rotation/revocation; holding accounts are claimed as program PDAs so the LEZ executor accepts the mutation). The semantic guest was rebuilt (ImageID `58470667…d0b960`, 480,352-byte ELF), redeployed (deploy tx `b16831c0…04ab5`, block 49551), and re-driven: `create_mint` confirmed (`7d582e7b…b6f9d63`), `mint_to(100)` confirmed (`c474cf82…d3c74f`), `set_mint_authority(None)` confirmed (`756ee393…7351c`), post-revoke `mint_to` rejected (`27df9483…2bff`). This localnet run additionally captured the exact guest panic (`Program error 2008: authority has been revoked`) that the testnet's hidden sequencer logs cannot surface.

The **2026-06-27 run is the load-bearing on-chain evidence** (corrected four-instruction guest). The 2026-06-03 testnet run and the two 2026-05 localnet sessions predate the correctness fix and are retained as corroboration that the SPEL/LEZ wire path and authority semantics work end-to-end (the 2026-05-18 localnet run additionally captured the exact `Program error 2008` guard panic the testnet's hidden logs cannot surface). Ready now: the offline release gates, the corrected source (`create_holding` + mutable `mint_to`), the SPEL-generated IDL (regenerated for the four-instruction surface; rc1 and rc3 generations byte-identical), the deployed + verified testnet lifecycle, and the final narrated demo video linked from the submission files.

## v0.2.0 / current testnet-matching IDL regeneration (2026-06-03)

To match the program pins used by the public testnet, the IDL is regenerated from the corrected guest under the testnet pins (`spel-framework`/`spel` rev `31e52c52`, `nssa_core` tag `v0.2.0`) and committed at `idl/admin-authority.idl.spel-generated.rc3-testnet.json`. Both generations now emit the full `AuthorityInfo` / `MintDefinition` / `TokenHolding` account bodies (with `current_authority` as `option<array<u8,32>>`), because the corrected guest annotates the account structs with `#[account_type]`. Re-running the generator under the rc1 pins (`spel` rev `ed3bbedb`, `v0.2.0-rc1`) produces a **byte-identical** file — verified in `tests/test_validate_submission_docs.py::test_idl_claims_match_generated_artifact_limitations`. We retain both pin-labeled files as a cross-revision stability check; they are not richer-vs-poorer. The differences from the hand-written design reference (illustrative discriminators, `errors`/`types`/`execution` documented only by hand, instruction-arg `Option`s emitted as bare `{"defined":"Option"}`) are documented above.
