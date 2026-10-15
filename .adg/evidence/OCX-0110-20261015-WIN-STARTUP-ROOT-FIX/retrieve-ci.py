"""Download a terminal ordinary CI artifact and preserve its first library capture."""
from datetime import datetime, timezone
import hashlib
import gzip
import importlib.util
import json
from pathlib import Path
import sys
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
run_id = int(sys.argv[1])
sources = {
    38723079993: "bc14165ceb91cb084361e8dce23b10d381330e1a",
    38723086934: "bc14165ceb91cb084361e8dce23b10d381330e1a",
    38724440086: "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9",
    38724446374: "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9",
    38726727330: "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0",
    38726732452: "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0",
}
source = sources[run_id]
token = api.credential()
_, run = api.request(f"actions/runs/{run_id}", token)
_, jobs = api.request(f"actions/runs/{run_id}/jobs?per_page=100", token)
assert run["head_sha"] == source and run["event"] in {"push", "pull_request"}
assert run["status"] == "completed" and run["run_attempt"] == 1
_, listing = api.request(f"actions/runs/{run_id}/artifacts?per_page=100", token)
artifacts = [row for row in listing["artifacts"] if row["name"] == "windows-startup-evidence"]
assert len(artifacts) == 1
artifact = artifacts[0]
assert not artifact["expired"] and artifact["workflow_run"]["id"] == run_id
destination = root / f"run-{run_id}"
destination.mkdir(exist_ok=True)
metadata_path = destination / "artifact.original.json"
if metadata_path.exists():
    assert json.loads(metadata_path.read_text())["id"] == artifact["id"]
else:
    metadata_path.write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
archive_path = destination / "artifact.original.zip"
if not archive_path.exists():
    admit.download(api.BASE + f"actions/artifacts/{artifact['id']}/zip", archive_path,
                   32 * 1024 * 1024, token=token, expected=artifact["digest"][7:])
assert archive_path.stat().st_size == artifact["size_in_bytes"]
assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == artifact["digest"][7:]
entries = []
with zipfile.ZipFile(archive_path) as archive:
    admit.checked_zip(archive)
    for info in archive.infolist():
        if info.is_dir():
            continue
        selected = (info.filename.startswith(("out/windows-query-etw/", "runtime/ci-bd2-query-etw/capture/"))
                    or info.filename in {"out/windows-backend/environment.json",
                                         "out/windows-backend/test-executable-sha256.json"})
        if not selected:
            continue
        content = archive.read(info)
        saved_name = info.filename + ".gz" if info.filename.endswith("owned-etw.events.jsonl") else info.filename
        saved_content = gzip.compress(content, mtime=0) if saved_name.endswith(".jsonl.gz") else content
        target = destination / "selected-original" / saved_name
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == saved_content
        else:
            with target.open("xb") as stream:
                stream.write(saved_content)
        entries.append({"path": info.filename, "bytes": len(content),
                        "sha256": hashlib.sha256(content).hexdigest(),
                        "savedAs": saved_name, "savedBytes": len(saved_content),
                        "savedSha256": hashlib.sha256(saved_content).hexdigest()})
report = {
    "runId": run_id, "event": run["event"], "source": source, "conclusion": run["conclusion"],
    "artifactId": artifact["id"], "artifactBytes": archive_path.stat().st_size,
    "artifactSha256": artifact["digest"][7:], "apiDigestMatches": True,
    "safePathsAndCrcVerified": True, "selectedEntries": entries, "crossHostCredentialsForwarded": False,
}
report_path = destination / "download-verification.json"
if report_path.exists():
    assert json.loads(report_path.read_text()) == report
else:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": run_id, "conclusion": run["conclusion"],
                  "artifactId": artifact["id"], "selectedEntries": len(entries)}))
