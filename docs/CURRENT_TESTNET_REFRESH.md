# LP-0013 current public-testnet refresh — 2026-06-27

This is the current load-bearing public LEZ evidence for LP-0013 after the public testnet/client drift discovered during resubmission prep. Older 2026-06-03 and 2026-06-04 hashes are retained in `docs/LEZ_PROOF_LOG.md` as historical context only.

## Network and client

- Network: `https://testnet.lez.logos.co/`
- JSON-RPC method shape: camelCase (`getTransaction`, `getAccount`)
- Client/signing path used for fresh lifecycle: LEZ v0.2.0 wallet / `WalletCore` public transaction path
- Program artifact: release RISC Zero guest ELF wrapped as `risc0_binfmt::ProgramBinary`
- ProgramBinary SHA-256: `fac6f9715efc03edcb695dc71545cb24fac6bc86530644e2748f50d6ef9009f3`

## Program and accounts

- ProgramId words: `[3915745331, 4212693844, 3927990387, 3805644783, 632583424, 247468046, 2358372872, 1247113544]`
- ProgramId / ImageID bytes: `338865e9549b18fb736020eaef87d5e20075b4250e10c00e08ea918c4871554a`
- Explorer-form base58 ProgramId: `4UARaVcJJoLxebFAobocsZyzpJ5TTUvvhRtFuHtuHypd`
- Authority: `6HEYFUW4QbHPfdHTMPZLDeC6F5PL6suhSGJbTnsauhWJ`
- Mint PDA: `4gMBXeUskbUTzxoP8fJJEXj3jxTQz91m6ZW7fMsLMJq6`
- Holding PDA: `366n7Nj21EzD27BXRKE2hFDWPtJ1E2Fcx9RmqQoGRD7h`

## Included lifecycle transactions

| Step | Transaction hash | Verdict |
|---|---|---|
| deploy_program | `793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce` | included as `ProgramDeployment` |
| create_mint | `55908821088c98e898c4ef99e9a36e02856092f7afd0155f3457c25c5cf67746` | included as `Public` |
| create_holding | `8a37a8fb7200856c57d199ce081f2b744ed3cbaeec8326c83092f5ca05ac668f` | included as `Public` |
| mint_to(60) | `daf5aa91f35dff8250794c0dcfe932de473c651bd25c946d76f09a42cfdb6a97` | included as `Public` |
| mint_to(40) | `ed07b29c004a796d504814ddf1a9a0cfda373d1618398b620e330ccb529b3cce` | included as `Public` |
| set_mint_authority(None) | `719123f918df2aee42c4e69d36ba8860807b2a69c97a2927097d8313a508550e` | included as `Public` |
| mint_to(post-revoke) | `016043771c0cc60efaf158ec120a9bf341326967c881285878469503ddd3d4fa` | not included, as expected |

## Final state readback

- Mint state: `authority=None`, `supply=100`, `decimals=6`
- Holding state: `balance=100`
- Result: post-revoke mint did not land; fixed-supply invariant held.

## Re-verification

Run from the implementation repository:

```bash
bash scripts/demo-testnet-live.sh verify
# or the curl-only CI verifier:
bash scripts/ci-verify-testnet.sh
```

Both scripts are read-only. They query the public sequencer and fail closed if the public testnet resets again or if final state no longer matches the recorded lifecycle.

## Root-cause notes for evaluator trust

- The rc3 wallet path could submit deployments but produced `InvalidSignature` for current-testnet custom `PublicTransaction` calls. The v0.2.0 wallet path signs them correctly.
- LEZ expects a RISC Zero `ProgramBinary`, not a raw RISC-V ELF. Raw/debug ELF attempts produced `InvalidProgramBytecode(Malformed ProgramBinary)` or oversized payloads.
- The current successful path used a release guest ELF wrapped as `ProgramBinary`; all lifecycle transactions were included and final account state was read back.
