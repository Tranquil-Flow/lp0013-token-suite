# SPEL / IDL Integration Status

Status: host-side spike landed, the guest source was advanced from structural surface proof to semantic account-adapter source, and the semantic guest has been redeployed and re-driven against the live local LEZ sequencer on 2026-05-18. Real `spel generate-idl` output for the LP-0013 instruction surface (regenerated against the semantic guest) is committed alongside the hand-written fallback as additive evidence; both are tracked, and the hand-written one remains test-guarded and shipped as the canonical IDL because it carries the richer detail evaluators need.

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

The generated output is committed at `idl/admin-authority-idl.spel-generated.json`. The spike sources are checked in under `spel-spike/` as reproducible evidence.

## Why the hand-written IDL is still the canonical artifact

The SPEL-generated IDL captures the wire-essential surface — instruction names, PDA seeds, account access modes (signer/writable/init), and primitive arg types — but the current SPEL framework revision does not yet emit:

- per-instruction discriminators,
- the `execution` block (public / private_owned),
- declared error codes,
- expanded `Option<T>` / nested account-type bodies,
- per-account comments.

The hand-written fallback at `idl/admin-authority-idl.json` includes all of those, declares its `metadata.generation = "hand-written"`, and is test-guarded by `admin_authority_spel::tests::fallback_idl_documents_token_authority_surface`. The two artifacts agree on the instruction set. The fallback is the documented superset: this SPEL revision leaves `accounts` / `types` empty for this surface and does not emit the richer account bodies, discriminators, execution metadata, or errors.

## Fallback artifact

`idl/admin-authority-idl.json` is the canonical IDL shipped with the submission. It mirrors the proven offline instruction surface:

- `create_mint`
- `mint_to`
- `set_mint_authority`

And documents the account types needed by `spel inspect`-style consumers:

- `AuthorityInfo`
- `MintDefinition`
- `TokenHolding`

The fallback IDL is test-guarded by:

```bash
cargo test -p admin-authority-spel fallback_idl_documents_token_authority_surface
```

## LEZ proof and semantic release-candidate rerun

`docs/LEZ_PROOF_LOG.md` records two host proof sessions:

- 2026-05-17 — structural-surface spike: real IDL generation, RISC0 guest build, live local-sequencer deploy, four signed lifecycle transactions. Proved the SPEL/LEZ wire path and exposed the semantic gap in the first spike.
- 2026-05-18 — semantic release-candidate rerun: the guest source was advanced from structural stub to semantic source (`mint_to` decodes mint/holding state, enforces nonzero amount, current-authority authorization, revoked-authority rejection, and supply/balance overflow checks, then writes updated post-states; `set_mint_authority` enforces the current authority and persists rotation/revocation; holding accounts are claimed as program PDAs so the LEZ executor accepts the mutation). The semantic guest was rebuilt (ImageID `58470667…d0b960`, 480,352-byte ELF), redeployed (deploy tx `b16831c0…04ab5`, block 49551), and re-driven: `create_mint` confirmed (`7d582e7b…b6f9d63`), `mint_to(100)` confirmed (`c474cf82…d3c74f`), `set_mint_authority(None)` confirmed (`756ee393…7351c`), post-revoke `mint_to` rejected (`27df9483…2bff`). The decoded mint PDA after the run shows `supply=100, current_authority=None, decimals=6`.

The semantic rerun supersedes the archival structural-surface evidence. Offline release gates, source-level semantic port, on-chain semantic LEZ proof, and SPEL-generated IDL are ready; the only non-technical deliverable still required before the public PR is the narrated demo video.
