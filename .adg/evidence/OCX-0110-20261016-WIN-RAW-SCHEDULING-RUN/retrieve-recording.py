"""Retrieve this one completed diagnostic; never execute or repeat its input."""
import gzip
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
RUN = 38876971001
RUNTIME = Path(f"C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/raw-scheduling-run-{RUN}")
assert not (ROOT / "manifest.json").exists()
spec = importlib.util.spec_from_file_location("recording_artifact_api", ROOT / "github-reader-via-proxy.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
admit_spec = importlib.util.spec_from_file_location("recording_zip_guard", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(admit_spec)
admit_spec.loader.exec_module(admit)
token = api.credential()
terminal = ROOT / "recording-terminal.original.json"
if terminal.exists():
    previous = json.loads(terminal.read_text(encoding="utf-8"))
    run, jobs, artifacts = previous["run"], previous["jobs"], previous["artifacts"]
else:
    _, run = api.request(f"actions/runs/{RUN}", token)
    _, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
    _, artifacts = api.request(f"actions/runs/{RUN}/artifacts?per_page=100", token)
    terminal.write_bytes((json.dumps({"run": run, "jobs": jobs, "artifacts": artifacts}, indent=2) + "\n").encode())
assert run["id"] == RUN and run["head_sha"] == SOURCE and run["run_attempt"] == 1
assert run["status"] == "completed" and run["event"] == "workflow_dispatch"
artifact, = [a for a in artifacts["artifacts"] if a["name"] == "windows-query-etw-" + SOURCE]
assert artifact["workflow_run"]["id"] == RUN and artifact["workflow_run"]["head_sha"] == SOURCE
assert not artifact["expired"] and re.fullmatch("sha256:[0-9a-f]{64}", artifact["digest"])
RUNTIME.mkdir(exist_ok=True)


def download(route, destination, cap, expected=None):
    assert not destination.exists()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    partial = destination.with_name(destination.name + "." + stamp + ".transfer")
    url = api.BASE + route
    hosts = []
    for hop in range(4):
        parsed = urlparse(url)
        assert parsed.scheme == "https" and not parsed.username and not parsed.password
        assert parsed.hostname == "api.github.com" if hop == 0 else parsed.hostname.endswith(
            (".blob.core.windows.net", ".githubusercontent.com", ".actions.githubusercontent.com"))
        hosts.append(parsed.hostname)
        config = ["url = " + json.dumps(url)]
        if hop == 0:
            config += ["header = " + json.dumps("Authorization: Bearer " + token),
                       'header = "Accept: application/vnd.github+json"']
        tls = ["--tlsv1.2", "--tls-max", "1.2"] if hop == 0 else ["--tlsv1.2"]
        result = subprocess.run([
            api.CURL, "-q", "--config", "-", "--proxy", api.PROXY, "--noproxy", "", "--http1.1", *tls,
            "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "45",
            "--max-filesize", str(cap), "--output", str(partial), "--dump-header", "-",
            "--write-out", "\nSTATUS:%{http_code}\n",
        ], input=("\n".join(config) + "\n").encode(), capture_output=True, timeout=50)
        headers = result.stdout.decode("iso-8859-1")
        statuses = re.findall(r"\nSTATUS:(\d{3})", headers)
        status = int(statuses[-1]) if statuses else 0
        if result.returncode:
            raise RuntimeError(f"Read-only artifact transfer stopped (curl {result.returncode}, HTTP {status}, host {parsed.hostname}); partial retained")
        if status == 200:
            break
        assert status in (301, 302, 303, 307, 308)
        locations = re.findall(r"(?im)^location:\s*([^\r\n]+)", headers)
        assert len(locations) == 1
        url = locations[0].strip()
    else:
        raise RuntimeError("Too many anonymous download redirects")
    data = partial.read_bytes()
    assert 0 < len(data) <= cap
    digest = hashlib.sha256(data).hexdigest()
    if expected:
        assert digest == expected, "Artifact API digest mismatch"
    partial.rename(destination)
    return {"bytes": len(data), "sha256": digest, "hosts": hosts, "proxy": api.PROXY,
            "certificateVerificationEnabled": True, "crossHostCredentialsForwarded": False,
            "globalSettingsChanged": False}


archive_path = RUNTIME / "artifact.original.zip"
if archive_path.exists():
    archive_bytes = archive_path.read_bytes()
    assert hashlib.sha256(archive_bytes).hexdigest() == artifact["digest"][7:]
    receipt = {"bytes": len(archive_bytes), "sha256": hashlib.sha256(archive_bytes).hexdigest(),
        "revalidatedExistingArchive": True, "originalTransferHostListRetained": False,
        "certificateVerificationEnabledByExecutedMethod": True,
        "crossHostCredentialsForwarded": False, "globalSettingsChanged": False}
else:
    receipt = download(f"actions/artifacts/{artifact['id']}/zip", archive_path, 64 * 1024 * 1024, artifact["digest"][7:])
assert receipt["bytes"] == artifact["size_in_bytes"]
entries = []
with zipfile.ZipFile(archive_path) as archive:
    admit.checked_zip(archive)
    for info in archive.infolist():
        if info.is_dir():
            continue
        assert info.filename.startswith(("out/windows-query-etw/", "runtime/ci-bd2-query-etw/capture/"))
        data = archive.read(info)
        target = RUNTIME / "original" / info.filename
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == data
        else:
            with target.open("xb") as output:
                output.write(data)
        public = ROOT / "remote-original" / info.filename
        public.parent.mkdir(parents=True, exist_ok=True)
        compressed = info.filename.endswith(".jsonl")
        public = public.with_suffix(public.suffix + ".gz") if compressed else public
        archived = gzip.compress(data, mtime=0) if compressed else data
        if public.exists():
            assert public.read_bytes() == archived
        else:
            with public.open("xb") as output:
                output.write(archived)
        if compressed:
            assert gzip.decompress(archived) == data
        entries.append({"path": info.filename, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
                        "archivedPath": public.relative_to(ROOT).as_posix(), "gzipRoundtrip": compressed,
                        "archivedBytes": len(archived), "archivedSha256": hashlib.sha256(archived).hexdigest()})
job, = [j for j in jobs["jobs"] if j["name"] == "query-etw"]
log_path = ROOT / f"recording-{RUN}-query-etw.original.log"
log_receipt = download(f"actions/jobs/{job['id']}/logs", log_path, 16 * 1024 * 1024)
log = re.sub(r"\x1b\[[0-9;]*m", "", log_path.read_text(encoding="utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log)
assert checkout == SOURCE
result = {"source": SOURCE, "runId": RUN, "runAttempt": 1, "artifactId": artifact["id"],
          "runtimeOriginalDirectory": str(RUNTIME / "original"), "runtimeArchive": str(archive_path),
          "artifactApi": artifact, "archive": receipt, "entries": entries,
          "originalJob": {"id": job["id"], "actualCheckout": checkout, "path": log_path.name, **log_receipt},
          "apiDigestAndCrcVerified": True, "libraryExecutedByRetrieval": False,
          "originalLibraryRunRepeated": False, "releaseRequested": False}
with (ROOT / "recording-download-verification.json").open("xb") as output:
    output.write((json.dumps(result, indent=2) + "\n").encode())
print(json.dumps({"runId": RUN, "artifactId": artifact["id"], "entries": len(entries),
                  "archiveBytes": receipt["bytes"], "archiveSha256": receipt["sha256"]}))
