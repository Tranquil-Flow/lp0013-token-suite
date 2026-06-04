# onchain-program — LP-0013 deployable LEZ/SPEL program

> **✅ Evidence state (2026-06-04): deployed + verified on the public testnet.** The
> guest source in this directory carries the correctness fix (single-use `init`
> holding PDA → `create_holding` + mutable `mint_to`). It was built reproducibly
> (`cargo risczero build` → ImageID/ProgramId `32335764…b0a9ce`), deployed to
> `testnet.lez.logos.co` (`RISC0_DEV_MODE=0`), and the full lifecycle was captured:
> two accumulating mints (60+40 → 100, variable supply on chain) and a post-revoke
> mint rejected by the authority guard (`require_authority`), not by an `init` side
> effect. Re-verify read-only: `bash ../scripts/demo-testnet-live.sh verify`. The
> earlier 2026-06-03 hashes are the superseded pre-fix run. Authoritative status →
> [`../RESUBMISSION_STATUS.md`](../RESUBMISSION_STATUS.md).

This is the **on-chain half** of the LP-0013 submission: the deployable
[spel-framework](https://github.com/logos-co/spel) program (a RISC0 zkVM guest)
that ports the token-authority semantics proven in the parent Rust workspace
into the LEZ account-adapter, plus the host-side driver that deploys it and
exercises a full lifecycle against a sequencer.

It lives in the repository (not only on a developer laptop) so an evaluator can
build, inspect, deploy, and re-derive the ProgramId/ImageID themselves.

## Relationship to the parent workspace

This is a **self-contained nested cargo workspace**, pinned to the LEZ testnet
revisions and requiring the risc0 toolchain (`cargo-risczero`) to build. It is
intentionally listed under `exclude` in the parent `../Cargo.toml`, so the
toolchain-light `cargo check/test --workspace` CI gate at the repo root does
**not** attempt to build the guest. Build and deploy it from inside this
directory.

The offline Rust crates in the parent workspace (`admin-authority-core`,
`mint-core`, `mint-program`, …) are the runtime-agnostic reference semantics;
this program is their on-chain realization. Both enforce the same authority
model (nonzero amount, current-authority authorization, revoked-authority
rejection, supply/balance overflow guards).

## Instruction surface

| Instruction | Accounts | Purpose |
|-------------|----------|---------|
| `create_mint` | `mint` (`init` PDA `lp0013:mint:v1`), `authority` (signer) | Create a variable-supply mint owned by `authority`. |
| `create_holding` | `holding` (`init` PDA `lp0013:holding:v1`), `payer` (signer) | Claim the recipient holding account **once**, balance 0. |
| `mint_to` | `mint` (`mut`), `recipient_holding` (`mut`), `authority` (signer) | Mint to an **existing** holding. Repeatable — balances accumulate. The authority guard runs **before** any write. |
| `set_mint_authority` | `mint` (`mut`), `authority` (signer) | Rotate or revoke (`None`) the mint authority. |

### Why minting is split into two instructions

The earlier deployed guest declared the recipient holding `#[account(init)]`
inside `mint_to`. `init` claims a fresh program-owned PDA and is **single-use**:

- a *second* `mint_to` to the same holding would fail on the re-claim — so
  variable supply was never actually demonstrated on chain (the lifecycle only
  ever landed one mint, and `supply` read `100` because of that, not because
  accumulation was shown); and
- the post-revoke `mint_to` was rejected by that `init` side effect **before**
  `require_authority` ran, so the authority guard itself was never genuinely
  exercised on chain.

`#[lez_program]` derives the account claim from the attribute (`init` →
non-idempotent claim) and the handler cannot override it, so the fix is
structural: `create_holding` claims the holding once (`init`), and `mint_to`
takes the holding as `#[account(mut)]` (must already exist) so it is repeatable
and the authority check is the first thing that can reject the transaction.
A corrected lifecycle therefore mints **twice** to one holding (balance
accumulates) and shows a post-revoke `mint_to` rejected by the guard
(`Program error 2008: authority has been revoked`).

## Prerequisites

- Rust + [risc0 toolchain](https://dev.risczero.com/api/zkvm/install) (`cargo-risczero`)
- [LSSA wallet CLI](https://github.com/logos-blockchain/lssa) (`wallet` binary)
- A reachable sequencer — the public testnet (`https://testnet.lez.logos.co/`)
  or a local `lgs localnet`

Pins (match the public testnet, `v0.2.0-rc3` = `cf3639d8`): see `Cargo.toml`
and `methods/guest/Cargo.toml` (`spel`/`spel-framework` rev `31e52c52`,
`nssa_core` tag `v0.2.0-rc3`).

## Quick start

```bash
# 1. Build the guest binary (risc0)
make build

# 2. Generate the IDL from the #[lez_program] source
make idl

# 3. Inspect the ProgramId / ImageID for the built binary
make inspect

# 4. Deploy to the configured sequencer
make deploy

# 5. Drive the full lifecycle (deploy + create_mint + create_holding +
#    two mints + set_mint_authority(None) + post-revoke mint_to)
cargo run --bin live_lifecycle
```

`live_lifecycle.rs` reads sequencer/account config from the environment and
passes strongly-typed instruction args to `nssa` directly (the generated IDL
does not emit the instruction-arg `Option<T>` inner type — see
`../docs/SPEL_STATUS.md`).

## Make targets

| Target | Description |
|--------|-------------|
| `make build` | Build the guest binary (risc0) |
| `make idl` | Generate IDL JSON from program source |
| `make inspect` | Show ProgramId / ImageID for the built binary |
| `make deploy` | Deploy the program to the sequencer |
| `make cli ARGS="..."` | Run the IDL-driven CLI |
| `make setup` | Create accounts via wallet |
| `make status` | Show saved state and binary info |
| `make clean` | Remove saved state |

## Layout

```
onchain-program/
├── admin_authority_spike_core/   # Shared types (guest + host)
│   └── src/lib.rs
├── methods/
│   ├── build.rs                  # Compiles the guest, emits *_ELF / *_ID
│   └── guest/
│       └── src/bin/admin_authority_spike.rs   # the #[lez_program] guest
├── examples/
│   └── src/bin/
│       ├── generate_idl.rs              # IDL generator (spel generate-idl)
│       ├── admin_authority_spike_cli.rs # IDL-driven CLI wrapper
│       └── live_lifecycle.rs            # deploy + full lifecycle driver
├── spel.toml                     # SPEL CLI config (IDL + binary paths)
├── Cargo.toml                    # nested workspace root (excluded from parent)
└── Makefile
```

The crate is historically named `admin_authority_spike` (its `spel init`
scaffold name); the directory is named `onchain-program` to signal its role as
the submission's deployable program.
