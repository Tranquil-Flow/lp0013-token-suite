# Variable Supply Example

Runnable LP-0013 example for a token mint with an active mint authority.

## Run

```bash
cargo run -p variable-supply
```

## What it proves

This example exercises the evaluator-facing authority lifecycle:

1. Create a variable-supply mint with decimals set to 6.
2. Mint initial supply using the current authority.
3. Rotate authority to a new account.
4. Prove the old authority is rejected.
5. Mint additional supply with the new authority.
6. Revoke authority.
7. Prove post-revoke minting is rejected deterministically.

Expected final state:

- supply: 125
- holder balance: 125
- current authority: revoked / `None`
