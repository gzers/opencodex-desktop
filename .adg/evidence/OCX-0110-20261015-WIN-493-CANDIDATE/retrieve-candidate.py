"""Retrieve only verified terminal candidate artifacts; installer never executed here."""
import hashlib
import json
import os
import stat
import subprocess
import sys
import urllib.parse
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
snapshot = root / "candidate-terminal.json"
terminal = json.loads(snapshot.read_text(encoding="utf-8-sig"))
run = terminal["run"]
assert run["head_sha"] == source
assert run["status"] == "completed" and run["conclusion"] == "success"
assert run["event"] == "workflow_dispatch" and run["id"] == 38673288645
assert next(row for row in terminal["jobs"]["jobs"] if row["name"] == "package")["conclusion"] == "success"
assert next(row for row in terminal["jobs"]["jobs"] if row["name"] == "query-etw")["conclusion"] == "skipped"
artifacts = terminal["artifacts"]["artifacts"]
installers = [row for row in artifacts if row["name"] == "opencodex-desktop-0.1.10-windows-x64-candidate"]
evidence = [row for row in artifacts if row["name"] == "windows-candidate-evidence"]
assert len(installers) == len(evidence) == 1
destination = code / "test/software/candidate-493da80-38673288645"
if destination.exists():
    assert "--resume-verified" in sys.argv, "Inspect existing bytes before an explicit read-only resume."
    assert destination.resolve().is_relative_to((code / "test/software").resolve())
    assert not destination.is_symlink()
else:
    destination.mkdir()
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}


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
        assert urllib.parse.urlparse(url).scheme == "https" and urllib.parse.urlparse(url).hostname == "api.github.com"
        archive = destination / f"{artifact['id']}.zip"
        reused = archive.exists()
        if not reused:
            config = (f'url = "{url}"\nheader = "Authorization: Bearer {token}"\n'
                      'header = "Accept: application/vnd.github+json"\n')
            response = subprocess.run([
                r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--location", "--proto-redir", "=https", "--connect-timeout", "10",
                "--max-time", "45", "--max-filesize", str(32 * 1024 * 1024),
                "--silent", "--show-error", "--fail", "--output", str(archive)],
                input=config.encode(), capture_output=True, timeout=50)
            if response.returncode:
                stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
                save(f"download-error-{artifact['id']}-{stamp}.json", {
                    "curlExit": response.returncode, "partialBytesRetained": archive.exists(),
                    "artifactId": artifact["id"], "automaticRetry": False})
                raise RuntimeError("Read-only download failed; inspect partial bytes instead of overwriting.")
        data = archive.read_bytes()
        assert len(data) <= 32 * 1024 * 1024
        assert len(data) == artifact["size_in_bytes"]
        digest = hashlib.sha256(data).hexdigest()
        assert artifact["digest"] == "sha256:" + digest
        with zipfile.ZipFile(archive) as bundle:
            assert bundle.testzip() is None
            assert sum(entry.file_size for entry in bundle.infolist()) <= 128 * 1024 * 1024
            seen = set()
            for entry in bundle.infolist():
                path = PurePosixPath(entry.filename)
                assert not path.is_absolute() and ".." not in path.parts
                assert "\\" not in entry.orig_filename and "\0" not in entry.orig_filename
                assert not stat.S_ISLNK(entry.external_attr >> 16)
                key = entry.filename.casefold().rstrip("/")
                assert key not in seen
                seen.add(key)
                assert not path.parts or ":" not in path.parts[0]
                assert entry.file_size < 32 * 1024 * 1024
                output = destination / ("installers" if artifact in installers else "evidence") / path
                if entry.is_dir():
                    output.mkdir(parents=True, exist_ok=True)
                else:
                    output.parent.mkdir(parents=True, exist_ok=True)
                    content = bundle.read(entry)
                    if output.exists():
                        assert "--resume-verified" in sys.argv and output.read_bytes() == content
                    else:
                        output.write_bytes(content)
        verified.append({"id": artifact["id"], "name": artifact["name"], "bytes": len(data),
                         "sha256": digest, "digestMatchesApi": True, "zipCrcPassed": True,
                         "safeEntries": True, "authorizationForwardedAcrossHosts": False,
                         "existingVerifiedArchiveReused": reused})
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
