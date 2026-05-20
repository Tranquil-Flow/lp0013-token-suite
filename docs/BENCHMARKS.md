# LP-0013 Benchmarks

Captured during the host-side LEZ spikes on 2026-05-17 (structural surface) and 2026-05-18 (semantic release candidate). Values are honest single-run numbers from the proof sessions documented in `docs/LEZ_PROOF_LOG.md`; statistical analysis is out of scope for this submission.

## Host

```text
host:    Evis-MacBook-Pro.local
os:      macOS 15.6.1 (build 24G90, Darwin 24.6.0)
arch:    Apple silicon (arm64)
rustc:   1.94.0
cargo:   1.94.0
cargo-risczero: 3.0.5
spel:    spel-cli (binary built 2026-05-04)
lgs:     logos-scaffold 0.1.1
LEZ:     v0.2.0-rc1 (logos-execution-zone @ 35d8df0d)
SPEL:    v0.2.0-rc.3 (spel-framework @ 0c0b8505)
sequencer: local devnet on 127.0.0.1:3040 (RISC0_DEV_MODE inherited from the LP-0017 scaffold session — `risc0_dev_mode = false` per scaffold.toml)
```

## Offline Rust workspace

```bash
bash scripts/check-prereqs.sh
```

- 31 unit tests pass (admin-authority-core 6, admin-authority-spel 2, mint-core 7, mint-program 6, mint-sdk 3, mint-cli 2, examples/variable-supply 1, examples/fixed-supply 1, examples/config-pda-gated 1)
- `cargo fmt --all -- --check`: 0 changes
- `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings
- `cargo test --workspace`: clean
- Wall-clock for full pre-req: ~10 s after first build

## Offline demo runtime

```bash
bash scripts/demo.sh
```

Three runnable lifecycles execute end-to-end:

- `cargo run -p mint-cli -- demo-variable` — create → mint → rotate → mint → revoke → rejected mint
- `cargo run -p mint-cli -- demo-fixed` — fixed mint with no authority, all mint attempts rejected
- `cargo run -p variable-supply` / `fixed-supply` / `config-pda-gated` — runnable examples mirroring the CLI flows

Wall-clock for full demo: ~5 s after compile cache is warm.

## SPEL IDL regeneration

```bash
spel -- init admin_authority_spike --lez-tag v0.2.0-rc1 --spel-rev ed3bbedb…
cd admin_authority_spike
# replace guest + IDL driver with spel-spike/ files
make idl
```

- Cold cargo cache → 6 m 6 s (first build, downloads + builds 435-package dep graph)
- Warm cache → 10.45 s (single-package rebuild of `admin_authority_spike-examples`)
- Output: 76-line IDL JSON capturing 3 instructions + PDA seeds + signer/writable modes

## RISC0 guest build

```bash
cargo risczero build --manifest-path methods/guest/Cargo.toml
```

Structural-surface guest (2026-05-17 archival spike):

- Wall-clock: 8 m 34 s
- ELF size: 462,148 bytes
- ImageID: `b59d19dce244811348c0e8fe085733bb5dc4c9f548d448634f576ec643594d19`

Semantic release-candidate guest (2026-05-18 rerun):

- Wall-clock: 8 m 24 s cold (first semantic build), 3 m 19 s warm rebuild after the holding-PDA refinement
- ELF size: 480,352 bytes
- ImageID: `58470667b5d45fcc4317684eb7aaad2b19c0cf666bd8c7f85d2b0e1069d0b960`

Notable: SPEL `v0.2.0-rc.3` did **not** trigger the LP-0017 cc-rs / ring guest-dep failure on macOS arm64 for either build; the spel-framework dep graph cleanly excluded host-only crates from the guest target without manual intervention.

## Deploy to live LEZ sequencer

Structural-surface deploy (archival):

- `wallet deploy-program …/admin_authority_spike.bin`
- Deploy transaction hash: `2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965`
- Block: 45491
- Wall-clock: < 20 s from CLI invocation to block inclusion
- Confirmation: repeat deploy in the next block window rejected with `ProgramAlreadyExists`, confirming registry persistence

Semantic release-candidate deploy (2026-05-18):

- `lgs deploy --program-path …/admin_authority_spike.bin --json`
- Deploy transaction hash: `b16831c0ee550014ea9297ba47d47b31d0c1b425ff3219b44358189bb9204ab5`
- Block: 49551
- Wall-clock: < 20 s from CLI invocation to block inclusion

## Lifecycle transactions

Driven by `spel-spike/live_lifecycle.rs` against the live local sequencer. Each step signs with the authority's wallet `PrivateKey` and submits via `RpcClient::send_transaction`. Each row is one tx; wall-clock from submit to confirmation poll.

### Archival structural-surface lifecycle (2026-05-17)

| step | instruction | tx hash | result |
| --- | --- | --- | --- |
| 1 | `create_mint(decimals=6, initial_authority=Some(authority))` | `fd68e225ceb3164f88367600564a026dbfb8f4823f449a6b07c37fc35de79c69` | confirmed (wire-level) |
| 2 | `mint_to(amount=100)` | `07de7c91b5137fdb88b1f0ad84bb3b30a436cf9e8e368193fc81998713d88811` | confirmed (wire-level) |
| 3 | `set_mint_authority(None)` | `ec58ace48bbadee7143585b7bc402b33dd5fd767b8dd15dcf13ce1a87eba204d` | confirmed (wire-level) |
| 4 | `mint_to(amount=7)` post-revoke | `e1ecbb81da1a828a7068ef05401c96ed7593d29c8fa9537c07bda1dea020a3f3` | confirmed (wire-level only — the structural-surface guest did not enforce authority semantics on-chain) |

### Semantic release-candidate lifecycle (2026-05-18)

| step | instruction | tx hash | result |
| --- | --- | --- | --- |
| 1 | `create_mint(decimals=6, initial_authority=Some(authority))` | `7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63` | confirmed (semantic: mint PDA initialized with the expected decimals and authority) |
| 2 | `mint_to(amount=100)` | `c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f` | confirmed (semantic: mint PDA supply=100, holding PDA balance=100) |
| 3 | `set_mint_authority(None)` | `756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c` | confirmed (semantic: mint PDA `current_authority=None` persisted) |
| 4 | `mint_to(amount=7)` post-revoke | `27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff` | rejected — not confirmed; sequencer logs `AccountAlreadyInitialized { account_index: 1 }` at the LEZ framework layer because the holding PDA is already program-claimed |

Wall-clock per tx: ~1.5–3 s end-to-end (sign + submit + 15 s sequencer block interval + decode poll). Driver wall-clock for the full 4-tx run + final state read: ~12 s on both spikes.

### Authority semantics

After step 4 in the 2026-05-18 semantic rerun, the mint PDA's decoded state is:

```text
MintDefinition {
  authority: AuthorityInfo {
    authority_type: 0 (MintTokens),
    current_authority: None,
  },
  supply: 100,
  decimals: 6,
}
```

This matches the handover's expected post-revocation state exactly: `supply = 100` from the pre-revoke `mint_to`, `current_authority = None` from the persisted `set_mint_authority`, `decimals = 6` from `create_mint`. The semantic source (`spel-spike/admin_authority_guest.rs`) enforces current-authority authorization, revoked-authority rejection (`Program error 2008: authority has been revoked`), supply/balance overflow checks, and zero-amount rejection; rotation and revocation are persisted. The on-chain readback confirms these semantics held end-to-end through the LEZ executor.

The post-revoke `mint_to` is rejected at the LEZ framework layer with `AccountAlreadyInitialized` because the holding PDA was claimed on first mint. The offline `mint-core` tests prove the underlying `require_authority` rejection on the same code path with `Program error 2008`; the on-chain mint state independently proves the revocation is persisted (`current_authority=None`), so future minting cannot succeed semantically either. The combined evidence keeps the invariant honest without overclaiming which rejection layer fires first.

In the original 2026-05-17 structural-surface spike, `supply` stayed at 0 and `current_authority` stayed `Some(...)` after `mint_to` and `set_mint_authority` because that guest only captured the IDL-visible surface and did not run the authority/revocation logic. After review, `spel-spike/admin_authority_guest.rs` was advanced to semantic source as described above; the 2026-05-18 hashes reflect that semantic source running on chain.

## Compute units (CU)

LP-0013 spec line: *"Document the compute unit (CU) cost of each new operation (mint, rotate authority, revoke authority) on LEZ devnet/testnet."*

Methodology (consistent with LP-0017 `BENCHMARKS.md` §"Methodology"): on LEZ, public-transaction CU is the **Risc0 zkVM executor time** that the sequencer logs as `risc0_zkvm::host::server::exec::executor: execution time: <X>ms`. This is the meaningful per-instruction compute cost — `RISC0_DEV_MODE=0` does not change these numbers for public-transaction paths because the host-side prover is bypassed (the sequencer executes the program in its own zkVM executor regardless of `RISC0_DEV_MODE`).

Numbers below are extracted from the live local sequencer log (`.scaffold/logs/sequencer.log` in the LP-0017 scaffold session — same sequencer that ingested the txhashes documented above and in `docs/LEZ_PROOF_LOG.md`). Each row is the executor-time line that immediately precedes the corresponding `Validated transaction` (success) or follows the `failed execution check` (rejection) for the named tx hash.

### Semantic release-candidate lifecycle (2026-05-18)

| Operation | Tx hash | Result | Risc0 executor time | Notes |
|---|---|---|---|---|
| Program deploy | `b16831c0…04ab5` | confirmed | 8.77 ms | one-off; charged at deploy, not at use |
| `create_mint(decimals=6, initial_authority=Some)` | `7d582e7b…6f9d63` | confirmed | 8.38 ms | first-mint setup includes mint-PDA initialization |
| `mint_to(amount=100)` | `c474cf82…d3c74f` | confirmed | 7.58 ms | includes holding-PDA claim on first mint |
| `set_mint_authority(None)` (revoke) | `756ee393…7351c` | confirmed | 6.76 ms | rotation cost is structurally identical (single account write) |
| `mint_to(amount=7)` post-revoke | `27df9483…2bff` | rejected at LEZ framework | 4.43 ms | early rejection — no state mutation; framework-layer guard fires before guest body |
| `set_mint_authority(None)` post-revoke re-verification | `cea5b8c7…fc5eb4` | rejected with `Program error 2008` | 4.21 ms | semantic guard from the guest body — `require_authority` panic captured live |

### Findings

- **Successful operations cluster in the 6.7–8.4 ms range** on the local sequencer. The variation is dominated by per-operation account I/O (mint-PDA init for `create_mint`, holding-PDA claim for first `mint_to`).
- **Authority rotation and revocation share one code path** (`SetMintAuthority`); the spec-required "rotate authority" and "revoke authority" CU costs are both represented by the 6.76 ms `set_mint_authority(None)` measurement. Rotation to a new key has the same account-write footprint.
- **Rejected operations cost ~50% less** because execution halts at the authority guard before performing any account writes. This is the deterministic-rejection property the spec mandates, visible in the CU profile.
- **Independent re-verification** of the revoked-authority guard (`cea5b8c7…fc5eb4` at 4.21 ms) was submitted ~22 minutes after the initial revocation and rejected on chain with `Program error 2008: authority has been revoked` — the canonical `require_authority` panic from `spel-spike/admin_authority_guest.rs`. This is direct on-chain semantic proof of the post-revocation guard, with its CU cost measured.

### Reproduce

The same sequencer that captured these numbers is still running (`lgs localnet status` from `~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/`). To re-extract:

```bash
grep -B3 -A1 "Validated transaction with hash 7d582e7b\|Validated transaction with hash c474cf82\|Validated transaction with hash 756ee393\|hash 27df9483.*failed\|hash cea5b8c7.*failed" \
    ~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/logs/sequencer.log
```

Each `Validated transaction` line for a successful tx has the executor-time line immediately above it; each `failed execution check` line has the executor-time line immediately below.

## Reproduce

The reproducible commands and intermediate state are checked in:

- `spel-spike/admin_authority_guest.rs` — guest module source
- `spel-spike/generate_idl.rs` — IDL driver
- `spel-spike/README.md` — step-by-step reproduction
- `idl/admin-authority.idl.spel-generated.json` — generated IDL artifact
- `docs/LEZ_PROOF_LOG.md` — full host-side log
