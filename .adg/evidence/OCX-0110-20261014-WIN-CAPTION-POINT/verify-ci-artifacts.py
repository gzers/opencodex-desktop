"""Verify exact-SHA CI evidence archives without executing their contents."""
import hashlib
import json
import os
import subprocess
import urllib.parse
import urllib.request
import urllib.error
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows-caption-point-5a36cda-20261014"
source = "5a36cda04018fb367c2f77e3a2a1bdc22d991e03"
artifacts = json.loads((root / "github-artifacts-38431616359.json").read_text())["artifacts"]
output = root / "ci-artifact-verification.json"
assert not output.exists()
assert len(artifacts) == 2


class SafeRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, url):
        assert urllib.parse.urlsplit(url).scheme == "https"
        redirected = super().redirect_request(request, fp, code, msg, headers, url)
        if urllib.parse.urlsplit(request.full_url).netloc != urllib.parse.urlsplit(url).netloc:
            redirected.remove_header("Authorization")
        return redirected


opener = urllib.request.build_opener(SafeRedirect)
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
observations = []
attempts = [
    {"artifactId": 11831361283, "attempt": "original", "result": "archive-saved"},
    {"artifactId": 11831009311, "attempt": "original", "result": "failed-before-second-archive-created",
     "reason": "TLS unexpected EOF; verification not disabled"},
    {"artifactId": 11831361283, "attempt": "second-invocation",
     "result": "existing-output-guard-stopped-before-network",
     "correction": "First archive existed; the original TLS failure was on the second archive"},
]
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n", capture_output=True, text=True,
        check=True, timeout=20, cwd=code)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
    for artifact in artifacts:
        assert artifact["workflow_run"]["head_sha"] == source and not artifact["expired"]
        expected = artifact["digest"].removeprefix("sha256:")
        destination = runtime / f"artifact-{artifact['id']}.zip"
        request = urllib.request.Request(
            f"https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/{artifact['id']}/zip",
            headers={"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
                     "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "ocx-ci-readonly-verifier"})
        if destination.exists():
            length = destination.stat().st_size
            actual = hashlib.sha256(destination.read_bytes()).hexdigest()
            attempts.append({"artifactId": artifact["id"], "attempt": "recovery",
                             "result": "existing-archive-read-only-rehash"})
        else:
            digest = hashlib.sha256()
            length = 0
            try:
                response = opener.open(request, timeout=30)
                attempts.append({"artifactId": artifact["id"], "attempt": "recovery",
                                 "result": "HTTPS-response-opened"})
            except urllib.error.URLError as error:
                attempts.append({"artifactId": artifact["id"], "attempt": "recovery",
                                 "result": "failed", "reasonType": type(error.reason).__name__})
                raise RuntimeError("Artifact HTTPS connection failed; no TLS or authority bypass") from None
            with response, destination.open("xb") as stream:
                while block := response.read(1024 * 1024):
                    length += len(block)
                    assert length < 32 * 1024 * 1024
                    digest.update(block)
                    stream.write(block)
            actual = digest.hexdigest()
        assert actual == expected and length == artifact["size_in_bytes"]
        with zipfile.ZipFile(destination) as archive:
            entries = []
            for item in archive.infolist():
                path = PurePosixPath(item.filename.replace("\\", "/"))
                assert not path.is_absolute() and ".." not in path.parts
                entries.append({"path": item.filename, "bytes": item.file_size, "crc32": item.CRC})
            bad = archive.testzip()
            assert bad is None
        observations.append({
            "id": artifact["id"], "name": artifact["name"], "source": source,
            "archiveBytes": length, "expectedSha256": expected, "actualSha256": actual,
            "digestMatches": True, "zipCrcVerified": True, "entries": entries,
            "downloadedToIgnoredRuntime": str(destination), "archiveContentsExecuted": False,
            "installerOrNewCandidate": False,
        })
finally:
    (root / "ci-artifact-download-attempts.json").write_text(
        json.dumps(attempts, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
output.write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "runId": 38431616359,
    "source": source, "artifacts": observations,
    "authorizationHeaderForwardedToStorageHost": False,
    "productInstallerAcceptance": False, "newNativeInstanceStarted": False,
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": 38431616359, "archivesVerified": len(observations),
                  "archiveContentsExecuted": False}))
