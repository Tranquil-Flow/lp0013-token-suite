# LP-0013 Demo Video — Narration Script

Recording aid. Estimated 9–11 minutes at a comfortable reading pace.

## What this video demos

LP-0013 adds **mint authority lifecycle** support to the LEZ Token program — variable-supply tokens, authority rotation, permanent revocation. The video has three parts:

1. **Offline Rust** — show the authority logic works end-to-end in pure Rust (fast, deterministic, 31 unit tests).
2. **IDL** — show both the hand-written canonical IDL and the real `spel generate-idl` output.
3. **On-chain** — show the same authority logic running on a live LEZ sequencer with real Risc0 proof execution.

## Format conventions

- **[SHOW]** — what's on screen.
- **[SAY]** — read this aloud (one short paragraph at a time, pause between).
- *wait for output* — let the command finish before continuing.

---

## Scene 1 — What this is  (~45s)

[SHOW] Repository root, `README.md` open.

[SAY]
Hi, I'm submitting LP-0013 for the Logos λPrize: token program improvements, authorities.

Right now LEZ tokens have a fixed supply at creation. With this submission you can do three new things — create a token with a designated mint authority that can mint more later, rotate that authority to another account, or permanently revoke it, which locks the supply forever.

I'll walk through it in three parts: the offline Rust implementation, the IDL, then the actual on-chain proof.

---

## Scene 2 — Local verification gate  (~30s)

[SHOW] Terminal in repo root.

[SAY]
First, the local gate.

[SHOW]
```
bash scripts/check-prereqs.sh
```
*wait for output*

[SAY]
That's cargo format, full test suite, clippy with errors-as-warnings, and a docs validator. Thirty-one unit tests pass. All green.

---

## Scene 3 — Offline lifecycle demo  (~2 min)

[SHOW]
```
bash scripts/demo.sh
```
*wait for output to begin*

[SAY]
This runs three example flows back-to-back.

### 3a. Variable-supply flow  (~45s)

[SHOW] Let the `variable-supply` example scroll.

[SAY]
First, a variable-supply token. The mint is created with `decimals=6` and an initial authority. The authority mints 100 units. Then we rotate the authority to a new key — and the old key's next mint attempt is rejected with "signer is not the configured authority". The new key mints 25 more units. Supply is now 125.

Then we revoke — `set_mint_authority None`. After that, every mint attempt is rejected with "authority has been revoked". Failed instructions don't partially mutate state.

### 3b. Fixed-supply flow  (~20s)

[SHOW] `fixed-supply` example output.

[SAY]
Second, a fixed-supply token. This one is created with `current_authority = None` from the start. Every mint attempt is rejected, same error. Revocation isn't a later state change — it's the initial state.

### 3c. RFP-001 config-PDA gate  (~30s)

[SHOW] `config-pda-gated` example output.

[SAY]
Third, the RFP-001 reusable authority library that the prize references. A program-derived address acts as the authority. An unauthorized config is rejected. The authorized config mints 64 units. Then we revoke the gate, and the post-revocation mint is rejected with the same error.

Same authority model, different key derivation — this proves the library composes with the LEZ PDA mechanism.

---

## Scene 4 — IDL artifacts  (~45s)

[SHOW] Open `idl/admin-authority-idl.json`.

[SAY]
The IDL ships as two files. This one is hand-written and canonical. It includes instruction discriminators, declared errors, and expanded `Option` types — things the current SPEL framework revision doesn't emit yet.

[SHOW] Open `idl/admin-authority-idl.spel-generated.json`.

[SAY]
This second one is real `spel generate-idl` output, run against our guest source on the same LEZ tag and SPEL revision LP-0017 uses. Both files describe the same instruction set. We ship both as evidence.

---

## Scene 5 — On-chain proof  (~2-3 min)

[SHOW] Open `docs/LEZ_PROOF_LOG.md`, scroll to "Semantic LEZ rerun".

[SAY]
Now the on-chain part.

We took the offline Rust authority logic and ported it into a SPEL guest binary. That guest decodes Borsh account state, runs the authority checks, persists the rotation or revocation. We built the guest, deployed it to a live local LEZ sequencer, and ran the full lifecycle.

[SHOW] Point at the tx hash table.

[SAY]
Four transactions on chain.

`create_mint` — confirmed.

`mint_to(100)` — confirmed. On-chain supply went from zero to 100. The holding account balance went from zero to 100.

`set_mint_authority(None)` — confirmed. The on-chain authority field is now persisted as `None`.

The fourth transaction is a post-revocation `mint_to`. Rejected, as expected.

Final readback of the mint shows: `supply = 100`, `current_authority = None`, `decimals = 6`. Matches the expected post-revocation state exactly.

[SHOW] Scroll to "Independent post-rerun re-verification".

[SAY]
About 22 minutes after that lifecycle, we ran the driver again — submitting `set_mint_authority` a second time against the now-revoked mint. Here's what the sequencer logged.

[SHOW] Highlight the panic message.

[SAY]
*"Guest panicked: Program error 2008: authority has been revoked."* That's the canonical `require_authority` check, firing inside the guest body, observed live on chain. The mint's revoked state persisted across runs. The authority check is real.

---

## Scene 5b — Proof generation under `RISC0_DEV_MODE=0`  (~30s)

[SHOW] Terminal.

[SHOW]
```
env | grep RISC0_DEV_MODE
```
*should print* `RISC0_DEV_MODE=0`

[SAY]
The spec asks for terminal output showing real proof execution under `RISC0_DEV_MODE=0`. Our sequencer was configured with `risc0_dev_mode = false`. Every transaction was processed by the real Risc0 zkVM — no dev-mode shortcut.

[SHOW]
```
grep "execution time" \
  ~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/logs/sequencer.log \
  | grep "2026-05-18T07:4[345]" | head -10
```
*wait for output*

[SAY]
Each `risc0_zkvm` execution time line is the real zkVM running our guest. The numbers themselves I'll cover in the next scene.

---

## Scene 6 — Compute cost  (~30s)

[SHOW] Open `docs/BENCHMARKS.md`, scroll to "Compute units (CU)".

[SAY]
The CU figures, pulled from the same sequencer log.

`create_mint` — 8.38 milliseconds.
`mint_to` — 7.58 milliseconds.
`set_mint_authority` — 6.76 milliseconds. That's the same code path for both rotation and revocation.

Rejected post-revocation operations cost about half as much — between 4.2 and 4.4 milliseconds — because execution halts at the authority check before any state write.

That deterministic-rejection property the spec calls for is visible in the compute cost itself.

---

## Scene 7 — Closing  (~30s)

[SHOW] Back to repo root.

[SAY]
To recap what's proven:

The offline Rust authority semantics are unit-tested and runnable.

The IDL is generated by the real SPEL framework and shipped alongside the hand-written canonical version.

The same semantics run on the LEZ sequencer — four-transaction lifecycle confirmed, post-revocation guard fired live in the guest body.

Outside the scope of this submission: end-to-end CI against a LEZ sequencer — the local toolchain is host-only — and any claim of Logos endorsement or audit.

Thanks for evaluating LP-0013.

[SHOW] End on the README.

---

## Recording checklist

- [ ] Local sequencer at `127.0.0.1:3040` with `RISC0_DEV_MODE=0` if re-running live. Otherwise narrate the already-recorded tx hashes from `docs/LEZ_PROOF_LOG.md`.
- [ ] Sequencer log file readable for Scene 5b — verify `~/Projects/logos-basecamp/lp-0017-whistleblower/whistleblower/.scaffold/logs/sequencer.log` exists and contains the 2026-05-18 entries (run the Scene 5b grep once off-camera to confirm).
- [ ] Terminal font ≥ 16 pt.
- [ ] Editor wraps long lines so tx hashes don't word-wrap mid-hex.
- [ ] No private wallet paths visible (the `NSSA_WALLET_HOME_DIR` env line is fine — it references the LP-0017 scaffold, which is public).
- [ ] No private chat windows, Slack, or DMs visible.
- [ ] After recording, host the video somewhere stable (YouTube unlisted, Loom) and paste the link into `SUBMISSION.md` under "Demo video link" before opening the λPrize PR.
