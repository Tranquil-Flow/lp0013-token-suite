# LP-0013 Requirements Matrix

This matrix maps each explicit LP-0013 requirement to a concrete artifact, command, or proof log. It is intentionally conservative: where a path requires a prepared LEZ/RISC0 host, it says so rather than implying that a generic hosted runner has the Logos localnet stack preinstalled.

## Summary

- Final narrated video: https://youtu.be/rUgsCCPiQfo
- Public implementation repo: https://github.com/Tranquil-Flow/lp0013-token-suite
- Corrected public-testnet ProgramId/ImageID: `32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce`
- Corrected public-testnet mint PDA: `HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu`
- Standalone local-sequencer e2e evidence: 2026-06-09 M4 Pro run, `RISC0_DEV_MODE=0`, documented in `docs/LEZ_PROOF_LOG.md`.

## Criteria

| LP-0013 criterion | Status | Artifact / command | Notes |
| --- | --- | --- | --- |
| Public repository containing token-program code changes | Complete | `onchain-program/`, `spel-spike/admin_authority_guest.rs`, `admin-authority-core/`, `mint-core/`, `mint-program/` | Corrected LEZ guest splits `create_holding` from mutable `mint_to`, so repeated mints accumulate and post-revoke rejection reaches the authority guard. |
| README and design docs for authority model/lifecycle | Complete | `README.md`, `SUBMISSION.md`, `docs/SPEC_COMPLIANCE.md`, `docs/SPEL_STATUS.md`, `docs/LEZ_PROOF_LOG.md`, this file | Docs describe create, mint, rotate/revoke, testnet evidence, and local-sequencer reproduction. |
| Tests and example programs/scripts | Complete | `cargo test --workspace`; `bash scripts/demo.sh`; `examples/variable-supply`, `examples/fixed-supply`, `examples/config-pda-gated`; `scripts/demo-localnet.sh`; `scripts/demo-testnet-live.sh verify` | Offline tests cover authority rotation/revocation, wrong authority, repeated mint, fixed supply, and unchanged failed-state paths. |
| Updated token program deployed and tested on LEZ devnet/testnet | Complete | `bash scripts/demo-testnet-live.sh verify`; `docs/LEZ_PROOF_LOG.md` | Public LEZ testnet corrected run: deploy `5b39deec…85cb4ce0`, two accumulating mints `60 + 40 = 100`, revoke authority, post-revoke mint not included, final PDA `authority=None, supply=100`. |
| End-to-end integration tests run against a LEZ sequencer standalone mode and are included in CI | Complete as CI path + real prepared-run evidence | `.github/workflows/ci.yml` jobs `local-sequencer-e2e-preflight` and `local-sequencer-e2e`; `bash scripts/demo-localnet.sh`; `docs/LEZ_PROOF_LOG.md` 2026-06-09 standalone run | Hosted CI runs the preflight/syntax check on every push. The actual standalone localnet e2e is a manual CI job for prepared self-hosted runners with `logos-lez` + `risc0` labels, because hosted runners lack LEZ localnet, RISC0 Docker guest builder readiness, and funded localnet wallet state. A real M4 Pro localnet run passed on 2026-06-09. |
| CI green on default branch | Complete | GitHub Actions run https://github.com/Tranquil-Flow/lp0013-token-suite/actions/runs/27200130940 at `87a79859c105a36566a75454e5d8f094cfaf7a10` | Hosted CI completed successfully: `Rust workspace` and `Standalone LEZ local-sequencer e2e preflight` passed; manual self-hosted localnet and public-testnet jobs are correctly skipped on push and available via `workflow_dispatch`. |
| README documents e2e usage: deployment, program addresses, minting, rotating/revoking via CLI/script | Complete | `README.md`; `SUBMISSION.md`; `docs/HOST_LOGOS_TOOLCHAIN.md`; `docs/LEZ_PROOF_LOG.md`; `scripts/demo-localnet.sh`; `scripts/demo-testnet-live.sh` | Public-testnet verifier is read-only; localnet script builds/deploys/runs lifecycle from a LEZ-equipped host. |
| Reproducible e2e demo script works against real local sequencer with `RISC0_DEV_MODE=0` | Complete on prepared LEZ host | `RISC0_DEV_MODE=0 NSSA_WALLET_HOME_DIR=<funded-localnet-wallet> bash scripts/demo-localnet.sh` | 2026-06-09 M4 Pro run built the guest with Docker-backed `cargo risczero`, deployed to local sequencer, confirmed lifecycle, final supply 100, holding balance 100, authority revoked. |
| Recorded narrated video demo included; terminal output includes proof generation / `RISC0_DEV_MODE=0` | Complete link present; contents manually reviewed by submitter | https://youtu.be/rUgsCCPiQfo | The link is reachable via YouTube oEmbed. The submitter recorded this as final LP-0013 video evidence. |
| CU cost documented | Complete, with limitation disclosed | `docs/BENCHMARKS.md`, `docs/LEZ_PROOF_LOG.md` | Public testnet does not expose per-tx CU telemetry, so docs report local-sequencer/executor measurements and avoid inventing public-testnet CU numbers. |

## Commands for evaluators

Deterministic hosted/local gates:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/demo.sh
python3 scripts/validate-submission-docs.py
python3 tests/test_validate_submission_docs.py
```

Standalone local-sequencer e2e on a LEZ-equipped host:

```bash
bash scripts/preflight-localnet-e2e.sh --report
export RISC0_DEV_MODE=0
export NSSA_SEQUENCER_URL=http://127.0.0.1:3040
export NSSA_WALLET_HOME_DIR=<funded-localnet-wallet-home>
bash scripts/demo-localnet.sh
```

Public-testnet read-only re-verification:

```bash
bash scripts/demo-testnet-live.sh verify
```
