# Config PDA Gated Example

Runnable LP-0013 / RFP-001-style example for a config-derived mint authority gate.

## Run

```bash
cargo run -p config-pda-gated
```

## What it proves

This example derives a deterministic config PDA-like authority from:

```text
lp0013:mint-authority || config_account
```

It then exercises this flow:

1. Create a variable-supply mint whose authority is the derived config gate.
2. Attempt to mint with a different config-derived account and prove rejection.
3. Mint successfully with the authorized config gate.
4. Revoke the gate.
5. Prove post-revoke config minting is rejected.

The derivation in this offline example is intentionally lightweight and deterministic; final submission work still needs a real SPEL/LEZ PDA derivation and `RISC0_DEV_MODE=0` local-sequencer proof.
