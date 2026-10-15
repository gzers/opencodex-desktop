"""Download the one terminal diagnostic artifact, verify API digest/ZIP and retain original bytes."""
import hashlib
import io
import json
import os
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
terminal = json.loads((root / "diagnostic-terminal.json").read_text(encoding="utf-8"))
run = terminal["run"]
assert run["id"] == 38662786466 and run["head_sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
assert run["status"] == "completed" and run["run_attempt"] == 1
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


def get(path, destination, limit, redirect=False):
    assert not destination.exists(), "Preserve partial downloads; inspect instead of blindly repeating."
    config = (
        f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
        f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
    args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
            "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "45",
            "--max-filesize", str(limit), "--output", str(destination)]
    if redirect:
        args += ["--location", "--proto-redir", "=https"]
    response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=50)
    if response.returncode:
        failure = root / (destination.name + ".download-error.json")
        failure.write_text(json.dumps({"curlExit": response.returncode, "retryPerformed": False,
                                       "remoteTerminalClaimUnchanged": True}) + "\n", encoding="utf-8")
        raise RuntimeError("Download unavailable; retain partial bytes.")
    assert destination.stat().st_size <= limit
    return destination.read_bytes()


artifacts_path = root / "diagnostic-artifacts.original.json"
artifact_bytes = get(f"actions/runs/{run['id']}/artifacts?per_page=100", artifacts_path, 1024 * 1024)
artifacts = json.loads(artifact_bytes)
assert artifacts["total_count"] == len(artifacts["artifacts"]) == 1
artifact = artifacts["artifacts"][0]
assert artifact["name"] == "windows-query-etw-" + run["head_sha"]
assert artifact["workflow_run"]["id"] == run["id"] and artifact["workflow_run"]["head_sha"] == run["head_sha"]
assert not artifact["expired"]
archive_path = root / f"diagnostic-artifact-{artifact['id']}.zip"
raw = get(f"actions/artifacts/{artifact['id']}/zip", archive_path, 100 * 1024 * 1024, redirect=True)
assert len(raw) == artifact["size_in_bytes"]
assert artifact["digest"] == "sha256:" + hashlib.sha256(raw).hexdigest()
entries = []
seen = set()
original_root = root / "diagnostic-original"
assert not original_root.exists()
original_root.mkdir()
with zipfile.ZipFile(io.BytesIO(raw)) as archive:
    assert archive.testzip() is None
    assert sum(entry.file_size for entry in archive.infolist()) < 100 * 1024 * 1024
    for entry in archive.infolist():
        path = PurePosixPath(entry.filename)
        assert not path.is_absolute() and ".." not in path.parts and "\\" not in entry.orig_filename
        assert ":" not in entry.orig_filename and "\0" not in entry.orig_filename
        assert entry.filename.casefold() not in seen
        seen.add(entry.filename.casefold())
        if entry.is_dir():
            continue
        data = archive.read(entry)
        target = original_root.joinpath(*path.parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        entries.append({"path": entry.filename, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "runId": run["id"], "collectorSource": run["head_sha"],
    "artifactId": artifact["id"], "archiveSha256": hashlib.sha256(raw).hexdigest(), "archiveBytes": len(raw),
    "apiDigestMatches": True, "zipCrcAndSafePathsPassed": True, "entries": entries,
    "authorizationForwardedAcrossHosts": False, "installerExecuted": False,
    "originalFailureCleared": False, "versionAcceptanceClaimed": False,
}
(root / "diagnostic-download-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: report[key] for key in (
    "runId", "artifactId", "archiveSha256", "archiveBytes", "apiDigestMatches", "zipCrcAndSafePathsPassed")}))
print(json.dumps({"retainedOriginalFiles": len(entries)}))
