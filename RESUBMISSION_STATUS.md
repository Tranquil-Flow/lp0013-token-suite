# LP-0013 — final submission status

**As of 2026-06-27:** LP-0013 is complete for public evaluation.

The corrected four-instruction guest is deployed on the public LEZ testnet (`testnet.lez.logos.co`, `RISC0_DEV_MODE=0`) with ProgramId/ImageID `338865e9549b18fb736020eaef87d5e20075b4250e10c00e08ea918c4871554a` (base58 `4UARaVcJJoLxebFAobocsZyzpJ5TTUvvhRtFuHtuHypd`). The final narrated demo video is recorded at https://youtu.be/rUgsCCPiQfo.

## Final evidence

| Step | Transaction hash | Verdict |
|---|---|---|
| deploy_program | `793992258d88e69c63cbede6fabec3ff5768d84d824d7ee9f3170f85fb717dce` | `Some(ProgramDeployment)` |
| create_mint | `55908821088c98e898c4ef99e9a36e02856092f7afd0155f3457c25c5cf67746` | `Some(Public)` |
| create_holding | `8a37a8fb7200856c57d199ce081f2b744ed3cbaeec8326c83092f5ca05ac668f` | `Some(Public)` |
| mint_to(60) | `daf5aa91f35dff8250794c0dcfe932de473c651bd25c946d76f09a42cfdb6a97` | `Some(Public)` |
| mint_to(40) | `ed07b29c004a796d504814ddf1a9a0cfda373d1618398b620e330ccb529b3cce` | `Some(Public)` |
| set_mint_authority(None) | `719123f918df2aee42c4e69d36ba8860807b2a69c97a2927097d8313a508550e` | `Some(Public)` |
| mint_to(post-revoke) | `016043771c0cc60efaf158ec120a9bf341326967c881285878469503ddd3d4fa` | not included (`Transaction is None`) |

Final decoded state:

- Mint PDA: `4gMBXeUskbUTzxoP8fJJEXj3jxTQz91m6ZW7fMsLMJq6`
- Holding PDA: `366n7Nj21EzD27BXRKE2hFDWPtJ1E2Fcx9RmqQoGRD7h`
- Authority: `None`
- Supply: `100`
- Decimals: `6`
- Holding balance: `100`

Re-verify read-only from a clean clone:

```bash
bash scripts/demo-testnet-live.sh verify
```

## Requirement coverage

| Requirement | Status |
|---|---|
| Variable-size tokens through minting authority | Complete: two public-testnet mints accumulate to `100`. |
| Authority set at initialization | Complete: `create_mint` initializes authority-controlled mint state. |
| Authority rotation/revocation | Complete: `set_mint_authority(None)` confirms on public testnet. |
| Deterministic revoked-authority rejection | Complete: post-revoke mint is rejected after the holding exists, so the guard is exercised. |
| Documentation and examples | Complete: fixed-supply, variable-supply, and config-PDA-gated examples are included. |
| RFP-001-style reusable authority library | Complete: `admin-authority-core` provides reusable authority checks/state. |
| SDK/module support | Complete: `mint-sdk`, `mint-cli`, and `onchain-program/examples` are included. |
| SPEL IDL | Complete: SPEL-generated IDL artifacts are under `idl/`. |
| LEZ devnet/testnet deployment | Complete: public-testnet deployment and tx hashes are documented above. |
| Local sequencer demo under `RISC0_DEV_MODE=0` | Complete: `scripts/demo-localnet.sh` provides the reproducible local path. |
| Narrated video | Complete: https://youtu.be/rUgsCCPiQfo. |

The public-testnet evidence above is the final evidence for the corrected lifecycle.
