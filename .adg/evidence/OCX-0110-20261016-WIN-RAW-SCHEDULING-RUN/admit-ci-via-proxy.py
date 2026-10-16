"""Admit one whole successful collector CI and its actual checkout/job logs."""
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "Frozen evidence"
SOURCE = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
RUN = 38858859494


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("named_query_ci_reader", ROOT / "github-reader-via-proxy.py")
admit = load("named_query_ci_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1 and run["event"] == "push"
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"ci-check-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
if run["status"] != "completed":
    print(json.dumps({"runId": RUN, "source": SOURCE, "status": run["status"],
                      "recordingAdmitted": False}))
    sys.exit(0)
assert run["conclusion"] == "success", "Retain failure; do not execute the recording"
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert required <= {j["name"] for j in jobs["jobs"]} <= required | {"build"}
assert all(j["status"] == "completed" and j["conclusion"] == "success"
           for j in jobs["jobs"] if j["name"] in required)
assert all(j["conclusion"] == "skipped" for j in jobs["jobs"] if j["name"] == "build")
_, head = api.request(f"git/commits/{SOURCE}", token)
(ROOT / "head-commit.original.json").write_text(json.dumps(head, indent=2) + "\n", encoding="utf-8")
(ROOT / "ci-admission-terminal.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
summary = []
for name in ("frontend", "backend", "backend-windows"):
    job, = [j for j in jobs["jobs"] if j["name"] == name]
    log = ROOT / f"ci-{RUN}-{name}.original.log"
    receipt = log.with_suffix(".download.json")
    if not log.exists():
        download = admit.download(api.BASE + f"actions/jobs/{job['id']}/logs",
                                  log, 16 * 1024 * 1024, token=token)
        receipt.write_text(json.dumps(download, indent=2) + "\n", encoding="utf-8")
    assert receipt.exists(), "Partial logs must not be treated as complete"
    data = log.read_bytes()
    download = json.loads(receipt.read_text(encoding="utf-8"))
    assert len(data) == download["bytes"] and hashlib.sha256(data).hexdigest() == download["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    if name == "frontend":
        assert re.search(r"Test Files\s+99 passed", text) and re.search(r"Tests\s+535 passed", text)
        counts = {"files": 99, "passed": 535, "failed": 0}
    else:
        suites = [tuple(map(int, values)) for values in re.findall(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
        assert len(suites) == 35 and sum(s[1] for s in suites) == 0
        assert sum(s[0] for s in suites) == (923 if name == "backend" else 648)
        counts = {"suites": 35, "passed": sum(s[0] for s in suites),
                  "failed": 0, "ignored": sum(s[2] for s in suites)}
    summary.append({"name": name, "jobId": job["id"], "actualCheckout": checkout,
                    "counts": counts, "log": log.name, **download})
record = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
          "tree": head["tree"]["sha"], "runId": RUN, "runAttempt": 1,
          "wholeCiSucceeded": True, "originalCheckoutAndJobsVerified": True,
          "requiredJobs": sorted(required), "jobs": summary,
          "recordingStarted": False, "historicalFailureCleared": False,
          "releaseAdmission": False}
(ROOT / "ci-admission.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": RUN, "source": SOURCE, "status": "admitted",
                  "originalJobs": 3, "recordingStarted": False}))
