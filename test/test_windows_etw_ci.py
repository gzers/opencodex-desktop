"""Offline admission regressions; no downloads, binaries, sessions, or credentials."""
import copy
import importlib.util
import io
from pathlib import Path
import stat
import unittest
import urllib.request
import zipfile

SPEC = importlib.util.spec_from_file_location(
    "windows_etw_ci_admit", Path(__file__).parent / "smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(admit)


class AdmissionTests(unittest.TestCase):
    def setUp(self):
        self.sha = "e" * 40
        self.ci = {
            "head_sha": self.sha, "event": "push", "status": "completed", "conclusion": "success",
            "path": ".github/workflows/ci.yml", "repository": {"full_name": admit.REPOSITORY},
        }
        self.jobs = {"total_count": 5, "jobs": [
            {"name": name, "status": "completed", "conclusion": "success"}
            for name in admit.REQUIRED_JOBS
        ]}
        self.original = {
            "id": admit.ORIGINAL_RUN, "head_sha": admit.BD2, "event": "pull_request",
            "status": "completed", "conclusion": "failure", "repository": {"full_name": admit.REPOSITORY},
        }
        self.artifact = {
            "id": admit.ARTIFACT_ID, "expired": False, "size_in_bytes": 8810232,
            "digest": "sha256:" + admit.ARTIFACT_SHA,
            "workflow_run": {"id": admit.ORIGINAL_RUN, "head_sha": admit.BD2},
        }

    def test_exact_ci_and_original_failure_are_admitted_independently(self):
        admit.verify_ci(self.ci, self.jobs, self.sha)
        admit.verify_original(self.original, self.artifact)

    def test_other_commit_pr_success_and_unfinished_run_cannot_admit_recording(self):
        for key, bad in (("head_sha", "a" * 40), ("event", "pull_request"),
                         ("status", "in_progress"), ("conclusion", "failure"),
                         ("path", ".github/workflows/release.yml")):
            with self.subTest(key=key):
                modified = {**self.ci, key: bad}
                with self.assertRaises(ValueError):
                    admit.verify_ci(modified, self.jobs, self.sha)

    def test_missing_failed_duplicate_or_truncated_job_is_rejected(self):
        for mutation in ("missing", "failed", "duplicate", "truncated"):
            jobs = copy.deepcopy(self.jobs)
            if mutation == "missing":
                jobs["jobs"].pop()
                jobs["total_count"] -= 1
            elif mutation == "failed":
                jobs["jobs"][0]["conclusion"] = "failure"
            elif mutation == "duplicate":
                jobs["jobs"].append(jobs["jobs"][0])
                jobs["total_count"] += 1
            else:
                jobs["total_count"] += 1
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                admit.verify_ci(self.ci, jobs, self.sha)

    def test_original_failure_cannot_be_replaced_with_success(self):
        with self.assertRaises(ValueError):
            admit.verify_original({**self.original, "conclusion": "success"}, self.artifact)

    def test_foreign_expired_wrong_digest_size_and_run_artifact_are_rejected(self):
        for key, value in (
            ("id", 1), ("expired", True), ("digest", "sha256:" + "0" * 64),
            ("size_in_bytes", 0), ("workflow_run", {"id": 1, "head_sha": admit.BD2}),
        ):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_original(self.original, {**self.artifact, key: value})
        with self.assertRaises(ValueError):
            admit.verify_ci({**self.ci, "repository": {"full_name": "foreign/repo"}}, self.jobs, self.sha)

    def test_safe_archive_and_crc_are_checked(self):
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            archive.writestr("windows-backend/fixture.bin", b"original")
        with zipfile.ZipFile(stream) as archive:
            admit.checked_zip(archive)

    def test_archive_traversal_windows_drive_backslash_and_symlink_are_rejected(self):
        for name in ("../outside", "/outside", "C:/outside", "safe\\outside", "symlink"):
            stream = io.BytesIO()
            with zipfile.ZipFile(stream, "w") as archive:
                info = zipfile.ZipInfo(name)
                # ZipInfo normalizes os.sep in its constructor on Windows.
                # Keep the malicious on-disk name, which the reader exposes
                # through orig_filename even after normalizing filename.
                info.filename = name
                if name == "symlink":
                    info.create_system = 3
                    info.external_attr = (stat.S_IFLNK | 0o777) << 16
                archive.writestr(info, b"target")
            with self.subTest(name=name), zipfile.ZipFile(stream) as archive, self.assertRaises(ValueError):
                admit.checked_zip(archive)

    def test_case_colliding_archive_entries_are_rejected(self):
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            archive.writestr("fixture.exe", b"one")
            archive.writestr("FIXTURE.exe", b"two")
        with zipfile.ZipFile(stream) as archive, self.assertRaises(ValueError):
            admit.checked_zip(archive)

    def test_redirect_never_forwards_api_authorization_to_signed_download_host(self):
        request = urllib.request.Request("https://api.github.com/repos/owner/repo/actions/artifacts/1/zip",
                                         headers={"Authorization": "Bearer sentinel"})
        request.add_unredirected_header("AUTHORIZATION", "Bearer another-sentinel")
        redirected = admit.SafeRedirect().redirect_request(
            request, None, 302, "Found", {}, "https://example.blob.core.windows.net/signed?token=opaque")
        self.assertFalse(any(name.lower() == "authorization" for name in redirected.headers))
        self.assertFalse(any(name.lower() == "authorization" for name in redirected.unredirected_hdrs))

    def test_non_https_or_embedded_credential_redirect_is_rejected(self):
        request = urllib.request.Request("https://api.github.com/original")
        for url in ("http://example.com/file", "https://user:secret@example.com/file"):
            with self.subTest(url=url), self.assertRaises(ValueError):
                admit.SafeRedirect().redirect_request(request, None, 302, "Found", {}, url)


if __name__ == "__main__":
    unittest.main()
