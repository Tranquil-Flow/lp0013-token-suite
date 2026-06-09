import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate-submission-docs.py"


class SubmissionValidatorTests(unittest.TestCase):
    def test_submission_validator_accepts_current_tree(self):
        result = subprocess.run(
            [sys.executable, str(VALIDATOR)],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("submission docs validated", result.stdout)
        self.assertIn("README.md", result.stdout)
        self.assertIn("idl/admin-authority.idl.json", result.stdout)
        self.assertIn(".github/workflows/ci.yml", result.stdout)

    def test_docs_do_not_retain_pre_spike_pending_language(self):
        forbidden_phrases = {
            "README.md": [
                "remaining submission work is the bounded SPEL/IDL + LEZ local-sequencer integration gate",
                "optional final host re-run after the semantic guest port",
                "Remaining (human): re-record the narrated video against the corrected lifecycle, then resubmit.",
            ],
            "SUBMISSION.md": [
                "The real SPEL/LEZ proof remains the final integration gate",
                "pending chain/toolchain proof",
                "Optional: re-run the host LEZ lifecycle against the semantic guest release candidate",
            ],
            "docs/SPEL_STATUS.md": [
                "Remaining steps to record a full LEZ proof",
                "This work is staged for the Evi sign-off gate before any push or PR.",
                "rerun `spel-spike/live_lifecycle.rs` on the host once after Evi sign-off",
                "The one remaining task before the public PR is a **narrated demo video**",
                "is the canonical IDL shipped with the submission",
            ],
            "docs/SPEC_COMPLIANCE.md": [
                "final host proof docs pending after real run",
                "SPEL/RFP-001 integration evidence is not complete yet",
                "Requires LEZ local sequencer/devnet measurement path",
                "Deployed/tested on LEZ devnet/testnet: not complete",
                "Final `RISC0_DEV_MODE=0` local sequencer demo still required",
                "optional final rerun recommended after semantic guest port",
            ],
        }

        for relative, phrases in forbidden_phrases.items():
            text = (ROOT / relative).read_text()
            for phrase in phrases:
                self.assertNotIn(phrase, text, f"{relative} still contains stale phrase: {phrase}")

    def test_idl_claims_match_generated_artifact_limitations(self):
        import json

        spec = (ROOT / "docs" / "SPEC_COMPLIANCE.md").read_text()
        rc1_idl_text = (ROOT / "idl" / "admin-authority.idl.spel-generated.json").read_text()
        rc3_idl_text = (
            ROOT / "idl" / "admin-authority.idl.spel-generated.rc3-testnet.json"
        ).read_text()

        # Under the corrected guest (which annotates #[account_type]) the two
        # pin-labeled generations coincide byte-for-byte — a cross-revision
        # stability check, not a richer-vs-poorer asymmetry.
        self.assertEqual(rc1_idl_text, rc3_idl_text)

        # Both carry the corrected 4-instruction surface and the account bodies.
        for idl_text in (rc1_idl_text, rc3_idl_text):
            idl = json.loads(idl_text)
            names = [i["name"] for i in idl["instructions"]]
            self.assertEqual(
                names, ["create_mint", "create_holding", "mint_to", "set_mint_authority"]
            )
            account_names = [a["name"] for a in idl.get("accounts", [])]
            for account in ("AuthorityInfo", "MintDefinition", "TokenHolding"):
                self.assertIn(account, account_names)

        # The compliance doc must describe the artifacts honestly: the two
        # generations coincide, and the spel-generated IDL is the authoritative
        # on-chain surface (the hand-written one is a design reference).
        self.assertIn("byte-identical across the rc1", spec)
        self.assertIn("spel-generated IDL is authoritative for the on-chain", spec)

    def test_archival_proof_validator_tracks_all_live_tx_hashes(self):
        validator = VALIDATOR.read_text()
        for tx_hash in [
            # archival structural-surface spike (2026-05-17)
            "2a5162350724273a09ecfdb32026fc3c7b48b66ae78e441bd602e2d6b72a8965",
            "fd68e225ceb3164f88367600564a026dbfb8f4823f449a6b07c37fc35de79c69",
            "07de7c91b5137fdb88b1f0ad84bb3b30a436cf9e8e368193fc81998713d88811",
            "ec58ace48bbadee7143585b7bc402b33dd5fd767b8dd15dcf13ce1a87eba204d",
            "e1ecbb81da1a828a7068ef05401c96ed7593d29c8fa9537c07bda1dea020a3f3",
        ]:
            self.assertIn(tx_hash, validator)

    def test_semantic_rerun_validator_tracks_all_release_candidate_hashes(self):
        validator = VALIDATOR.read_text()
        for tx_hash in [
            # semantic release-candidate rerun (2026-05-18)
            "b16831c0ee550014ea9297ba47d47b31d0c1b425ff3219b44358189bb9204ab5",
            "7d582e7b8dfd166b96f2e3b6c2b52b0febbb42032be198b45c984f1e8b6f9d63",
            "c474cf82465fefed6e8e45ae22c4d6060d05d2a4610f37f04d033dfad5d3c74f",
            "756ee393ed7e4957fd73ec89ffe93dd5fc342535f028edf45f21ca755ee7351c",
            "27df9483e9b74d3860ced99cb596739be73f6e7c5d0a34f47798acfb08bc2bff",
            "58470667b5d45fcc4317684eb7aaad2b19c0cf666bd8c7f85d2b0e1069d0b960",
        ]:
            self.assertIn(tx_hash, validator)

    def test_public_testnet_validator_tracks_all_testnet_hashes(self):
        validator = VALIDATOR.read_text()
        for token in [
            # public testnet deploy + lifecycle (2026-06-03) — superseded pre-fix
            # run, retained as a historical record
            "https://testnet.lez.logos.co/",
            "59e15341b10dfacf6bfeb8436f587e18fb4bf714fc042c79aba9f8878fb0ae2c",  # ImageID
            "07561014a617dc18c3a420db01c9f752755053eb58f44d8db98871646cb968ba",  # deploy
            "17d90ea633db426a863efc697239aa158293c20822ff07839a2a0b6f2eeb37d2",  # create_mint
            "be393bcf82e489bc5a940904ed0e38ea861b61939f43529132ca4c701f29bbd8",  # mint_to(100)
            "0540648f9f5099296340bcf65d0ac1a4cf89ff226eca7abb27dcdcb0b29f5784",  # set_mint_authority(None)
            "312ea9f120602f9aa2d574d43fefa73ae25d74e1bd228b9f65317fef8fef4798",  # post-revoke (rejected)
        ]:
            self.assertIn(token, validator)

    def test_corrected_guest_validator_tracks_all_testnet_hashes(self):
        validator = VALIDATOR.read_text()
        for token in [
            # public testnet deploy + lifecycle (2026-06-04) — CORRECTED guest
            # (load-bearing on-chain evidence of the fix)
            "32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce",  # ImageID == ProgramId
            "4NxnuVrQBiwq2dCwZ3g3EnaD8JXGgBwEf6CR2a8L9JXF",                       # program (base58)
            "HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu",                       # mint PDA
            "5b39deec38e49bb1bedf1956e5d7429ec20e3c009f0ccfe7a4fc449685cb4ce0",  # deploy
            "7d1dcb04b5f339b33f04a120b7334cf9802720d4a917e600becd62476e44da74",  # create_mint
            "520d080b833c7e4038a1aa214bba43a3fc97328e8f379a093b74ca3e32be5893",  # create_holding
            "8c865d0184f55ce5a881e24c8c125cd3729c5f90a4b83d0484c8d1610f743f61",  # mint_to(60)
            "c63168b7f615221ab2425b2ba003d32183f4df2e482eb4203e4e216675993d21",  # mint_to(40)
            "8c4b08b5c750c57d0dbb4e9f43c32b7c0f2627ce5508da85408e3aaf01f5a331",  # set_mint_authority(None)
            "6e92e605e932756332c9721a4e4754f155780069490b256fe67b35f374a972d1",  # post-revoke (rejected)
        ]:
            self.assertIn(token, validator)

    def test_spel_guest_comments_do_not_claim_unimplemented_semantics(self):
        guest = (ROOT / "spel-spike" / "admin_authority_guest.rs").read_text()
        for phrase in [
            "signer mismatches, the transaction fails",
            "The signer must match the current authority",
            "Revocation is one-way",
        ]:
            self.assertNotIn(phrase, guest)
        self.assertIn("fn require_authority", guest)
        self.assertIn("checked_add(amount)", guest)
        self.assertIn("authority has been revoked", guest)


if __name__ == "__main__":
    unittest.main()
