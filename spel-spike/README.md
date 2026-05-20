# LP-0013 SPEL Spike

Real SPEL guest source used to regenerate `idl/admin-authority-idl.spel-generated.json` against the on-host Logos toolchain. This directory is documentation/evidence — it is **not** wired into the workspace build to keep the offline submission free of the macOS arm64 RISC0 guest dependency conflict (see `docs/SPEL_STATUS.md`).

## Files

- `admin_authority_guest.rs` — `#[lez_program]` guest module mirroring the three offline instructions (`create_mint`, `mint_to`, `set_mint_authority`) and three account types (`AuthorityInfo`, `MintDefinition`, `TokenHolding`). Drop this in as `methods/guest/src/bin/admin_authority.rs` inside a `spel init` scaffold. The guest now includes the semantic account-adapter port: it decodes Borsh account state, enforces authority/revocation checks, updates supply and balances, and persists authority rotation/revocation. The offline `mint-core` remains the canonical tested reference model.
- `generate_idl.rs` — driver that invokes `spel_framework::generate_idl!` to emit the IDL. Drop this in as `examples/src/bin/generate_idl.rs` inside the scaffold.
- `live_lifecycle.rs` — host driver that signs and submits four real transactions against the live LEZ sequencer (`create_mint`, `mint_to(100)`, `set_mint_authority(None)`, `mint_to(post-revoke)`) and reads back the mint PDA. Bypasses the IDL CLI because the current SPEL revision does not expand `Option<T>` args. Drop this in as `examples/src/bin/live_lifecycle.rs` inside the scaffold; see the inline doc comment for env vars.

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

The resulting IDL is what was committed alongside the fallback at `idl/admin-authority-idl.spel-generated.json`. The hand-written `idl/admin-authority-idl.json` is the documented superset (instruction discriminators, execution metadata, named errors, expanded type signatures).

## Reproduce — live lifecycle

```bash
# 4. Build the RISC0 guest binary (uses cargo-risczero docker build).
cargo risczero build --manifest-path methods/guest/Cargo.toml

# 5. Deploy the program against a running local LEZ sequencer.
export NSSA_WALLET_HOME_DIR=~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/wallet
wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin

# 6. Drive the lifecycle (create_mint → mint_to → set_mint_authority → mint_to-post-revoke → read state).
cp .../spel-spike/live_lifecycle.rs examples/src/bin/live_lifecycle.rs
# also add the host-side deps: nssa, common, wallet, sequencer_service_rpc, borsh, serde,
# risc0-zkvm to examples/Cargo.toml — see the live_lifecycle.rs doc-comment for the full set.
export LP0013_PROGRAM_BIN=methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
cargo run --bin live_lifecycle
```

See `docs/LEZ_PROOF_LOG.md` for the host environment, the four captured transaction hashes, and the read-back mint-PDA state.
