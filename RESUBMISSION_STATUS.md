# LP-0013 — final submission status

**As of 2026-06-04:** LP-0013 is complete for public evaluation.

The corrected four-instruction guest is deployed on the public LEZ testnet (`testnet.lez.logos.co`, `RISC0_DEV_MODE=0`) with ProgramId/ImageID `32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce` (base58 `4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF`). The final narrated demo video is recorded at https://youtu.be/rUgsCCPiQfo.

## Final evidence

| Step | Transaction hash | Verdict |
|---|---|---|
| deploy_program | `5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0` | `Some(ProgramDeployment)` |
| create_mint | `7d1dcb04b5f339b33f04a120b7334cf9802720d4a917e600becd62476e44da74` | `Some(Public)` |
| create_holding | `520d080b833c7e4038a1aa214bba43a3fc97328e8f379a093b74ca3e32be5893` | `Some(Public)` |
| mint_to(60) | `8c865d0184f55ce5a881e24c8c125cd3729c5f90a4b83d0484c8d1610f743f61` | `Some(Public)` |
| mint_to(40) | `c63168b7f615221ab2425b2ba003d32183f4df2e482eb4203e4e216675993d21` | `Some(Public)` |
| set_mint_authority(None) | `8c4b08b5c750c57d0dbb4e9f43c32b7c0f2627ce5508da85408e3aaf01f5a331` | `Some(Public)` |
| mint_to(post-revoke) | `6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1` | not included (`Transaction is None`) |

Final decoded state:

- Mint PDA: `HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu`
- Holding PDA: `4yswbZaRR1HQt4a5HS4uN7nLvAwL1txHTMSXKo1WZH2S`
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
