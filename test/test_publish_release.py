"""Offline publication regressions: no credentials or GitHub mutations."""

import copy
import base64
import hashlib
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
        {"name": url.rsplit("/", 1)[1], "browser_download_url": url, "size": 100, "state": "uploaded"}
        for url in dict.fromkeys(entry["url"] for entry in MANIFEST["platforms"].values())
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
        if args == ("release", "view", TAG, "--repo", REPO, "--json", "apiUrl,tagName"):
            return json.dumps({"tagName": TAG, "apiUrl": f"https://api.github.com/repos/{REPO}/releases/123"})
        if args == ("api", f"https://api.github.com/repos/{REPO}/releases/123"):
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
        self.crypto = patch.object(publisher, "verify_artifacts").start()
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

    def test_draft_resolution_does_not_use_public_tag_lookup(self):
        self.assertEqual(self.run_main("stable"), 0)
        self.gh.assert_any_call("release", "view", TAG, "--repo", REPO, "--json", "apiUrl,tagName")
        self.assertNotIn(unittest.mock.call("api", f"repos/{REPO}/releases/tags/{TAG}"), self.gh.call_args_list)

    def test_draft_untagged_urls_match_final_manifest_urls(self):
        for asset in self.api.release["assets"]:
            asset["browser_download_url"] = asset["browser_download_url"].replace(TAG, "untagged-25955b20eb61985d71ae")
        self.assertEqual(self.run_main("stable"), 0)
        self.assertEqual(self.api.latest, TAG)

    def test_public_release_cannot_use_draft_urls(self):
        self.api.release["draft"] = False
        self.api.release["assets"][0]["browser_download_url"] = f"{BASE}/download/untagged-abcd/Desktop.app.tar.gz"
        self.assertEqual(self.run_main("stable"), 1)
        self.assertEqual(self.api.edits, [])

    def test_draft_urls_still_require_matching_repo_tag_and_filename(self):
        for url in (f"{BASE}/download/v0.1.5/Desktop.app.tar.gz",
                    f"{BASE}/download/untagged-abcd/wrong.app.tar.gz",
                    "https://github.com/other/repo/releases/download/untagged-abcd/Desktop.app.tar.gz"):
            with self.subTest(url=url):
                self.api.release["assets"][0]["browser_download_url"] = url
                self.assertEqual(self.run_main("stable"), 1)
                self.assertEqual(self.api.edits, [])

    def test_duplicate_asset_names_are_rejected(self):
        self.api.release["assets"].append(copy.deepcopy(self.api.release["assets"][0]))
        self.assertEqual(self.run_main("stable"), 1)
        self.assertEqual(self.api.edits, [])

    def test_release_lookup_rejects_wrong_identity_before_edit(self):
        for view in ({"tagName": "v9.0.0", "apiUrl": f"https://api.github.com/repos/{REPO}/releases/123"},
                     {"tagName": TAG, "apiUrl": "https://example.invalid/releases/123"}):
            with self.subTest(view=view):
                self.gh.side_effect = lambda *args: json.dumps(view)
                self.assertEqual(self.run_main(), 1)
                self.assertEqual(self.api.edits, [])

    def test_edit_failure_is_nonzero_and_does_not_restore_or_report_success(self):
        self.api.fail_edit = TAG
        self.assertEqual(self.run_main(), 17)
        self.assertEqual(len(self.api.edits), 1)
        self.verify.assert_not_called()
        self.assertNotIn("Verified public release", sys.stdout.getvalue())

    def test_invalid_signature_blocks_all_publication(self):
        self.crypto.side_effect = subprocess.CalledProcessError(1, ["minisign"], stderr="Signature verification failed")
        self.assertEqual(self.run_main("stable"), 1)
        self.assertEqual(self.api.edits, [])
        self.verify.assert_not_called()

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
                if args[:2] == ['release', 'edit'] or (args[0] == 'api' and '/contents/' in args[1]):
                    print('simulated publish failure', file=sys.stderr)
                    sys.exit(17)
                elif args[:2] == ['release', 'download']:
                    print(json.dumps({MANIFEST!r}))
                elif args[:2] == ['release', 'view']:
                    print(json.dumps({{'tagName': {TAG!r}, 'apiUrl': 'https://api.github.com/repos/{REPO}/releases/123'}}))
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


class ArtifactSignatureTests(unittest.TestCase):
    """Real minisign verification, with local ephemeral test keys and fake downloads."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.payload = b"signed updater fixture for release verification"
        self.public = self.root / "test.pub"
        self.secret = self.root / "test.key"
        subprocess.run(["minisign", "-G", "-W", "-p", str(self.public), "-s", str(self.secret)],
                       check=True, capture_output=True)
        artifact = self.root / "artifact"
        artifact.write_bytes(self.payload)
        subprocess.run(["minisign", "-S", "-m", str(artifact), "-s", str(self.secret)],
                       check=True, capture_output=True)
        signature = base64.b64encode((self.root / "artifact.minisig").read_bytes()).decode()
        self.manifest = copy.deepcopy(MANIFEST)
        for entry in self.manifest["platforms"].values():
            entry["signature"] = signature
        self.release = copy.deepcopy(RELEASE)
        for asset in self.release["assets"]:
            asset.update(name=asset["browser_download_url"].rsplit("/", 1)[1],
                         size=len(self.payload), digest="sha256:" + hashlib.sha256(self.payload).hexdigest())
        self.config = {"version": TAG[1:], "plugins": {"updater": {
            "pubkey": base64.b64encode(self.public.read_bytes()).decode()}}}
        self.real_run = subprocess.run
        self.downloaded = []

    def run_verify(self):
        def run(args, **kwargs):
            if args[0] != "gh":
                return self.real_run(args, **kwargs)
            self.downloaded.append(args)
            Path(args[args.index("--output") + 1]).write_bytes(self.payload)
            return subprocess.CompletedProcess(args, 0, "", "")
        response = json.dumps({"content": base64.b64encode(json.dumps(self.config).encode()).decode()})
        with patch.object(publisher, "gh", return_value=response), \
                patch.object(publisher.subprocess, "run", side_effect=run), \
                patch("sys.stdout", new_callable=io.StringIO):
            publisher.verify_artifacts(REPO, TAG, self.manifest, self.release)

    def test_all_real_signatures_verified_and_duplicate_assets_downloaded_once(self):
        self.run_verify()
        self.assertEqual(len(self.downloaded), 3)

    def test_draft_asset_bytes_verified_with_original_manifest_signatures(self):
        for asset in self.release["assets"]:
            asset["browser_download_url"] = asset["browser_download_url"].replace(TAG, "untagged-25955b20eb61985d71ae")
        self.run_verify()
        self.assertEqual(len(self.downloaded), 3)

    def test_same_size_tampering_rejected_even_without_server_digest(self):
        self.payload = b"X" + self.payload[1:]
        for asset in self.release["assets"]:
            asset.pop("digest")
        with self.assertRaises(subprocess.CalledProcessError):
            self.run_verify()

    def test_server_digest_mismatch_rejected(self):
        self.release["assets"][0]["digest"] = "sha256:" + "0" * 64
        with self.assertRaisesRegex(publisher.PublishError, "digest mismatch"):
            self.run_verify()

    def test_wrong_tagged_version_rejected_before_download(self):
        self.config["version"] = "0.1.5"
        with self.assertRaisesRegex(publisher.PublishError, "Tagged app version"):
            self.run_verify()
        self.assertEqual(self.downloaded, [])

    def test_truncated_download_rejected(self):
        self.payload = self.payload[:-1]
        with self.assertRaisesRegex(publisher.PublishError, "size mismatch"):
            self.run_verify()

    def test_signature_for_another_key_rejected(self):
        other_public, other_secret = self.root / "other.pub", self.root / "other.key"
        subprocess.run(["minisign", "-G", "-W", "-p", str(other_public), "-s", str(other_secret)],
                       check=True, capture_output=True)
        self.config["plugins"]["updater"]["pubkey"] = base64.b64encode(other_public.read_bytes()).decode()
        with self.assertRaises(subprocess.CalledProcessError):
            self.run_verify()


if __name__ == "__main__":
    unittest.main()
