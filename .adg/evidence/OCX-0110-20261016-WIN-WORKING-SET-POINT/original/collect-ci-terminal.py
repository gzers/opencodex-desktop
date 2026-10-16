"""Read the first push run only; never dispatch or rerun CI."""
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
SOURCE = "3a2ff08a6b480d237e93502626861090aa14423d"
RUN = 38910385568


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def save(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


api = module("ci_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = module("ci_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
assert jobs["total_count"] == len(jobs["jobs"])
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
save(f"ci-{stamp}.original.json", {"atUtc": stamp, "run": run, "jobs": jobs})
if run["status"] != "completed":
    print(json.dumps({"status": run["status"], "jobs": [
        {"name": j["name"], "status": j["status"], "conclusion": j["conclusion"]} for j in jobs["jobs"]
    ]}))
    sys.exit(0)
safe.verify_ci(run, jobs, SOURCE)
records = []
for job in jobs["jobs"]:
    assert job["run_attempt"] == 1 and job["head_sha"] == SOURCE
    if job["name"] not in safe.REQUIRED_JOBS:
        assert job["name"] == "build" and job["conclusion"] == "skipped"
        continue
    target = ROOT / ("ci-terminal-" + job["name"] + ".original.log")
    if job["name"] == "windows-smoke-evidence":
        target = ROOT / "method-job.original.log"
    elif not target.exists():
        safe.download(api.BASE + f"actions/jobs/{job['id']}/logs", target, 20 * 1024 * 1024, token=token)
    text = target.read_text(encoding="utf-8-sig")
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE, job["name"]
    counts = re.findall(r"test result: ok[^\r\n]*", text)
    records.append({
        "job": job["name"], "jobId": job["id"], "actualCheckout": checkout,
        "path": target.name, "bytes": target.stat().st_size,
        "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
        "rustTestResults": counts,
    })
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "runId": RUN, "attempt": 1, "event": run["event"], "status": run["status"],
    "conclusion": run["conclusion"], "requiredJobLogs": records,
    "wholeToolCiAdmitted": True, "productCandidateSource": "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
    "sameSourceAsProductCandidate": False, "fullProductGateAccepted": False,
    "newDispatchRequested": False,
}
save("ci-terminal-admission.json", summary)
print(json.dumps(summary))
