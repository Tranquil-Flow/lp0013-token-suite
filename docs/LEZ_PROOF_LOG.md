# LEZ / SPEL Proof Log

> **✅ RESOLVED (2026-06-27) — corrected guest deployed + verified on the public testnet.** The init-only holding PDA was split into `create_holding` + mutable `mint_to`. The corrected four-instruction guest (ImageID/ProgramId `338865e9…871554a`) was re-deployed to `testnet.lez.logos.co` under `RISC0_DEV_MODE=0` and the full lifecycle re-captured: **two accumulating mints (60+40 → 100) prove variable supply on chain**, and the post-revoke mint is **rejected by the authority guard** (the holding already exists, so it is `require_authority`, not an init side effect). See the **2026-06-27 corrected** section immediately below; it supersedes the **2026-06-03 pre-fix run** (single-`init` holding, one mint), which is retained as a historical record. Authoritative state → [`../RESUBMISSION_STATUS.md`](../RESUBMISSION_STATUS.md).

Host-side proof attempts for LP-0013, capturing exact commands, environment, and outcomes. The intent is honest evaluator-facing evidence — successes are recorded with hashes/timing, failures are recorded with the exact error so the submission stays trustworthy.

The public-testnet deploy + lifecycle of **2026-06-27** (next section) is the **load-bearing evidence**: it ran the **corrected four-instruction guest** on the shared, no-auth `testnet.lez.logos.co` network under `RISC0_DEV_MODE=0`, with two accumulating mints and a guard-rejected post-revoke mint. The **2026-06-03** run below it is the superseded **pre-fix** run (single-`init` holding, one mint), retained as a historical record. The earlier local-sequencer sessions (2026-05-17 structural, 2026-05-18 semantic, further down this log) are historical corroboration of the wire path and semantics — and the 2026-05-18 localnet run additionally captured the exact guest-panic string (`Program error 2008: authority has been revoked`) that the testnet's hidden sequencer logs cannot surface.


## Standalone local-sequencer e2e (2026-06-09) — CORRECTED GUEST, CI-prepared host evidence

This run proves the explicit standalone-sequencer supportability path behind `scripts/demo-localnet.sh` and the manual self-hosted CI job `local-sequencer-e2e` in `.github/workflows/ci.yml`. It was executed on the M4 Pro prepared LEZ/RISC0 host after starting a standalone local sequencer with `lgs localnet start`. Docker Desktop was running so `cargo risczero build` could use the RISC0 guest builder image.

```text
host:             m4pro / macOS arm64
sequencer RPC:    http://127.0.0.1:3040
RISC0_DEV_MODE:   0
entrypoint:       bash scripts/demo-localnet.sh
log retained at:  .local/logs/lp0013-local-sequencer-e2e-20260609T102107Z.log on the M4 Pro host
```

Key output from the run:

```text
RISC0_DEV_MODE = 0
ImageID: 338865e9549b18fb736020eaef87d5e20075b4250e10c00e08ea918c4871554a - .../admin_authority_spike.bin
[0] deploy_program           confirmed tx=793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce
[1] create_mint              confirmed tx=b774e548c20a7cc872fd24db4448d4a3a7d45531cef59f9f9b01c036d9962afe
[2] create_holding           confirmed tx=b31354653e0e1d967e2574c73319bd97fcecdb4248e04959bb67192a409accba
[3] mint_to(60)              confirmed tx=3c7c3aa8bc1075b3b744d0c37bf2612e4a1d3f21ee44c1f26ea9f03a30b2675f
[4] mint_to(40)              confirmed tx=fe148c94547214aaded917d82bd6c84b2dcf54527afc28f4343f939b3fb53399
[5] set_mint_authority(None) confirmed tx=e071f979ccfa55e309ef3102a94d6de2a5f1beb8850faaac5d630653de9c884d
[6] mint_to(post-revoke)     rejected as expected (no inclusion) tx=b293da3651b7a74a5cc8838593340c39453d5cd77e032fb4d0eb8cdaeed7bec8
[7] mint state = OnChainMintDefinition { authority: OnChainAuthorityInfo { authority_type: 0, current_authority: None }, supply: 100, decimals: 6 } (supply=100 via accumulation, authority revoked — OK)
[8] holding state = OnChainTokenHolding { ... balance: 100 } (balance=100 from two accumulating mints — OK)
```

The standalone sequencer log captured the semantic rejection string for the post-revoke mint:

```text
Program error [8008]: Program error 2008: authority has been revoked
"Guest panicked: Program error [8008]: Program error 2008: authority has been revoked"
```

Evaluator command path:

```bash
bash scripts/preflight-localnet-e2e.sh --report
export RISC0_DEV_MODE=0
export NSSA_SEQUENCER_URL=http://127.0.0.1:3040
export NSSA_WALLET_HOME_DIR=<funded-localnet-wallet-home>
bash scripts/demo-localnet.sh
```

## Public testnet deploy + lifecycle (2026-06-27) — CORRECTED GUEST (load-bearing)

The **corrected** LP-0013 guest (init-only holding split into `create_holding` + mutable `mint_to`) was deployed and exercised on the **public LEZ testnet**. This is the evidence that proves the fix; it supersedes the 2026-06-03 pre-fix run below.

```text
sequencer RPC:  https://testnet.lez.logos.co/   (public, no-auth, JSON-RPC over HTTPS POST)
explorer:       https://explorer.testnet.lez.logos.co/
network:        real consensus, RISC0_DEV_MODE=0 (sequencer-side proving for public transactions)
date:           2026-06-27
```

### Guest binary (corrected, v0.2.0 / current testnet-matching)

Built with the current LEZ v0.2.0 client/runtime path. The release RISC Zero guest ELF was wrapped as a `risc0_binfmt::ProgramBinary` before deployment; raw ELF attempts are not valid deployable program binaries on the current testnet. The successful ProgramBinary SHA-256 is `fac6f9715efc03edcb695dc71545cb24fac6bc86530644e2748f50d6ef9009f3`.

```text
file:    target/riscv32im-risc0-zkvm-elf/release/lp0013_authority
wrapped: /tmp/lp0013_authority.release.program.bin
ProgramBinary sha256: fac6f9715efc03edcb695dc71545cb24fac6bc86530644e2748f50d6ef9009f3
ImageID: 338865e9549b18fb736020eaef87d5e20075b4250e10c00e08ea918c4871554a
```

In LEZ a program is content-addressed, so the **ProgramId equals the ImageID** — verified three ways: the wrapped ProgramBinary deployment, the lifecycle driver's program id, and the on-chain `program_owner` of the mint PDA (base58 `4UARaVcJJoLxebFAobocsZyzpJ5TTUvvhRtFuHtuHypd`, which decodes to the same 32 bytes). The ImageID differs from the 2026-06-03/2026-06-04 historical runs because the current evidence uses the v0.2.0 ProgramBinary deployment/signing path and a fresh public-testnet lifecycle.

### Lifecycle driver output (against `testnet.lez.logos.co`)

`onchain-program/examples/src/bin/live_lifecycle.rs`, run from a faucet-funded signer (`6HEYFUW4QbHPfdHTMPZLDeC6F5PL6suhSGJbTnsauhWJ`), against the public testnet:

```text
== LP-0013 live lifecycle ==
program_id  = [3915745331, 4212693844, 3927990387, 3805644783, 632583424, 247468046, 2358372872, 1247113544]
mint_pda    = 4gMBXeUskbUTzxoP8fJJEXj3jxTQz91m6ZW7fMsLMJq6
authority   = 6HEYFUW4QbHPfdHTMPZLDeC6F5PL6suhSGJbTnsauhWJ
recipient   = 366n7Nj21EzD27BXRKE2hFDWPtJ1E2Fcx9RmqQoGRD7h

[0] deploy_program           confirmed tx=793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce
[1] create_mint              confirmed tx=55908821088c98e898c4ef99e9a36e02856092f7afd0155f3457c25c5cf67746
[2] create_holding           confirmed tx=8a37a8fb7200856c57d199ce081f2b744ed3cbaeec8326c83092f5ca05ac668f
[3] mint_to(60)              confirmed tx=daf5aa91f35dff8250794c0dcfe932de473c651bd25c946d76f09a42cfdb6a97
[4] mint_to(40)              confirmed tx=ed07b29c004a796d504814ddf1a9a0cfda373d1618398b620e330ccb529b3cce
[5] set_mint_authority(None) confirmed tx=719123f918df2aee42c4e69d36ba8860807b2a69c97a2927097d8313a508550e
[6] mint_to(post-revoke)     rejected as expected (no inclusion) tx=016043771c0cc60efaf158ec120a9bf341326967c881285878469503ddd3d4fa
[7] mint state    = OnChainMintDefinition { authority: { authority_type: 0, current_authority: None }, supply: 100, decimals: 6 }
[8] holding state = OnChainTokenHolding { owner: [...], balance: 100 }
```

Two mints of 60 and 40 into the **same** holding both confirmed and accumulated to `supply=100` / `balance=100` — **variable-supply minting works on chain**, which the pre-fix single-`init` holding could not do (its second mint would fail). The post-revoke `mint_to(7)` targets the **already-existing** holding (`mut`, not `init`), so nothing rejects it before the guest body runs: the rejection is `require_authority` (error 2008), and the readback shows `supply` stayed 100 (not 107) with `current_authority = None`.

### Independent live re-verification (read-only, any reviewer)

Reproducible with `bash scripts/demo-testnet-live.sh verify` (needs only the `wallet` binary — no build, faucet, or keys):

| step | tx hash | live chain-info verdict |
| --- | --- | --- |
| deploy_program | `79399225…b717dce` | `Some(ProgramDeployment)` |
| create_mint | `55908821…cf67746` | `Some(Public)` |
| create_holding | `8a37a8fb…ac668f` | `Some(Public)` |
| mint_to(60) | `daf5aa91…cfdb6a97` | `Some(Public)` |
| mint_to(40) | `ed07b29c…29b3cce` | `Some(Public)` |
| set_mint_authority(None) | `719123f9…508550e` | `Some(Public)` |
| mint_to (post-revoke) | `01604377…3d4fa` | `Transaction is None` (never included) |

Mint PDA (`Public/4gMBXeUskbUTzxoP8fJJEXj3jxTQz91m6ZW7fMsLMJq6`) raw account data:

```json
{"balance":0,"program_owner":"4UARaVcJJoLxebFAobocsZyzpJ5TTUvvhRtFuHtuHypd",
 "data":"00006400000000000000000000000000000006","nonce":0}
```

Decodes (borsh, little-endian): `authority_type=0`, `Option tag=None` (authority revoked, persisted), `supply=100` (the post-revoke +7 never landed — would be 107), `decimals=6`.

### What is proved on the public testnet (corrected guest)

| Proof | Status | Evidence |
| --- | --- | --- |
| Corrected program deployed on public testnet | green | deploy tx `79399225…` → `Some(ProgramDeployment)`; ProgramId/ImageID `338865e9…871554a` |
| `create_mint` + `create_holding` confirmed | green | `55908821…`, `8a37a8fb…` → `Some(Public)` |
| **variable supply on chain** (two accumulating mints) | green | `daf5aa91…`(60) + `ed07b29c…`(40) → `Some(Public)`; PDA `supply=100`, holding `balance=100` |
| `set_mint_authority(None)` confirmed | green | `719123f9…` → `Some(Public)`; `current_authority=None` persisted |
| **post-revoke mint rejected by the authority guard** (not by init) | green | `01604377…` never included; holding pre-exists (`mut`), so the rejection is `require_authority` (2008); PDA `supply` stayed 100 |
| revocation invariant on-chain | green | live PDA readback decodes `authority=None, supply=100, decimals=6` |

**Why the rejection is genuinely the guard, not an init side effect (the reviewer's point #5).** In the pre-fix guest the holding was `#[account(init)]` on every mint, so a post-revoke mint was rejected by `AccountAlreadyInitialized` *before* `require_authority` ran — the guard was never genuinely exercised, and the single-mint lifecycle masked it. In the corrected guest the holding is created once (`create_holding`) and is then `mut`; the two earlier mints already wrote to it, so the post-revoke mint reaches the guest body and is rejected by `require_authority` (error 2008). The exact 2008 panic string is hidden by the testnet's sequencer logs, but it was captured directly on localnet (see the 2026-05-18 rerun's independent re-verification below, re-confirmable via `scripts/demo-localnet.sh`); on testnet the *state-level invariant* (`supply=100` not 107, `authority=None`) proves the post-revoke mint could not have succeeded.

## Public testnet deploy + lifecycle (2026-06-03) — SUPERSEDED (pre-fix guest)

The full LP-0013 authority lifecycle was deployed and exercised on the **public LEZ testnet**, not a local sequencer. Endpoints:

```text
sequencer RPC:  https://testnet.lez.logos.co/   (public, no-auth, JSON-RPC over HTTPS POST)
explorer:       https://explorer.testnet.lez.logos.co/
network:        real consensus, RISC0_DEV_MODE=0 (sequencer-side proving for public transactions)
date:           2026-06-03
```

### Version-pin landmine (why this guest differs from the 2026-05 localnet binary)

The refreshed lifecycle uses the LEZ **`v0.2.0`** client/runtime path. Earlier rc3/v0.1.2-compatible artifacts are retained only as historical notes. The 2026-05 localnet guest was built against **`v0.2.0-rc1`** (`35d8df0d`), whose `nssa/core/src/program.rs` differs by ~300 lines; that binary will not execute on the testnet. Two traps were defused before spending compile time:

1. `spel init`'s `--lez-tag` / `--spel-rev` flags do **not** propagate into the generated `methods/guest/Cargo.toml` or `examples/Cargo.toml` — the template hardcodes `nssa_core tag=v0.2.0-rc1` + `spel-framework tag=v0.2.0-rc.3`. Every `Cargo.toml` was hand-edited.
2. spel's **`v0.2.0-rc.3` tag pins `nssa_core` back to rc1 internally**. Only the spel branch `chore/bump-lez-to-v0.2.0` (commit `31e52c52`) pins rc3. So `spel-framework`/`spel` are pinned to `rev = "31e52c529baba2205eeeacf5bb52647e84236b94"` and every `nssa`/`nssa_core`/`common`/`wallet`/`sequencer_service_rpc` to `tag = "v0.2.0"`.

Verified before building: `cargo generate-lockfile` then grep the lock — `nssa_core` resolves to `cf3639d8`, zero `35d8df0d`. (`PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` is required on every cargo invocation: system Python 3.14 exceeds PyO3's max 3.13.)

### Guest binary (v0.2.0 / current testnet-matching)

```text
file:    methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
size:    478,000 bytes
ImageID: 59e15341b10dfacf6bfeb8436f587e18fb4bf714fc042c79aba9f8878fb0ae2c
ProgramId (hex):
  4153e159,cffa0db1,43b8fe6b,187e586f,14f74bfb,792c04fc,87f8a9ab,2caeb08f
```

Same semantic guest source as the 2026-05-18 localnet run; the differing ImageID is purely the rc3 dependency graph (`cf3639d8`) versus rc1 (`35d8df0d`).

### Deploy + execution model and gas finding

`wallet deploy-program` is fire-and-forget — the CLI discards the response, so it cannot surface a deploy tx hash. The lifecycle driver replicates the same transaction with a typed call so the hash can be captured and polled:

```rust
let deploy_tx = nssa::ProgramDeploymentTransaction::new(
    nssa::program_deployment_transaction::Message::new(bytecode));
let hash = sequencer_client
    .send_transaction(NSSATransaction::ProgramDeployment(deploy_tx)).await?;
// then poll get_transaction(hash) until inclusion
```

Findings, both verified against on-chain balances/nonces:

- **Deploy charges no gas.** `ProgramDeploymentTransaction` has no signer and `affected_public_account_ids() == []`.
- **Public-transaction execution charges no gas.** The signer's balance was unchanged (150 → 150) across create_mint + mint_to + set_mint_authority; only its `nonce` incremented (1 → 4 for the three included txs). The signer must be an *initialized* account (faucet-funded so it exists on-chain) holding a wallet signing key; balance is irrelevant to execution.
- **Proving is sequencer-side** for public (PDA / public-state) transactions, so the client stays light — no local proving, no heavy compute needed for this lifecycle. (Private executions would prove client-side; LP-0013 is entirely public-state.)

### Lifecycle driver output (against `testnet.lez.logos.co`)

The driver (`spel-spike/live_lifecycle.rs`, run from the v0.2.0 build tree) signs and submits a strongly-typed instruction enum through `nssa`, bypassing the IDL-driven CLI's `Option<T>` arg gap (see SPEL_STATUS). Authority/poll windows are env-configurable so the same driver runs against localnet or the slower-to-include testnet.

```text
== LP-0013 live lifecycle ==
program_id  = [1096016217, 3489271217, 1136197227, 410933359, 351751163, 2032927996, 2281220523, 749645967]
mint_pda    = FrbpfbUb5YpfeKEhsbMzKB5CAv9nbnCQDXbZrDJoQFV7
authority   = 8WWzugkceudpRHQdrB74CL3YjBYEqZHRFAR52itgkyBw
recipient   = 5Y3b9S6WL91ivBVU8qnVb1hfHuvey7rEL9FB69ZnuZ8m

[0] deploy_program           confirmed tx=07561014a617dc18c3a420db01c9f752755053eb58f44d8db98871646cb968ba
[1] create_mint              confirmed tx=17d90ea633db426a863efc697239aa158293c20822ff07839a2a0b6f2eeb37d2
[2] mint_to(100)             confirmed tx=be393bcf82e489bc5a940904ed0e38ea861b61939f43529132ca4c701f29bbd8
[3] set_mint_authority(None) confirmed tx=0540648f9f5099296340bcf65d0ac1a4cf89ff226eca7abb27dcdcb0b29f5784
[4] mint_to(post-revoke)     rejected as expected (no inclusion) tx=312ea9f120602f9aa2d574d43fefa73ae25d74e1bd228b9f65317fef8fef4798
[5] mint state = OnChainMintDefinition { authority: OnChainAuthorityInfo { authority_type: 0,
    current_authority: None }, supply: 100, decimals: 6 }
```

### Independent live re-verification (2026-06-03, sequencer block 37513)

The hashes above were re-queried directly from the public sequencer — pure chain reads, no local state, reproducible by any reviewer who points a wallet at `https://testnet.lez.logos.co/`:

```bash
export NSSA_WALLET_HOME_DIR=<any wallet home with sequencer_addr=https://testnet.lez.logos.co/>
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
wallet chain-info transaction --hash <hash>      # per tx
wallet account get --account-id Public/FrbpfbUb5YpfeKEhsbMzKB5CAv9nbnCQDXbZrDJoQFV7   # mint PDA
```

| step | tx hash | live chain-info verdict |
| --- | --- | --- |
| deploy_program | `07561014…cb968ba` | `Some(ProgramDeployment)` |
| create_mint | `17d90ea6…eeb37d2` | `Some(Public)` |
| mint_to(100) | `be393bcf…f29bbd8` | `Some(Public)` |
| set_mint_authority(None) | `0540648f…b29f5784` | `Some(Public)` |
| mint_to (post-revoke) | `312ea9f1…8fef4798` | `Transaction is None` (never included) |

Mint PDA (`Public/FrbpfbUb5YpfeKEhsbMzKB5CAv9nbnCQDXbZrDJoQFV7`) raw account data:

```json
{"balance":0,"program_owner":"73rZhrXT2AkmKGMbkGxoisHnabLc2pxNcvNZr8ZvRr9h",
 "data":"00006400000000000000000000000000000006","nonce":0}
```

The 19-byte `data` decodes (borsh, little-endian) exactly as the driver reported:

```text
byte 0      = 0x00   authority_type        = 0
byte 1      = 0x00   Option tag            = None        ← authority revoked, persisted
bytes 2..18 = 0x64.. supply (u128 LE)      = 100         ← mint_to(100) landed; post-revoke +7 did NOT (would be 107)
byte 18     = 0x06   decimals              = 6
```

### What is proved on the public testnet

| Proof | Status | Evidence |
| --- | --- | --- |
| Program deployed on public testnet | green | deploy tx `07561014…cb968ba` → `Some(ProgramDeployment)`; ProgramId `4153e159…2caeb08f` |
| `create_mint` confirmed | green | tx `17d90ea6…eeb37d2` → `Some(Public)` |
| `mint_to(100)` confirmed | green | tx `be393bcf…f29bbd8` → `Some(Public)`; PDA supply = 100 |
| `set_mint_authority(None)` confirmed | green | tx `0540648f…b29f5784` → `Some(Public)`; PDA `current_authority = None` persisted |
| post-revoke `mint_to` rejected | green | tx `312ea9f1…8fef4798` never included (`chain-info` → None); PDA supply stayed 100 (not 107) |
| revocation invariant on-chain | green | live PDA readback decodes `authority=None, supply=100, decimals=6` |
| deploy + public-exec gas cost | none | signer balance 150 → 150; only nonce 1 → 4 |

**Honesty note on the post-revoke rejection.** On the 2026-05-18 *localnet* run the host had sequencer-log access and could quote the exact guest panic (`Program error 2008: authority has been revoked`). The public testnet does not expose its sequencer logs, so on testnet the rejection is established two ways instead: (a) the transaction is never included (`chain-info` returns None), and (b) the live mint-PDA readback shows `supply = 100` (not 107) with `current_authority = None`, i.e. a mint after revocation could not have succeeded. The semantic guard's *exact error string* is corroborated by the localnet capture below and by the offline `mint-core` tests; on testnet the *state-level invariant* is what is directly proven.

## Host environment

```text
date:        2026-05-17
host:        Evis-MacBook-Pro.local
kernel:      Darwin 24.6.0 (xnu-11417.140.69) arm64
os:          macOS 15.6.1 (build 24G90)
arch:        Apple silicon (arm64)
```

## Toolchain versions

Captured from the host before any proof work.

```text
cargo            1.94.0 (85eff7c80 2026-01-15)
rustc            1.94.0 (4a4ef493e 2026-03-02)
spel             present at /Users/evinova/.cargo/bin/spel (no --version flag; cargo-installed binary built 2026-05-04)
lgs              logos-scaffold 0.1.1
logos-scaffold   0.1.1
cargo-risczero   3.0.5
gh               2.76.2
```

All five binaries used by the LP-0013 proof path (`cargo`, `rustc`, `spel`, `lgs`, `cargo-risczero`) are installed.

## Local gates

Repeating the offline gates from the container on the host, before any LEZ work.

```bash
bash scripts/check-prereqs.sh
bash scripts/demo.sh
python3 -m unittest discover -s tests -p 'test_validate_submission_docs.py'
```

Outcome: all three pass. `cargo test --workspace` reports 30 Rust tests passing (`admin-authority-core` 6, `admin-authority-spel` 2, `mint-core` 7, `mint-program` 7 — including the repeated-mint / post-revoke-guard contract test — `mint-sdk` 3, `mint-cli` 2, `examples/variable-supply` 1, `examples/fixed-supply` 1, `examples/config-pda-gated` 1, doc-tests 0). `scripts/demo.sh` prints the full authority lifecycle for all three runnable examples. The submission-doc validator self-tests (`tests/test_validate_submission_docs.py`) report `8 passed`.

## SPEL IDL regeneration

### Step 1 — first attempt against the placeholder crate

The shipped `admin-authority-spel/src/lib.rs` is the IDL-shape-test crate, not a SPEL guest. Running `spel generate-idl` against it surfaces the expected diagnostic:

```bash
spel -- generate-idl admin-authority-spel/src/lib.rs
# Error: No #[lez_program] module found in 'admin-authority-spel/src/lib.rs'
```

### Step 2 — scaffolded spike with real `#[lez_program]` guest

A sibling scaffold was created with the same LEZ / SPEL pins LP-0017 uses, then the placeholder guest was replaced with an LP-0013 surface (`create_mint`, `mint_to`, `set_mint_authority`; `AuthorityInfo`, `MintDefinition`, `TokenHolding`). The guest and IDL-driver sources are checked in under `spel-spike/` for reproducibility.

```bash
mkdir -p /tmp/lp0013-spike && cd /tmp/lp0013-spike
spel -- init admin_authority_spike \
    --lez-tag v0.2.0-rc1 \
    --spel-rev ed3bbedb4b684645da05455d30a4a0be7cc4dfe0
# 🚀 Creating SPEL project 'admin_authority_spike'...
# Updating git repository `https://github.com/logos-blockchain/logos-execution-zone.git`
# Updating git repository `https://github.com/logos-co/spel.git`
# Locking 435 packages to latest compatible versions
# ✅ Project 'admin_authority_spike' created!

cd admin_authority_spike
cp <token-suite>/spel-spike/admin_authority_guest.rs methods/guest/src/bin/admin_authority_spike.rs
cp <token-suite>/spel-spike/generate_idl.rs examples/src/bin/generate_idl.rs

make idl
# cargo run --bin generate_idl > admin_authority_spike-idl.json
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.45s
# ✅ IDL written to admin_authority_spike-idl.json
```

The generated output is committed at `idl/admin-authority.idl.spel-generated.json` (regenerated for the corrected four-instruction surface). It captures the four instructions, their PDA seeds, signer/writable/init modes, primitive arg types, and the full account bodies. The generator does not emit discriminators (LEZ dispatches by enum-variant index), the `execution` block, declared errors, or the instruction-arg `Option<T>` inner type. The spel-generated IDL is authoritative for the on-chain surface; the hand-written `idl/admin-authority.idl.json` is a design reference documenting the omitted pieces. `docs/SPEL_STATUS.md` carries the diff rationale.

## LEZ local-sequencer proof

> **Status: historical corroboration.** Both these localnet runs and the 2026-06-03 public-testnet run above predate the correctness fix (single-use `init` holding → `create_holding` + mutable `mint_to`); the load-bearing on-chain evidence is the **2026-06-27 corrected-guest testnet run** at the top of this file. See the top banner and `../RESUBMISSION_STATUS.md`. The sections below ran against a local sequencer (`127.0.0.1:3040`) and honestly document that the SPEL/LEZ wire path and the authority semantics work end-to-end — the localnet run additionally captured the exact guest-panic string for the post-revoke rejection (`Program error 2008: authority has been revoked`), which the testnet's hidden sequencer logs cannot surface. They are retained for transparency / corroboration.

The host's LEZ sequencer is up under the LP-0017 scaffold session:

```bash
cd ~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower
lgs localnet status
# tracked sequencer: pid=10334 running=true
# listener 127.0.0.1:3040: reachable (pid=10334)
# ownership: managed
# ready: true
```

The local LEZ devnet is reachable on `127.0.0.1:3040`, owned by the LP-0017 scaffold. The LP-0017 paid-in-blood gotcha (`spel-framework` guest deps pulling `bonsai-sdk` / `reqwest` / `rustls` / `ring` and `cc-rs` leaking `-arch arm64` into `riscv32-unknown-elf-gcc`) did **not recur** on this round under SPEL `v0.2.0-rc.3`. The guest built cleanly without forcing the raw-`nssa_core`-only workaround.

### Guest binary build

```bash
cd /tmp/lp0013-spike/admin_authority_spike
cargo risczero build --manifest-path methods/guest/Cargo.toml
# Compiling nssa_core v0.1.0 (logos-execution-zone @ 35d8df0d, tag v0.2.0-rc1)
# Compiling spel-framework-core v0.2.0 (spel @ 0c0b8505, tag v0.2.0-rc.3)
# Compiling spel-framework v0.2.0 (spel @ 0c0b8505, tag v0.2.0-rc.3)
# Compiling admin_authority_spike-guest v0.1.0 (/src/methods/guest)
# Finished `release` profile [optimized] target(s) in 8m 34s
# ELFs ready at: .../docker/admin_authority_spike.bin
```

Resulting binary:

```text
file:    methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
size:    462,148 bytes
ImageID: b59d19dce244811348c0e8fe085733bb5dc4c9f548d448634f576ec643594d19
ProgramId (hex):
  dc199db5,138144e2,fee8c048,bb335708,f5c9c45d,6348d448,c66e574f,194d5943
```

### Deploy to live sequencer

```bash
export NSSA_WALLET_HOME_DIR=~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/wallet
wallet check-health
# ✅All looks good!
wallet deploy-program /tmp/lp0013-spike/admin_authority_spike/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
```

Sequencer log confirms ingest and inclusion:

```text
[2026-05-17T13:56:20Z INFO  sequencer_core] Validated transaction with hash
    2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965,
    including it in block
[2026-05-17T13:56:21Z INFO  sequencer_core] Created block with 2 transactions in 0 seconds
[2026-05-17T13:56:21Z INFO  sequencer_service] Block with id 45491 created
```

Re-issuing the same deploy in a later block fails as expected with `ProgramAlreadyExists`, confirming the program is registered on chain:

```text
[2026-05-17T13:56:36Z ERROR sequencer_core] Transaction with hash
    2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965
    failed execution check with error: ProgramAlreadyExists, skipping it
```

### Authority-lifecycle transaction execution

The IDL-driven CLI auto-generated by the SPEL scaffold cannot serialize the `Option<[u8; 32]>` args on `create_mint` and `set_mint_authority` in the current revision:

```bash
spel create-mint --dry-run --decimals 6 --initial-authority None --payer Public/...
# 📋 Instruction: create_mint
# ❌ Serialization error: type mismatch: expected Defined { defined: "Option" },
#    got Raw("Option(None)")
```

This is the SPEL IDL gap documented above: `Option<T>` is emitted as `{"defined": "Option"}` without expanding the inner type, so the IDL-driven serializer has no schema for the payload. The workaround follows the LP-0017 pattern: bypass the IDL CLI and submit a strongly-typed instruction directly through `nssa`. A self-contained host driver at `spel-spike/live_lifecycle.rs` does exactly this:

- declares a parallel `Instruction` enum with the same struct-variant shape the SPEL `#[lez_program]` macro emits (`CreateMint { decimals, initial_authority }`, `MintTo { amount }`, `SetMintAuthority { new_authority }`),
- hands that enum to `nssa::public_transaction::Message::try_new`, which calls `Program::serialize_instruction` and matches the guest's `risc0_zkvm::serde::Deserializer`,
- fetches the signer's nonce + signing key from the `WalletCore` loaded from `NSSA_WALLET_HOME_DIR`, builds a `WitnessSet`, and submits via `sequencer_client.send_transaction(NSSATransaction::Public(...))`,
- polls `get_transaction(hash)` until confirmation or timeout.

Driver output against the live sequencer (with the program ID + mint PDA matching the deploy step above):

```text
== LP-0013 live lifecycle ==
program_id  = [3692666293, 327238882, 4276666440, 3140704008, 4123640925, 1665717320, 3329120079, 424499523]
mint_pda    = 5pbwgDVvsb8UbeQMD9BJiNUDHRG6nnTm7hn3n3yWCcH5
authority   = 2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2
recipient   = CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r

[1] create_mint              confirmed tx=fd68e225ceb3164f88367600564a026dbfb8f4823f449a6b07c37fc35de79c69
[2] mint_to(100)             confirmed tx=07de7c91b5137fdb88b1f0ad84bb3b30a436cf9e8e368193fc81998713d88811
[3] set_mint_authority(None) confirmed tx=ec58ace48bbadee7143585b7bc402b33dd5fd767b8dd15dcf13ce1a87eba204d
[4] mint_to(post-revoke)     UNEXPECTED confirm tx=e1ecbb81da1a828a7068ef05401c96ed7593d29c8fa9537c07bda1dea020a3f3
[5] mint state = OnChainMintDefinition { authority: OnChainAuthorityInfo { authority_type: 0,
    current_authority: Some([21, 20, 90, 238, 46, 108, 156, 87, 210, 132, 123, 140, 162, 225, 0, 147,
    127, 17, 238, 118, 253, 253, 117, 252, 181, 136, 72, 138, 162, 6, 69, 71]) }, supply: 0, decimals: 6 }
```

What is proved by this run:

- end-to-end wire integration through the real SPEL framework — risc0-serde instruction encoding, PDA derivation (`compute_pda(program_id, &[seed_from_str("lp0013:mint:v1")])`), `Message::try_new` payload assembly, `WitnessSet::for_message` signing with the wallet's `PrivateKey`, RPC submission, sequencer execution and inclusion;
- four transactions confirmed in sequencer blocks under the deployed program ID `dc199db5…194d5943`;
- correct create-time state write (the mint PDA decodes as `MintDefinition` with the expected `decimals = 6` and `current_authority = Some(authority)`).

What this run originally exposed, and what changed after review:

- In the first host spike, `mint_to` did not update `supply` and `set_mint_authority` did not change the authority field. That was the important honesty gap: the first guest captured the IDL-visible surface (account ordering, signer modes, PDA seeds, arg types) before the offline `mint-core` business logic had been ported into the RISC0 account adapter.
- After review, `spel-spike/admin_authority_guest.rs` was advanced to semantic source: `mint_to` Borsh-decodes mint/holding state, rejects zero amount, enforces the current authority, rejects revoked authority, checks supply/balance overflow, and writes updated post-states; `set_mint_authority` now enforces current-authority authorization and persists rotation/revocation.

The tx hashes above remain useful archival proof that the SPEL/LEZ wire path worked on real local infrastructure with the structural-surface guest. The semantic guest release-candidate rerun is recorded below.

### Net state captured

| Proof | Status | Evidence |
| --- | --- | --- |
| Offline Rust suite | green | 30 unit + validator (8 self-tests) passing on host |
| SPEL IDL generation | green | `idl/admin-authority.idl.spel-generated.json` |
| RISC0 guest build (macOS arm64) | green | image id `b59d19dc…594d19`, 462 KB ELF |
| Local LEZ deploy | green | txhash `2a516235…2a8965`, block 45491 |
| `create_mint` on-chain (real signing, real risc0-serde encoding) | green | txhash `fd68e225…de79c69`, mint PDA decoded as `MintDefinition` |
| `mint_to` accepted by sequencer | green (wire-level) | txhash `07de7c91…3d88811` |
| `set_mint_authority` accepted by sequencer | green (wire-level) | txhash `ec58ace4…7eba204d` |
| Authority semantics enforced in-guest | source ported | `spel-spike/admin_authority_guest.rs` now enforces authorization, revocation, supply/balance updates, and overflow checks; host rerun recommended to refresh tx hashes for the exact release candidate |

The rows above are reproducible from `spel-spike/live_lifecycle.rs` + this proof log against the structural-surface guest captured on 2026-05-17.

## Semantic LEZ rerun (2026-05-18)

After the structural-surface spike, `spel-spike/admin_authority_guest.rs` was advanced to semantic source (Borsh-decoded mint/holding state, authority/revocation enforcement, supply/balance overflow, persisted post-states). One refinement was required to make `mint_to` valid on the LEZ framework: the holding account is now claimed as a program-owned PDA (`#[account(init, pda = literal("lp0013:holding:v1"))]`) because LEZ rejects mutations of default-owned accounts that were never claimed. The semantic guest was rebuilt, redeployed, and the lifecycle re-driven on the same local sequencer.

### Toolchain (rerun)

Same as the host environment at the top of this log; sequencer continued running under the LP-0017 scaffold on `127.0.0.1:3040`, `RISC0_DEV_MODE=0`.

### Guest binary (rerun)

```bash
cd /tmp/lp0013-spike/admin_authority_spike
cp <token-suite>/spel-spike/admin_authority_guest.rs methods/guest/src/bin/admin_authority_spike.rs
make idl
# cp ./admin_authority_spike-idl.json <token-suite>/idl/admin-authority.idl.spel-generated.json
cargo risczero build --manifest-path methods/guest/Cargo.toml
# Finished `release` profile [optimized] target(s) in 3m 19s (warm cache)
# ELFs ready at: .../docker/admin_authority_spike.bin
```

```text
file:    methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin
size:    480,352 bytes
ImageID: 58470667b5d45fcc4317684eb7aaad2b19c0cf666bd8c7f85d2b0e1069d0b960
```

### Deploy (rerun)

```bash
cd ~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower
lgs deploy --program-path /tmp/lp0013-spike/admin_authority_spike/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/admin_authority_spike.bin --json
# {"program":"admin_authority_spike","program_id":"58470667…d0b960","status":"submitted"}
```

Sequencer log captured inclusion at block 49551:

```text
[2026-05-18T07:44:03Z INFO  sequencer_core] Validated transaction with hash
    b16831c0ee550014ea9297ba47d47b31d0c1b425ff3219b44358189bb9204ab5,
    including it in block
[2026-05-18T07:44:03Z INFO  sequencer_service] Block with id 49551 created
```

### Lifecycle driver output (rerun)

```text
== LP-0013 live lifecycle ==
program_id  = [1728464728, 3428832437, 1315444547, 732801719, 1724891161, 4173846635, 269364061, 1622790249]
mint_pda    = 69sLfJLLQ2zrqYbSeQg4dAND7eA5ruRMF9g5Qg1NTAX9
authority   = 2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2
recipient   = J24d7hKF1EEKMuv255NhgJ3ai7feX2niQ5CyNeGdCkjz

[1] create_mint              confirmed tx=7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63
[2] mint_to(100)             confirmed tx=c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f
[3] set_mint_authority(None) confirmed tx=756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c
[4] mint_to(post-revoke)     rejected as expected (no inclusion) tx=27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff
[5] mint state = OnChainMintDefinition { authority: OnChainAuthorityInfo { authority_type: 0,
    current_authority: None }, supply: 100, decimals: 6 }
```

The post-revoke `mint_to` is rejected at the LEZ framework layer with `AccountAlreadyInitialized { account_index: 1 }` because the holding PDA is now program-claimed and the `#[account(init, ...)]` guard refuses a second initialization. The offline `mint-core` tests prove the underlying `require_authority` check (`Program error 2008: authority has been revoked`) on the same code path; the on-chain mint state then independently proves that minting after revocation cannot succeed because `current_authority` is persisted as `None`. Either rejection layer is sufficient — the invariant (post-revoke mints cannot confirm) is preserved.

### Net state captured (rerun)

| Proof | Status | Evidence |
| --- | --- | --- |
| Semantic RISC0 guest build (macOS arm64) | green | image id `58470667…d0b960`, 480,352-byte ELF |
| Semantic-guest LEZ deploy | green | txhash `b16831c0…04ab5`, block 49551 |
| `create_mint` semantic on-chain | green | txhash `7d582e7b…b6f9d63`, mint PDA initialized with `decimals=6`, `current_authority=Some(authority)` |
| `mint_to(100)` semantic on-chain | green | txhash `c474cf82…d3c74f`, mint PDA `supply=100`, holding PDA `balance=100` |
| `set_mint_authority(None)` semantic on-chain | green | txhash `756ee393…7351c`, mint PDA `current_authority=None` persisted |
| `mint_to` post-revoke rejected on chain | green | txhash `27df9483…2bff` not confirmed; `AccountAlreadyInitialized` at the framework layer corroborates the offline `Program error 2008` authority-check rejection |
| Final mint PDA decoded readback | green | `supply=100`, `current_authority=None`, `decimals=6` — matches the expected post-revocation state |

This semantic rerun supersedes the archival structural-surface evidence above. The 2026-05-17 hashes are kept as historical proof of the wire path; the 2026-05-18 hashes are the release-candidate semantic proof. The final public-testnet evidence is documented above and the upstream Logos PR is open at https://github.com/logos-co/lambda-prize/pull/77.

### Independent post-rerun re-verification (2026-05-18, ~22 min after rerun)

To independently verify the chain state we documented, the lifecycle driver was re-executed against the still-running sequencer without resetting state. The final-state readback returned the same decoded `MintDefinition` (`supply=100, current_authority=None, decimals=6`), and the four submission attempts produced sequencer-confirmed rejections that triangulate the on-chain semantics from the opposite direction:

| step | tx hash | sequencer outcome |
| --- | --- | --- |
| re-`create_mint` | `9420757367edf4ea34d104957603505c60e9e7dcde8d4c7db19a64e62ffb7a0b` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 0 }")` — mint PDA already initialized |
| re-`mint_to(100)` | `53d34faa1bfef1b74bb71e5350b6d8b8664796a325933d7250022263a09290a7` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 1 }")` — holding PDA already initialized |
| re-`set_mint_authority(None)` | `cea5b8c7a23ed1e2bbb489284d993257786d15728627666a2c7c7581c1fc5eb4` | `ProgramExecutionFailed("Guest panicked: Program error [8008]: Program error 2008: authority has been revoked")` — canonical `require_authority` rejection from the guest body, on the live LEZ |
| re-`mint_to(post-revoke)` | `83f9c26bba5cdae7fb0793f91036f5da71c699feae74f2cb2f06aff33e316418` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 1 }")` — holding PDA already initialized |

Crucially, the re-`set_mint_authority` attempt reaches the guest body (the mint PDA is already program-owned, so the init check does not apply to it; only the holding PDA gets `init`). The guest then executes `require_authority`, observes `current_authority == None`, and panics with `Program error 2008: authority has been revoked` — the exact semantic rejection the offline `mint-core` tests prove. This is on-chain semantic proof that the post-revocation guard fires for real, not just at the framework layer.

## Current public-testnet refresh status (2026-06-26)

Reviewer `xAlisher` requested a fresh re-deploy/re-anchor because Logos reset `testnet.lez.logos.co` after the June 4 capture. The repository now fails closed when those pre-reset lifecycle transactions are absent; `LP0013_FORCE_CURL_VERIFY=1 bash scripts/demo-testnet-live.sh verify` currently reports the deploy hash present, all public lifecycle hashes absent, and the mint PDA empty.

Fresh refresh attempt on 2026-06-26 (M4 Pro, repo SHA `2a44aa3a57b847e488827bb8b3f09055f27e60bd` before this documentation update):

- Endpoint: `https://testnet.lez.logos.co/`.
- Working read schema: camelCase (`checkHealth`, `getProgramIds`, `getAccountsNonces`, `getAccount`, `getTransaction`). Snake_case methods returned `-32601 Method not found` during this run.
- Submit method: `sendTransaction` exists (`sendTransaction(null)` returns `-32602 Invalid params`, expected string).
- Preflight state: authority nonce `0`; mint PDA and holding PDA empty; candidate lifecycle hashes absent except deploy.
- Program deploy: `sendTransaction` accepted the corrected `ProgramDeploymentTransaction` and returned / confirmed `793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce`.
- First public lifecycle call: `create_mint` was rejected before program execution with JSON-RPC error `-32602 InvalidSignature` (expected candidate hash `3d8617cd607d179354894f2d6457a14c37cc0d0d95e028d9f8f47ad1e2ce6c37`). The guarded executor stopped there; no later lifecycle transactions were submitted.
- Tracked reproducer: `scripts/lp0013-testnet-refresh-readiness.py` performs schema discovery/readiness checks in dry-run mode and only submits if invoked with both `--execute` and `LP0013_I_UNDERSTAND_PUBLIC_SEND=YES`.
- Follow-up compatibility probe: built-in wallet public transactions (`wallet auth-transfer init`) fail with the same `InvalidSignature` on fresh accounts. That means current public testnet accepts deploy transactions but rejects public signed transactions from both the LP-0013 client path and the upstream wallet path.

Conclusion: the reviewer-requested refresh was attempted and reached a precise upstream/tooling blocker. The current public testnet state is **not** a complete refreshed LP-0013 lifecycle. The June 4 lifecycle remains historical proof of the corrected guest, but should not be represented as current live re-query evidence after the reset. Fresh lifecycle hashes require Logos to restore public signed-transaction compatibility, or provide a currently matching wallet/client revision.
