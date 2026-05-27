# Bugs Filed — LP-0013 Token Authorities

Per LP-0013 spec submission requirements, this file documents upstream Logos issues encountered while building.

## Filed upstream

| # | Repo | Issue | URL |
|---|---|---|---|
| 1 | logos-blockchain/logos-blockchain-circuits | #33 — README missing install path docs (`~/.logos-blockchain-circuits`) | https://github.com/logos-blockchain/logos-blockchain-circuits/issues/33 |

## Worked around

### 4. LEZ tag vs rev pinning causes Cargo duplicate crate errors
**Symptom:** Pinning LEZ deps to both a tag and a rev causes duplicate-type errors.
**Workaround:** Pin all LEZ deps uniformly to the same tag.

### 5. `lgs localnet reset` requires `--yes` flag but error doesn't mention it
**Workaround:** Always use `lgs localnet reset --yes` in scripts.

### 6. LEZ template host runners reference removed API fields
**Symptom:** Fresh `lgs create` workspace fails to compile with `no field 'tx_hash'`.
**Workaround:** Delete template runner files and write against current `wallet`/`nssa` API directly.

### 7. Docker Desktop not auto-detected for `cargo risczero build`
**Workaround:** Always `export PATH="/Applications/Docker.app/Contents/Resources/bin:$PATH"` before building.

## Devnet clarification

Per fryorcraken on Logos Discord (2026-05-11): "devnet == localnet. We don't have a public testnet for lez (yet)." The local sequencer launched by `lgs localnet start` IS the LEZ devnet for lambda prize evaluation.
