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
        spec = (ROOT / "docs" / "SPEC_COMPLIANCE.md").read_text()
        generated_idl = (ROOT / "idl" / "admin-authority.idl.spel-generated.json").read_text()

        self.assertIn('"accounts": []', generated_idl)
        self.assertNotIn("The two artifacts agree on the instruction set (`create_mint`, `mint_to`, `set_mint_authority`) and account types", spec)
        self.assertIn("The two artifacts agree on the instruction set", spec)
        self.assertIn("the current SPEL-generated artifact does not emit account/type bodies", spec)

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
