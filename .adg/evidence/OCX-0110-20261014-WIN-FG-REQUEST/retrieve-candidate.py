"""Retrieve only verified terminal candidate artifacts; installer never executed here."""
import hashlib
import json
import os
import subprocess
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
snapshot = root / "candidate-terminal.json"
if not snapshot.exists():
    snapshot = root / "candidate-artifact-ready.json"
terminal = json.loads(snapshot.read_text(encoding="utf-8-sig"))
run = terminal["run"]
assert run["head_sha"] == source
if run["status"] == "completed":
    assert run["conclusion"] == "success"
else:
    assert terminal["candidateProductStepsAndUploadCompleted"]
    assert not terminal["wholeRunSuccessClaimed"]
assert run["event"] == "workflow_dispatch" and run["id"] == 38591604133
artifacts = terminal["artifacts"]["artifacts"]
installers = [row for row in artifacts if row["name"] == "opencodex-desktop-0.1.10-windows-x64-candidate"]
evidence = [row for row in artifacts if row["name"] == "windows-candidate-evidence"]
assert len(installers) == len(evidence) == 1
destination = code / "test/software/candidate-e6a4b32-38591604133"
assert not destination.exists(), "Inspect existing download instead of overwriting candidate bytes."
destination.mkdir()
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def save(name, value):
    path = root / name
    assert not path.exists()
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
                                input="protocol=https\nhost=github.com\n\n", check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
    verified = []
    for artifact in installers + evidence:
        assert not artifact["expired"] and artifact["workflow_run"]["head_sha"] == source
        url = artifact["archive_download_url"]
        assert urllib.parse.urlparse(url).hostname == "api.github.com"
        request = urllib.request.Request(url, headers={
            "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
            "User-Agent": "OpenCodex-Windows-candidate-validation"})
        try:
            with urllib.request.build_opener(NoRedirect).open(request, timeout=25) as response:
                data = response.read(32 * 1024 * 1024 + 1)
        except urllib.error.HTTPError as error:
            assert error.code in (301, 302, 303, 307, 308)
            target = urllib.parse.urljoin(url, error.headers["Location"])
            parsed = urllib.parse.urlparse(target)
            assert parsed.scheme == "https" and parsed.hostname and (
                parsed.hostname.endswith(".blob.core.windows.net") or
                parsed.hostname.endswith(".actions.githubusercontent.com"))
            # Fresh request deliberately has no GitHub Authorization header.
            with urllib.request.urlopen(target, timeout=35) as response:
                data = response.read(32 * 1024 * 1024 + 1)
        assert len(data) <= 32 * 1024 * 1024
        digest = hashlib.sha256(data).hexdigest()
        assert artifact["digest"] == "sha256:" + digest
        archive = destination / f"{artifact['id']}.zip"
        archive.write_bytes(data)
        with zipfile.ZipFile(archive) as bundle:
            assert bundle.testzip() is None
            for entry in bundle.infolist():
                path = PurePosixPath(entry.filename.replace("\\", "/"))
                assert not path.is_absolute() and ".." not in path.parts
                assert not path.parts or ":" not in path.parts[0]
                assert entry.file_size < 32 * 1024 * 1024
                output = destination / ("installers" if artifact in installers else "evidence") / path
                if entry.is_dir():
                    output.mkdir(parents=True, exist_ok=True)
                else:
                    output.parent.mkdir(parents=True, exist_ok=True)
                    output.write_bytes(bundle.read(entry))
        verified.append({"id": artifact["id"], "name": artifact["name"], "bytes": len(data),
                         "sha256": digest, "digestMatchesApi": True, "zipCrcPassed": True,
                         "safeEntries": True, "authorizationForwardedAcrossHosts": False})
    receipts = list((destination / "installers").rglob("installer-verification.json"))
    assert len(receipts) == 1
    receipt = json.loads(receipts[0].read_text(encoding="utf-8-sig"))
    assert receipt["sourceCommit"] == source and receipt["result"] == "pass" and receipt["version"] == "0.1.10"
    for installer in receipt["installers"]:
        matches = list((destination / "installers").rglob(installer["name"]))
        assert len(matches) == 1 and matches[0].stat().st_size == installer["bytes"]
        assert hashlib.sha256(matches[0].read_bytes()).hexdigest() == installer["sha256"]
        assert installer["result"] == "pass" and installer["firstScreen"] == "pass"
    identities = list((destination / "evidence").rglob("environment.json"))
    assert len(identities) == 1
    environment = json.loads(identities[0].read_text(encoding="utf-8-sig"))
    assert environment["source"] == source
    save("candidate-download-result.json", {"atUtc": datetime.now(timezone.utc).isoformat(),
         "source": source, "runId": run["id"], "destination": str(destination),
         "artifacts": verified, "originalCiFailurePreserved": True,
         "candidateRunStatusWhenRetrieved": run["status"], "candidateRunConclusionWhenRetrieved": run["conclusion"],
         "hostInstallerExecuted": False, "releaseAccepted": False})
    save("installer-verification.original.json", receipt)
    save("candidate-environment.original.json", environment)
    print((root / "candidate-download-result.json").read_text(encoding="utf-8"))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
