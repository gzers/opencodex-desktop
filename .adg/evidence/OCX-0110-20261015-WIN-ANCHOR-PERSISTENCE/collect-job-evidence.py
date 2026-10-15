"""Preserve primary per-job logs when the run-wide ZIP transport is unavailable."""
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
identifier = int(sys.argv[1])
assert identifier in (38734133095, 38734138996)
token = api.credential()
terminal = root / f"run-{identifier}-terminal.original.json"
if terminal.exists():
    frozen = json.loads(terminal.read_text(encoding="utf-8-sig"))
    run, jobs = frozen["run"], frozen["jobs"]
else:
    _, run = api.request(f"actions/runs/{identifier}", token)
    _, jobs = api.request(f"actions/runs/{identifier}/jobs?per_page=100", token)
assert run["head_sha"] == api.SOURCE and run["status"] == "completed" and run["conclusion"] == "success"
assert run["run_attempt"] == 1 and len(jobs["jobs"]) == jobs["total_count"]
if terminal.exists():
    old = json.loads(terminal.read_text())
    assert old["run"]["id"] == identifier and old["run"]["head_sha"] == api.SOURCE
    assert old["run"]["status"] == run["status"] and old["run"]["conclusion"] == run["conclusion"]
else:
    terminal.write_text(json.dumps({"atUtc": datetime.now(timezone.utc).isoformat(),
                                    "run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
checkout, totals, originals = {}, {}, []
for name in ("backend", "backend-windows"):
    job, = [item for item in jobs["jobs"] if item["name"] == name]
    assert job["status"] == "completed" and job["conclusion"] == "success"
    process = subprocess.run([sys.executable, "-B", str(root / "read-job.py"), str(identifier), name],
                             capture_output=True, text=True, check=True)
    receipt_path = root / f"run-{identifier}-job-{job['id']}.json"
    log_path = root / f"run-{identifier}-job-{job['id']}.original.log"
    receipt = json.loads(receipt_path.read_text())
    data = log_path.read_bytes()
    assert receipt["source"] == api.SOURCE and receipt["job"]["id"] == job["id"]
    assert len(data) == receipt["log"]["bytes"] and hashlib.sha256(data).hexdigest() == receipt["log"]["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    checkout[name] = actual
    counts = [tuple(map(int, row)) for row in re.findall(
        r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
    totals[name] = {"suites": len(counts), "passed": sum(row[0] for row in counts),
                    "failed": sum(row[1] for row in counts), "ignored": sum(row[2] for row in counts)}
    originals.append({"job": name, "jobId": job["id"], "metadata": receipt_path.name,
                      "file": log_path.name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
report = {
    "runId": identifier, "source": api.SOURCE, "event": run["event"], "status": run["status"],
    "conclusion": run["conclusion"], "logFormat": "individual-job-originals",
    "actualCheckout": checkout, "rustResults": totals, "originalJobs": originals,
    "runWideZipObtained": False, "archiveFabricated": False, "crossHostCredentialsForwarded": False,
    "historicalFailureCleared": False,
}
target = root / f"run-{identifier}-log-verification.json"
if target.exists():
    assert json.loads(target.read_text()) == report
else:
    target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
