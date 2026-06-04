# LP-0013 Demo Video — Narration Script

Recording aid. Estimated 9–11 minutes at a comfortable reading pace.

> **✅ Ready to record (2026-06-04).** The corrected guest is deployed + verified on the public testnet, and this script's on-chain scene narrates the corrected lifecycle — **two accumulating mints (60 + 40 → 100) and a post-revoke mint rejected by the authority guard**. The hashes in `scripts/demo-testnet-live.sh` are the corrected guest's; dry-run `bash scripts/demo-testnet-live.sh verify` once off-camera (it should exit 0) before recording Scene 5a. See `../RESUBMISSION_STATUS.md`.

## What this video demos

LP-0013 adds **mint authority lifecycle** support to the LEZ Token program — variable-supply tokens, authority rotation, permanent revocation. The video has three parts:

1. **Offline Rust** — show the authority logic works end-to-end in pure Rust (fast, deterministic; green across the workspace including the repeated-mint / post-revoke-guard contract test).
2. **IDL** — show the authoritative `spel generate-idl` output (four instructions, full account bodies; rc1 and rc3 pin generations are byte-identical) and the hand-written design reference.
3. **On-chain** — show the corrected authority logic deployed and exercised on the **public LEZ testnet** (`testnet.lez.logos.co`, real consensus, `RISC0_DEV_MODE=0`): create_mint, create_holding, two accumulating mints, revoke, and a post-revoke mint rejected by the guard — re-verified live on camera; the local sequencer run is shown afterward as white-box corroboration (the only place the exact guest-panic string is visible).

> **On-chain evidence must be captured fresh after the re-deploy.** The earlier localnet run is corroboration only — λPrize reviewers reject localnet-only evidence, so the load-bearing proof must be the shared no-auth network running the *corrected* guest.

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

I'll walk through it in three parts: the offline Rust implementation, the IDL, then the actual on-chain proof — and that proof runs on the public LEZ testnet, not a local sequencer.

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
That's cargo format, full test suite, clippy with errors-as-warnings, and a docs validator. Thirty unit tests pass — including the repeated-mint / post-revoke-guard contract test. All green.

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

[SHOW] Open `idl/admin-authority.idl.spel-generated.json`.

[SAY]
This is the authoritative IDL — real `spel generate-idl` output from our guest source, exactly what `make idl` emits. It carries all four instructions — create_mint, create_holding, mint_to, set_mint_authority — plus the full account bodies.

[SHOW] Open `idl/admin-authority.idl.spel-generated.rc3-testnet.json`.

[SAY]
This is the same generator run under the testnet-matching pins, release candidate three. Under the corrected guest it comes out byte-identical to the release-candidate-one generation — so we ship both labels as a cross-revision stability check, not as a richer-versus-poorer pair.

[SHOW] Open `idl/admin-authority.idl.json`.

[SAY]
And this is a hand-written design reference. It documents the discriminators, the execution block, and the declared errors the generator doesn't emit. Its metadata caveats are explicit that those discriminators are illustrative — the runtime dispatches by enum-variant index — and that its args model the offline API. For the on-chain surface, the spel-generated IDL is authoritative.

---

## Scene 5 — On-chain proof, public testnet  (~3-4 min)

[SHOW] Open `docs/LEZ_PROOF_LOG.md`, scroll to "Public testnet deploy + lifecycle (2026-06-04) — CORRECTED GUEST (load-bearing)".

[SAY]
Now the on-chain part — and this is the heart of the submission.

We took the offline Rust authority logic, ported it into a SPEL guest binary, and deployed it to the **public LEZ testnet** — the shared, no-auth network at `testnet.lez.logos.co`, real consensus, real proving. Not a local sequencer. Anyone watching this can re-check every hash I'm about to show, from their own machine.

[SHOW] Point at the endpoint block — `https://testnet.lez.logos.co/` and the explorer URL.

[SAY]
One thing worth calling out: the testnet runs release candidate three of the LEZ stack. Our original guest was built against release candidate one, and that binary simply won't execute here — the core program module differs by a few hundred lines. So we re-pinned every dependency to the exact commit the testnet runs and rebuilt. That's the program ID on screen.

[SHOW] Point at the lifecycle driver output — the `[0]` through `[8]` block.

[SAY]
Here's the full lifecycle, all on the public testnet.

`deploy_program` — confirmed. The program is live on chain.

`create_mint` — confirmed. A variable-supply mint with `decimals=6` and a designated authority.

`create_holding` — confirmed. This claims the recipient's holding account once, up front — and it's the heart of the fix I'll explain in a second.

`mint_to(60)` then `mint_to(40)` — both confirmed, into that *same* holding. Supply accumulates: 60, then 100. That's variable supply working on chain — two separate mints adding to one balance.

`set_mint_authority(None)` — confirmed. The authority field is now persisted as `None` — revoked.

The last one is a post-revocation `mint_to` of seven units, into the holding that already exists. It is never included — rejected.

And here's why that rejection matters. In the earlier broken version, the holding was created *on every mint*, so a second mint — or a post-revocation mint — was rejected just because the account already existed, before the authority check ever ran. The guard was never actually exercised. In this corrected version the holding is created once and then mutated, so the post-revocation mint reaches the guest body and is rejected by the authority guard itself — `require_authority`, error 2008 — not by an account-already-exists side effect.

Final readback: the mint shows `supply = 100`, `current_authority = None`, `decimals = 6`, and the holding shows `balance = 100`. Two accumulating mints landed; the post-revocation mint did not.

[SAY]
And one finding worth flagging: across deploy and all three executions, the signer's balance never moved — 150 to 150 — only the nonce incremented. On this network, public-state program execution is proved sequencer-side and charges no gas to the submitter.

### 5a. Live re-verification on camera  (~1 min)

[SHOW] Terminal in repo root.

[SAY]
I don't want you to take my word for those hashes. Let's re-query them live, right now, straight from the public sequencer.

[SHOW]
```
bash scripts/demo-testnet-live.sh verify
```
*wait for output — this hits `testnet.lez.logos.co` live*

[SAY]
This script needs nothing but the wallet binary — no build, no faucet, no local state. It points a throwaway wallet at the public testnet and re-queries each transaction.

[SHOW] Let the verdict lines scroll: deploy → `Some(ProgramDeployment)`, the three executions → `Some(Public)`, the post-revoke tx → `Transaction is None`.

[SAY]
Deploy resolves as a program deployment. The three lifecycle transactions resolve as public transactions. The post-revocation mint resolves as `None` — it was never included in a block.

[SHOW] Let the mint-PDA readback + decode print.

[SAY]
And finally the script reads the mint account straight off chain and decodes its raw bytes: authority is `None`, supply is 100 — not 107. If that post-revocation mint of seven had succeeded, supply would read 107. It reads 100. The revocation held, on the live network.

---

## Scene 5b — Real proving, and the white-box corroboration  (~1-1.5 min)

[SAY]
The spec asks to see real proof execution under `RISC0_DEV_MODE=0`.

The public testnet *is* that, by construction — it's a real consensus network, proving happens on the sequencer, and there's no dev-mode shortcut available to me as an outside submitter. That's the strongest form of the claim: not "I turned dev mode off on my own machine," but "this ran on someone else's production network."

[SAY]
The one thing a public network won't give me is its internal sequencer log — so it can't show you the raw zkVM executor timings or the exact panic string from inside the guest. For that white-box view, here's our local sequencer run, which we configured explicitly with `RISC0_DEV_MODE=0`.

[SHOW] Open `docs/LEZ_PROOF_LOG.md`, scroll to "Independent post-rerun re-verification (2026-05-18)".

[SHOW] Highlight the `set_mint_authority` row's panic message.

[SAY]
On the local run we re-submitted `set_mint_authority` against the already-revoked mint, and the sequencer logged the guest panicking with *"Program error 2008: authority has been revoked."* That's the canonical `require_authority` check firing inside the guest body — the exact same semantic guard, now visible at the source-error level.

[SAY]
So: the public testnet proves the state-level invariant — minting after revocation cannot land. The local run, with its visible logs, shows you the precise in-guest error that enforces it. Two independent angles on the same property.

---

## Scene 6 — Compute cost  (~30s)

[SHOW] Open `docs/BENCHMARKS.md`, scroll to "Compute units (CU)".

[SAY]
The compute profile. These figures come from the local sequencer run, because that's the only place the per-transaction executor timings are exposed — the public testnet hides its sequencer log. They're single-run numbers from the semantic guest's localnet session; the methodology carries over unchanged to the corrected four-instruction guest.

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

The same authority semantics are deployed and exercised on the **public LEZ testnet** — a deploy plus the full corrected lifecycle (create_mint, create_holding, two accumulating mints, revoke, and a post-revoke mint rejected by the guard), with the revocation invariant re-verified live and straight off chain. That's the load-bearing evidence, and you can reproduce it yourself with one command.

The offline Rust authority semantics are unit-tested and runnable.

The IDL is generated by the real SPEL framework (authoritative for the on-chain surface), and a hand-written design reference documents the discriminators / execution / errors the generator omits.

And the local sequencer run corroborates it with the white-box view: the exact in-guest revocation panic under `RISC0_DEV_MODE=0`.

Outside the scope of this submission: end-to-end CI against a LEZ sequencer — the local toolchain is host-only — and any claim of Logos endorsement or audit.

Thanks for evaluating LP-0013.

[SHOW] End on the README.

---

## Recording checklist

- [ ] **Scene 5a is the load-bearing shot — it runs live against the public testnet.** The corrected guest is already deployed and the hashes in `scripts/demo-testnet-live.sh` are current; dry-run `bash scripts/demo-testnet-live.sh verify` once off-camera and confirm it exits 0 with every verdict matching (deploy → `ProgramDeployment`; create_mint / create_holding / two mints / set_mint_authority → `Public`; post-revoke mint → `None`) and the PDA decodes show mint `supply=100, authority=None, decimals=6` and holding `balance=100`. The only prerequisite is the `wallet` binary on PATH (built from LEZ `v0.1.2` / `v0.2.0-rc3`); no localnet, no faucet.
- [ ] Network reachable: `curl -s -o /dev/null -w '%{http_code}' https://testnet.lez.logos.co/` should be reachable (a bare GET returns `405 POST required` — that's healthy). If the testnet is down at record time, pause and retry rather than narrating stale hashes as if live.
- [ ] Scene 5b corroboration is read off `docs/LEZ_PROOF_LOG.md` (no live localnet needed) — you're pointing at the already-captured 2026-05-18 panic line, not re-running the local sequencer. If you *do* want to re-run it live, the local sequencer at `127.0.0.1:3040` must be up under the LP-0017 scaffold with `RISC0_DEV_MODE=0`.
- [ ] Terminal font ≥ 16 pt.
- [ ] Editor wraps long lines so tx hashes don't word-wrap mid-hex.
- [ ] No private wallet paths or seed phrases visible. The throwaway testnet wallet `demo-testnet-live.sh` creates is fine to show; do not show `.nssa-testnet-wallet` or any funded/keyed home.
- [ ] No private chat windows, Slack, or DMs visible.
- [ ] After recording, host the video somewhere stable (YouTube unlisted, Loom) and paste the link into `SUBMISSION.md` under "Demo video" + the README video line, **replacing** the old `youtu.be/3hQd2G8O-UM` localnet link, before opening the λPrize PR.
