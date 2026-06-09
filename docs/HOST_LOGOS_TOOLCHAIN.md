# Host Logos toolchain notes

This file records the toolchain used for LP-0013 proof generation and how evaluators can reproduce or inspect the same paths.

## Required tools for full local rebuild

The lightweight Rust workspace can be checked with standard Rust tooling. Full LEZ proof generation additionally needs the Logos toolchain:

```bash
which cargo rustc
which spel || true
which lgs || true
which logos-scaffold || true
which cargo-risczero || true
cargo --version
rustc --version
spel --version || true
lgs --version || true
cargo risczero --version || cargo-risczero --version || true
```

The final LP-0013 artifacts were built with `RISC0_DEV_MODE=0`, deployed to the public LEZ testnet, and documented in `docs/LEZ_PROOF_LOG.md`.

## Generated IDL

The authoritative SPEL-generated IDL files are committed under `idl/`:

```text
idl/admin-authority.idl.spel-generated.json
idl/admin-authority.idl.spel-generated.rc3-testnet.json
```

They describe the corrected four-instruction surface:

- `create_mint`
- `create_holding`
- `mint_to`
- `set_mint_authority`

The hand-written `idl/admin-authority.idl.json` is retained only as a design reference for pieces the generator does not emit, such as illustrative discriminators and declared errors.

## Public-testnet verification

The canonical verification path is read-only and needs no private keys, faucet, or wallet state:

```bash
bash scripts/demo-testnet-live.sh verify
```

Expected final state:

- ProgramId/ImageID: `32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce`
- Mint PDA: `HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu`
- Authority: `None`
- Supply: `100`
- Decimals: `6`

## Local sequencer reproduction

For a local sequencer run under `RISC0_DEV_MODE=0`:

```bash
bash scripts/preflight-localnet-e2e.sh --report
bash scripts/demo-localnet.sh --check
export RISC0_DEV_MODE=0
export NSSA_WALLET_HOME_DIR=<funded-localnet-wallet-home>
bash scripts/demo-localnet.sh
```

The public testnet does not expose per-transaction CU telemetry. `docs/BENCHMARKS.md` documents the available local-sequencer methodology and labels platform limits explicitly.


## CI local-sequencer e2e

`.github/workflows/ci.yml` includes two standalone-localnet entries:

- `local-sequencer-e2e-preflight` runs on hosted CI and reports whether the LEZ/RISC0/wallet prerequisites are present. It also shell-checks the localnet scripts.
- `local-sequencer-e2e` is a manual `workflow_dispatch` job for a prepared self-hosted runner labeled `self-hosted`, `logos-lez`, and `risc0`. It runs `bash scripts/preflight-localnet-e2e.sh --real-run` and then `bash scripts/demo-localnet.sh` with `RISC0_DEV_MODE=0`.

This split is deliberate: generic hosted runners do not ship a standalone LEZ sequencer, a funded localnet wallet home, or a ready Docker-backed RISC0 guest build environment. The 2026-06-09 M4 Pro run in `docs/LEZ_PROOF_LOG.md` is the retained real-run evidence.
