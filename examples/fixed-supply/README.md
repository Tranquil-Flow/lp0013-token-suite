# Fixed Supply Example

Runnable LP-0013 example for a fixed-supply token mint.

## Run

```bash
cargo run -p fixed-supply
```

## What it proves

A fixed-supply mint is represented by initializing the mint authority as revoked:

```text
current authority=None
```

Any later mint attempt is rejected with the same deterministic authority error path used after explicit revocation:

```text
authority has been revoked
```

This keeps fixed-supply and post-revocation behavior unified in one authority model.
