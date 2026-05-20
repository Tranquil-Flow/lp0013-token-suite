# Host Logos Toolchain Handoff

This sandbox does not have the Logos execution toolchain installed, but LP-0017 was previously built on this computer outside the sandbox. The real `spel` / `lgs` / `cargo-risczero` proof should therefore be attempted from the host environment that was used for LP-0017, likely the Claude Code macOS shell rather than this container.

## Goal

Turn the current offline LP-0013 authority suite into a final Logos-ready proof:

1. confirm host Logos tooling,
2. regenerate or replace fallback IDL with real SPEL output,
3. run a local LEZ/RISC0 proof with `RISC0_DEV_MODE=0`,
4. capture logs for the final submission.

## Preflight commands on the host

Run from the public-submission repo root:

```bash
pwd
which cargo rustc || true
which spel || true
which lgs || true
which logos-scaffold || true
which cargo-risczero || true
cargo --version
rustc --version
spel --version || true
lgs --version || true
cargo risczero --version || cargo-risczero --version || true
```

If the commands are missing on the host too, use the known LP-0017 setup notes in the umbrella project before proceeding:

```text
~/Projects/logos-basecamp/lp-0017-whistleblower/ARCHITECTURE.md
~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/BUGS_FILED.md
~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/DEMO.md
```

Important LP-0017 gotchas that likely still matter:

- local is devnet; do not wait for a separate remote devnet URL,
- use `RISC0_DEV_MODE=0` for the final proof path,
- avoid pulling `spel-framework` host dependencies into the RISC0 guest on macOS arm64,
- use raw `nssa_core` in guest code if the SPEL host dependency graph causes the known `ring` / `riscv32-unknown-elf-gcc` failure,
- use PDA claims correctly: PDA-owned accounts need PDA claim semantics rather than signer authorization semantics.

## Current sandbox result

Inside this sandbox, these tools are available:

```text
cargo
rustc
```

These tools are not available:

```text
spel
lgs / logos-scaffold
cargo-risczero
```

Therefore the current committed status is honest:

- offline Rust authority suite proven,
- fallback IDL committed at `idl/admin-authority.idl.json`,
- fallback IDL test-guarded by `admin-authority-spel`,
- real SPEL/LEZ proof pending host-side execution.

## Host-side proof outline

### 1. Fresh local gate

```bash
bash scripts/check-prereqs.sh
bash scripts/demo.sh
```

### 2. SPEL IDL regeneration attempt

Preferred final command shape:

```bash
spel generate-idl methods/guest/src/bin/admin_authority.rs > /tmp/admin-authority-idl.generated.json
python3 -m json.tool /tmp/admin-authority-idl.generated.json >/tmp/admin-authority-idl.generated.pretty.json
diff -u idl/admin-authority.idl.json /tmp/admin-authority-idl.generated.pretty.json || true
```

If generated output is correct, replace the fallback artifact:

```bash
cp /tmp/admin-authority-idl.generated.pretty.json idl/admin-authority.idl.json
```

Then update:

```text
docs/SPEL_STATUS.md
docs/SPEC_COMPLIANCE.md
SUBMISSION.md
```

### 3. LEZ local-sequencer proof

Exact commands depend on the final SPEL guest wiring. The proof must include:

```bash
export RISC0_DEV_MODE=0
export NSSA_SEQUENCER_URL=http://127.0.0.1:3040
```

Capture:

- local sequencer launch command,
- program build command,
- program deploy command,
- create mint transaction,
- mint transaction,
- rotate authority transaction,
- rejected old-authority mint,
- revoke authority transaction,
- rejected post-revoke mint,
- final inspect/query output,
- timing/CU or equivalent execution cost data if available.

### 4. Proof log file

Recommended log path before final PR:

```text
docs/LEZ_PROOF_LOG.md
```

Minimum contents:

- date,
- host OS and architecture,
- `cargo`, `rustc`, `spel`, `lgs`, `cargo-risczero` versions,
- exact commands,
- exact outputs or summarized outputs with transaction hashes,
- statement that `RISC0_DEV_MODE=0` was set,
- remaining caveats, if any.

## Do not do automatically

Do not push the public repo or open the Logos PR until Evi explicitly approves the final state.
