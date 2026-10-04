"""Offline publication regressions: no credentials or GitHub mutations."""

import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest
from unittest.mock import patch
import urllib.error


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("publish_release", ROOT / ".github/scripts/publish_release.py")
publisher = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publisher)
REPO = "example/desktop"
TAG = "v0.1.6"
BASE = f"https://github.com/{REPO}/releases"
MANIFEST = {
    "version": TAG[1:],
    "platforms": {
        platform: {"url": f"{BASE}/download/{TAG}/{name}", "signature": "fixture-signature"}
        for platform, name in (
            ("darwin-aarch64", "Desktop.app.tar.gz"),
            ("windows-x86_64", "Desktop.msi"),
            ("windows-x86_64-msi", "Desktop.msi"),
            ("windows-x86_64-nsis", "Desktop.exe"),
        )
    },
}
RELEASE = {
    "tag_name": TAG, "draft": True, "prerelease": True,
    "assets": [
        {"browser_download_url": entry["url"], "size": 100, "state": "uploaded"}
        for entry in MANIFEST["platforms"].values()
    ],
}


class FakeGitHub:
    def __init__(self, before="v0.1.3", fail_edit=None):
        self.release = copy.deepcopy(RELEASE)
        self.manifest = copy.deepcopy(MANIFEST)
        self.latest = before
        self.fail_edit = fail_edit
        self.edits = []
        self.ignore_latest_edit = False

    def __call__(self, *args):
        if args == ("api", f"repos/{REPO}/releases/latest"):
            if self.latest is None:
                raise subprocess.CalledProcessError(1, args, stderr="gh: Not Found (HTTP 404)")
            return json.dumps({"tag_name": self.latest})
        if args == ("api", f"repos/{REPO}/releases/tags/{TAG}"):
            return json.dumps(self.release)
        if args == ("release", "download", TAG, "--repo", REPO, "--pattern", "latest.json", "--output", "-"):
            return json.dumps(self.manifest)
        if args[:2] == ("release", "edit"):
            self.edits.append(args)
            if args[2] == self.fail_edit:
                raise subprocess.CalledProcessError(17, args, stderr="release edit rejected")
            if args[2] == TAG:
                self.release.update(draft=False, prerelease=False)
            if "--latest=true" in args and not self.ignore_latest_edit:
                self.latest = args[2]
            return ""
        raise AssertionError(f"Unexpected gh invocation: {args!r}")


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.api = FakeGitHub()
        self.addCleanup(patch.stopall)
        self.gh = patch.object(publisher, "gh", side_effect=self.api).start()
        self.verify = patch.object(publisher, "verify_public_manifest").start()
        patch("sys.stdout", new_callable=io.StringIO).start()
        patch("sys.stderr", new_callable=io.StringIO).start()

    def run_main(self, mode="preserve-latest"):
        with patch.object(sys, "argv", ["publish_release.py", TAG, "--repo", REPO, "--mode", mode]):
            return publisher.main()

    def test_preserves_existing_latest(self):
        self.assertEqual(self.run_main(), 0)
        self.assertEqual(self.api.latest, "v0.1.3")
        self.assertIn("--latest=false", self.api.edits[0])
        self.assertEqual(self.api.edits[1][2], "v0.1.3")
        self.verify.assert_called_once_with(f"{BASE}/download/{TAG}/latest.json", MANIFEST)

    def test_already_latest_is_idempotent_and_verifies_stable_alias(self):
        self.api.latest = TAG
        self.assertEqual(self.run_main(), 0)
        self.assertEqual(len(self.api.edits), 1)
        self.assertIn("--latest=true", self.api.edits[0])
        self.verify.assert_any_call(f"{BASE}/latest/download/latest.json", MANIFEST)

    def test_stable_promotes_target_and_verifies_both_endpoints(self):
        self.assertEqual(self.run_main("stable"), 0)
        self.assertEqual(self.api.latest, TAG)
        self.assertEqual(len(self.api.edits), 1)
        self.assertEqual(self.verify.call_count, 2)
        self.verify.assert_any_call(f"{BASE}/download/{TAG}/latest.json", MANIFEST)
        self.verify.assert_any_call(f"{BASE}/latest/download/latest.json", MANIFEST)

    def test_edit_failure_is_nonzero_and_does_not_restore_or_report_success(self):
        self.api.fail_edit = TAG
        self.assertEqual(self.run_main(), 17)
        self.assertEqual(len(self.api.edits), 1)
        self.verify.assert_not_called()
        self.assertNotIn("Verified public release", sys.stdout.getvalue())

    def test_restore_latest_failure_is_nonzero(self):
        self.api.fail_edit = "v0.1.3"
        self.assertEqual(self.run_main(), 17)
        self.assertEqual(len(self.api.edits), 2)
        self.verify.assert_not_called()

    def test_latest_mismatch_is_not_success(self):
        self.api.ignore_latest_edit = True
        self.assertEqual(self.run_main("stable"), 1)
        self.verify.assert_not_called()

    def test_anonymous_verification_failure_is_nonzero(self):
        self.verify.side_effect = publisher.PublishError("anonymous manifest unreachable")
        self.assertEqual(self.run_main("stable"), 1)
        self.assertNotIn("Verified public release", sys.stdout.getvalue())

    def test_invalid_manifests_fail_before_any_edit(self):
        variants = [None, [], {"version": TAG[1:]}, copy.deepcopy(MANIFEST)]
        variants[-1]["version"] = "0.1.5"
        for field, value in (("signature", ""), ("signature", None), ("url", "http://invalid/package"),
                             ("url", f"{BASE}/download/v0.1.5/Desktop.msi")):
            manifest = copy.deepcopy(MANIFEST)
            manifest["platforms"]["windows-x86_64"][field] = value
            variants.append(manifest)
        manifest = copy.deepcopy(MANIFEST)
        del manifest["platforms"]["windows-x86_64-nsis"]
        variants.append(manifest)
        for manifest in variants:
            with self.subTest(manifest=manifest):
                self.api.manifest = manifest
                self.assertEqual(self.run_main(), 1)
                self.assertEqual(self.api.edits, [])

    def test_missing_empty_or_unfinished_asset_fails_before_edit(self):
        for key, value in (("size", 0), ("state", "new"), ("browser_download_url", "https://example.invalid/other")):
            with self.subTest(key=key):
                self.api.release = copy.deepcopy(RELEASE)
                self.api.release["assets"][0][key] = value
                self.assertEqual(self.run_main(), 1)
                self.assertEqual(self.api.edits, [])

    def test_stable_downgrade_uses_numeric_version_order(self):
        self.api.latest = "v0.1.10"
        self.assertEqual(self.run_main("stable"), 1)
        self.assertEqual(self.api.edits, [])

    def test_first_release_requires_explicit_stable_mode(self):
        self.api.latest = None
        self.assertEqual(self.run_main(), 1)
        self.assertEqual(self.api.edits, [])
        self.assertEqual(self.run_main("stable"), 0)
        self.assertEqual(self.api.latest, TAG)

    def test_auth_error_does_not_become_missing_latest(self):
        def unauthorized_latest(*args):
            if args == ("api", f"repos/{REPO}/releases/latest"):
                raise subprocess.CalledProcessError(4, args, stderr="gh: Requires authentication (HTTP 401)")
            return self.api(*args)
        self.gh.side_effect = unauthorized_latest
        self.assertEqual(self.run_main("stable"), 4)
        self.assertEqual(self.api.edits, [])


class PublicEndpointTests(unittest.TestCase):
    def test_anonymous_verification_retries_until_expected_manifest(self):
        responses = [io.BytesIO(b'{"version":"0.1.3"}'), io.BytesIO(json.dumps(MANIFEST).encode())]
        with patch.object(publisher.urllib.request, "urlopen", side_effect=responses) as get, \
                patch.object(publisher.time, "sleep") as sleep:
            publisher.verify_public_manifest(f"{BASE}/latest/download/latest.json", MANIFEST)
            self.assertEqual(get.call_count, 2)
            sleep.assert_called_once_with(3)
            for call in get.call_args_list:
                self.assertFalse(call.args[0].has_header("Authorization"))

    def test_persistent_404_or_wrong_content_fails(self):
        for effect in (
            urllib.error.HTTPError(BASE, 404, "not found", {}, None),
            lambda *a, **k: io.BytesIO(b'{"version":"0.1.3"}'),
        ):
            with self.subTest(effect=effect), \
                    patch.object(publisher.urllib.request, "urlopen", side_effect=effect) as get, \
                    patch.object(publisher.time, "sleep"):
                with self.assertRaises(publisher.PublishError):
                    publisher.verify_public_manifest(f"{BASE}/latest/download/latest.json", MANIFEST)
                self.assertEqual(get.call_count, 5)


class WorkflowExitTests(unittest.TestCase):
    def test_real_workflow_shell_propagates_script_failure_through_tee(self):
        # Execute the actual publish run block, but all gh calls hit a local stub.
        # The diagnostic-publication block is never executed.
        workflow = (ROOT / ".github/workflows/publish-release.yml").read_text()
        block = workflow.split("        run: |\n", 1)[1].split("\n\n", 1)[0]
        shell = textwrap.dedent(block)
        with tempfile.TemporaryDirectory() as directory:
            temp = Path(directory)
            stub = temp / "gh"
            stub.write_text(f"#!{sys.executable}\n" + textwrap.dedent(f"""\
                import json, sys
                args = sys.argv[1:]
                if args[:2] == ['release', 'edit']:
                    print('simulated publish failure', file=sys.stderr)
                    sys.exit(17)
                elif args[:2] == ['release', 'download']:
                    print(json.dumps({MANIFEST!r}))
                elif args[0] == 'api' and args[1].endswith('/latest'):
                    print(json.dumps({{'tag_name': 'v0.1.3'}}))
                elif args[0] == 'api':
                    print(json.dumps({RELEASE!r}))
                else:
                    sys.exit(99)
            """))
            stub.chmod(0o755)
            shell = shell.replace("/tmp/publish-diag.txt", str(temp / "diagnostic.txt"))
            env = {**os.environ, "PATH": f"{temp}{os.pathsep}{os.environ['PATH']}",
                   "INPUT_TAG": TAG, "PUBLISH_MODE": "preserve-latest", "GITHUB_REPOSITORY": REPO}
            result = subprocess.run(["bash", "-c", shell], cwd=ROOT, env=env, text=True, capture_output=True)
            self.assertEqual(result.returncode, 17, result.stdout + result.stderr)
            self.assertIn("simulated publish failure", (temp / "diagnostic.txt").read_text())
            self.assertNotIn("Verified public release", result.stdout)


if __name__ == "__main__":
    unittest.main()
