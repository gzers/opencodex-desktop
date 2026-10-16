"""Offline installer admission checks; no installer or application is executed."""
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "retention_admit", Path(__file__).parent / "smoke/windows-retention-ci-admit.py")
admit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(admit)


class InstallerAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.run = {
            "id": admit.CANDIDATE_RUN, "head_sha": admit.CANDIDATE_SOURCE,
            "status": "completed", "conclusion": "success", "event": "workflow_dispatch",
            "run_attempt": 1, "repository": {"full_name": admit.REPOSITORY},
        }
        self.artifact = {
            "id": admit.CANDIDATE_ARTIFACT, "expired": False, "size_in_bytes": 7626479,
            "digest": "sha256:" + admit.CANDIDATE_ZIP_SHA,
            "workflow_run": {"id": admit.CANDIDATE_RUN, "head_sha": admit.CANDIDATE_SOURCE},
        }
        self.release = {
            "id": admit.BASELINE_RELEASE, "tag_name": "v0.1.9", "draft": False, "prerelease": False,
            "assets": [{"id": row["id"]} for row in admit.BASELINE.values()],
        }

    def asset(self, kind):
        row = admit.BASELINE[kind]
        return {
            "id": row["id"], "name": row["name"], "size": row["bytes"], "digest": "sha256:" + row["sha256"],
            "browser_download_url": "https://github.com/" + admit.REPOSITORY +
                                    "/releases/download/v0.1.9/" + row["name"],
        }

    def test_fixed_candidate_and_both_official_baselines(self):
        admit.verify_candidate(self.run, self.artifact)
        for kind in admit.BASELINE:
            admit.verify_baseline(self.release, self.asset(kind), kind)

    def test_current_harness_sha_cannot_replace_candidate_identity(self):
        for key, value in (("id", 1), ("head_sha", "f" * 40), ("run_attempt", 2),
                           ("status", "in_progress"), ("conclusion", "failure"),
                           ("event", "push"), ("repository", {"full_name": "other/repo"})):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_candidate({**self.run, key: value}, self.artifact)

    def test_artifact_source_size_digest_and_expiry_are_independent(self):
        for key, value in (("id", 1), ("expired", True), ("size_in_bytes", 0),
                           ("digest", "sha256:" + "0" * 64),
                           ("workflow_run", {"id": 1, "head_sha": "f" * 40})):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_candidate(self.run, {**self.artifact, key: value})

    def test_baseline_release_tag_attachment_and_publication_must_match(self):
        for key, value in (("id", 1), ("tag_name", "latest"), ("draft", True),
                           ("prerelease", True), ("assets", [])):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_baseline({**self.release, key: value}, self.asset("msi"), "msi")

    def test_baseline_asset_and_download_source_cannot_drift(self):
        for kind in admit.BASELINE:
            for key, value in (("id", 1), ("name", "other.exe"), ("size", 0),
                               ("digest", None), ("browser_download_url", "https://other.example/package")):
                with self.subTest(kind=kind, key=key), self.assertRaises(ValueError):
                    admit.verify_baseline(self.release, {**self.asset(kind), key: value}, kind)

    def test_msi_and_nsis_are_not_interchangeable(self):
        with self.assertRaises(ValueError):
            admit.verify_baseline(self.release, self.asset("msi"), "nsis")
        with self.assertRaises(ValueError):
            admit.verify_baseline(self.release, self.asset("msi"), "arbitrary")


if __name__ == "__main__":
    unittest.main()
