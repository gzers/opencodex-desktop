"""Retrieve only the admitted run's terminal diagnostic artifact."""
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("etw_admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
run_id = json.loads((root / "diagnostic-handle.json").read_text())["runId"]
token = api.credential()
_, run = api.request("actions/runs/" + str(run_id), token)
_, jobs = api.request("actions/runs/" + str(run_id) + "/jobs?per_page=100", token)
assert run["status"] == "completed" and run["run_attempt"] == 1 and run["head_sha"] == api.SOURCE
terminal_path = root / "diagnostic-terminal.original.json"
if terminal_path.exists():
    previous = json.loads(terminal_path.read_text())
    assert previous["run"]["id"] == run_id and previous["run"]["conclusion"] == run["conclusion"]
else:
    terminal_path.write_text(json.dumps({"atUtc": datetime.now(timezone.utc).isoformat(),
                                         "run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
assert next(job for job in jobs["jobs"] if job["name"] == "package")["conclusion"] == "skipped"
_, listing = api.request("actions/runs/" + str(run_id) + "/artifacts?per_page=100", token)
matches = [item for item in listing["artifacts"] if item["name"] == "windows-query-etw-" + api.SOURCE]
assert len(matches) == 1
artifact = matches[0]
assert artifact["workflow_run"]["id"] == run_id and not artifact["expired"]
assert artifact["digest"].startswith("sha256:")
metadata = root / "diagnostic-artifact.original.json"
if metadata.exists():
    assert json.loads(metadata.read_text())["id"] == artifact["id"]
else:
    metadata.write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
zip_path = root / ("diagnostic-artifact-" + str(artifact["id"]) + ".original.zip")
if not zip_path.exists():
    admit.download(api.BASE + "actions/artifacts/" + str(artifact["id"]) + "/zip",
                   zip_path, 32 * 1024 * 1024, token=token, expected=artifact["digest"][7:])
assert zip_path.stat().st_size == artifact["size_in_bytes"]
assert hashlib.sha256(zip_path.read_bytes()).hexdigest() == artifact["digest"][7:]
destination = root / "diagnostic-original"
entries = []
with zipfile.ZipFile(zip_path) as archive:
    admit.checked_zip(archive)
    for info in archive.infolist():
        if info.is_dir():
            continue
        assert info.filename.startswith(("out/windows-query-etw/", "runtime/ci-bd2-query-etw/capture/"))
        data = archive.read(info)
        target = destination / info.filename
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == data
        else:
            with target.open("xb") as stream:
                stream.write(data)
        entries.append({"path": info.filename, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
report = {"runId": run_id, "source": api.SOURCE, "artifactId": artifact["id"],
          "artifactBytes": zip_path.stat().st_size, "artifactSha256": artifact["digest"][7:],
          "apiDigestMatches": True, "safePathsAndCrcVerified": True, "entries": entries,
          "crossHostCredentialsForwarded": False}
report_path = root / "diagnostic-download-verification.json"
if report_path.exists():
    assert json.loads(report_path.read_text()) == report
else:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": run_id, "conclusion": run["conclusion"], "artifactId": artifact["id"],
                  "entries": len(entries), "apiDigestMatches": True}))
