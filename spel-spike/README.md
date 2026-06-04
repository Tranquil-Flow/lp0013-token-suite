# LP-0013 SPEL Spike (snapshot)

> **The canonical, buildable, deployable program now lives in [`../onchain-program/`](../onchain-program/).** This directory holds loose source snapshots mirroring the `onchain-program` files for at-a-glance reading and reproducibility. They are kept in sync with `onchain-program` but are **not** themselves wired into a build (`onchain-program` is a nested workspace excluded from the parent — see its README).

Real SPEL guest source used to generate `idl/admin-authority.idl.spel-generated.json`.

## Files

(each file mirrors its counterpart under `../onchain-program/`)

- `admin_authority_guest.rs` — the `#[lez_program]` guest module (mirror of `../onchain-program/methods/guest/src/bin/admin_authority_spike.rs`). Four instructions (`create_mint`, `create_holding`, `mint_to`, `set_mint_authority`) and three account types (`AuthorityInfo`, `MintDefinition`, `TokenHolding`). It decodes Borsh account state, enforces authority/revocation checks, accumulates supply and balance across repeated mints, and persists authority rotation/revocation. The recipient holding is claimed once by `create_holding` (`init`) and then mutated by `mint_to` (`mut`), so repeated mints accumulate and the authority guard runs before any write. The offline `mint-core` remains the canonical tested reference model.
- `generate_idl.rs` — driver that invokes `spel_framework::generate_idl!` to emit the IDL (mirror of `../onchain-program/examples/src/bin/generate_idl.rs`).
- `live_lifecycle.rs` — host driver that deploys and exercises the full lifecycle (mirror of `../onchain-program/examples/src/bin/live_lifecycle.rs`): `create_mint`, `create_holding`, two accumulating `mint_to` calls (60 + 40 → 100), `set_mint_authority(None)`, and a post-revoke `mint_to` rejected by the guard, then reads back the mint + holding PDAs. Bypasses the IDL CLI because the generator does not expand `Option<T>` args. See the inline doc comment for env vars.

## Reproduce — IDL generation

```bash
# 1. Scaffold a fresh SPEL project pinned to the same LEZ / SPEL revs the
#    on-host LP-0017 build uses.
spel -- init admin_authority_spike \
    --lez-tag v0.2.0-rc1 \
    --spel-rev ed3bbedb4b684645da05455d30a4a0be7cc4dfe0

cd admin_authority_spike

# 2. Replace the scaffold's placeholder guest + IDL driver with this spike's
#    files.
cp .../spel-spike/admin_authority_guest.rs \
    methods/guest/src/bin/admin_authority_spike.rs
cp .../spel-spike/generate_idl.rs \
    examples/src/bin/generate_idl.rs

# 3. Generate the IDL with the real SPEL framework.
make idl
cat admin_authority_spike-idl.json
```

The resulting IDL is what is committed at `idl/admin-authority.idl.spel-generated.json` (authoritative for the on-chain surface). The hand-written `idl/admin-authority.idl.json` is a design reference documenting the instruction discriminators / execution metadata / named errors the generator omits — with `metadata.caveats` disclosing that those discriminators are illustrative and its args model the offline API.

## Reproduce — live lifecycle

```bash
# 4. Build the RISC0 guest binary (uses cargo-risczero docker build).
cargo risczero build --manifest-path methods/guest/Cargo.toml

# 5. Deploy the program against a running local LEZ sequencer.
export NSSA_WALLET_HOME_DIR=~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/wallet
wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin

# 6. Drive the lifecycle (create_mint → create_holding → mint_to(60) → mint_to(40) →
#    set_mint_authority(None) → mint_to-post-revoke[rejected] → read mint + holding state).
cp .../spel-spike/live_lifecycle.rs examples/src/bin/live_lifecycle.rs
# also add the host-side deps: nssa, common, wallet, sequencer_service_rpc, borsh, serde,
# risc0-zkvm to examples/Cargo.toml — see the live_lifecycle.rs doc-comment for the full set.
export LP0013_PROGRAM_BIN=methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
cargo run --bin live_lifecycle
```

See `docs/LEZ_PROOF_LOG.md` for the host environment, the four captured transaction hashes, and the read-back mint-PDA state.
