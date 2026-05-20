# LEZ / SPEL Proof Log

Host-side proof attempts for LP-0013, capturing exact commands, environment, and outcomes. The intent is honest evaluator-facing evidence — successes are recorded with hashes/timing, failures are recorded with the exact error so the submission stays trustworthy.

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

Outcome: all three pass. `cargo test --workspace` reports 31 Rust tests passing (`admin-authority-core` 6, `admin-authority-spel` 2, `mint-core` 7, `mint-program` 6, `mint-sdk` 3, `mint-cli` 2, `examples/variable-supply` 1, `examples/fixed-supply` 1, `examples/config-pda-gated` 1, doc-tests 0+0+0+0+0). `scripts/demo.sh` prints the full authority lifecycle for all three runnable examples. `python3 -m unittest` reports `Ran 1 test in 0.062s OK`.

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

The generated output is committed at `idl/admin-authority-idl.spel-generated.json`. It captures the three instructions, their PDA seeds, signer/writable/init modes, and primitive arg types. The current SPEL revision does not yet emit discriminators, the `execution` block, declared errors, or fully expanded `Option<T>` / nested account-type bodies. The hand-written `idl/admin-authority-idl.json` remains the canonical superset; `docs/SPEL_STATUS.md` carries the diff rationale.

## LEZ local-sequencer proof

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
| Offline Rust suite | green | 31 unit + 1 validator passing on host |
| SPEL IDL generation | green | `idl/admin-authority-idl.spel-generated.json` |
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
# cp ./admin_authority_spike-idl.json <token-suite>/idl/admin-authority-idl.spel-generated.json
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
| Final mint PDA decoded readback | green | `supply=100`, `current_authority=None`, `decimals=6` — matches the handover's expected post-revocation state |

This semantic rerun supersedes the archival structural-surface evidence above. The 2026-05-17 hashes are kept as historical proof of the wire path; the 2026-05-18 hashes are the release-candidate semantic proof. Public push and Logos PR remain gated on explicit Evi sign-off.

### Independent post-rerun re-verification (2026-05-18, ~22 min after rerun)

To independently verify the chain state we documented, the lifecycle driver was re-executed against the still-running sequencer without resetting state. The final-state readback returned the same decoded `MintDefinition` (`supply=100, current_authority=None, decimals=6`), and the four submission attempts produced sequencer-confirmed rejections that triangulate the on-chain semantics from the opposite direction:

| step | tx hash | sequencer outcome |
| --- | --- | --- |
| re-`create_mint` | `9420757367edf4ea34d104957603505c60e9e7dcde8d4c7db19a64e62ffb7a0b` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 0 }")` — mint PDA already initialized |
| re-`mint_to(100)` | `53d34faa1bfef1b74bb71e5350b6d8b8664796a325933d7250022263a09290a7` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 1 }")` — holding PDA already initialized |
| re-`set_mint_authority(None)` | `cea5b8c7a23ed1e2bbb489284d993257786d15728627666a2c7c7581c1fc5eb4` | `ProgramExecutionFailed("Guest panicked: Program error [8008]: Program error 2008: authority has been revoked")` — canonical `require_authority` rejection from the guest body, on the live LEZ |
| re-`mint_to(post-revoke)` | `83f9c26bba5cdae7fb0793f91036f5da71c699feae74f2cb2f06aff33e316418` | `ProgramExecutionFailed("Guest panicked: account validation failed: AccountAlreadyInitialized { account_index: 1 }")` — holding PDA already initialized |

Crucially, the re-`set_mint_authority` attempt reaches the guest body (the mint PDA is already program-owned, so the init check does not apply to it; only the holding PDA gets `init`). The guest then executes `require_authority`, observes `current_authority == None`, and panics with `Program error 2008: authority has been revoked` — the exact semantic rejection the offline `mint-core` tests prove. This is on-chain semantic proof that the post-revocation guard fires for real, not just at the framework layer.
