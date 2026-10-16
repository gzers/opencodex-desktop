"""Read one of two already dispatched first-push runs; never replay anything."""
import concurrent.futures
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
RUNS = {
    "gpu": (38934816693, "391e70979ffbb32a548c0d05bdeeb62160d942d3", HERE),
    "ipc": (38933650482, "ca89fbac763d2376c6bfafcdfab4c256359d2a6b",
            Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/"
                 "test/runtime/ipc-idle-signal-20261017T0035")),
}


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


api = load("ci_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("ci_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()


def collect(key):
    run_id, source, root = RUNS[key]
    _, run = api.request(f"actions/runs/{run_id}", token)
    _, jobs = api.request(f"actions/runs/{run_id}/jobs?per_page=100", token)
    assert run["head_sha"] == source and run["run_attempt"] == 1
    assert run["event"] == "push" and run["path"] == ".github/workflows/ci.yml"
    assert jobs["total_count"] == len(jobs["jobs"])
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    snapshot = root / f"ci-terminal-check-{stamp}.original.json"
    with snapshot.open("x", encoding="utf-8") as out:
        json.dump({"atUtc": stamp, "run": run, "jobs": jobs}, out, indent=2)
        out.write("\n")
    if run["status"] != "completed":
        return {"key": key, "status": run["status"], "jobs": [
            {"name": j["name"], "status": j["status"], "conclusion": j["conclusion"],
             "active": [s["name"] for s in j["steps"] if s["status"] == "in_progress"]}
            for j in jobs["jobs"]]}
    if run["conclusion"] == "success":
        safe.verify_ci(run, jobs, source)
    records = []
    for job in jobs["jobs"]:
        assert job["head_sha"] == source and job["run_attempt"] == 1
        if job["name"] not in safe.REQUIRED_JOBS:
            assert job["name"] == "build" and job["conclusion"] == "skipped"
            continue
        target = root / ("ci-complete-" + job["name"] + ".original.log")
        if key == "gpu" and job["name"] == "windows-smoke-evidence":
            target = root / "ci-method-job.original.log"
        if not target.exists():
            safe.download(api.BASE + f"actions/jobs/{job['id']}/logs", target,
                          20 * 1024 * 1024, token=token)
        data = target.read_bytes()
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == source
        tests = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if job["conclusion"] == "success":
            assert all(row[0] == "ok" and int(row[2]) == 0 for row in tests)
        records.append({"job": job["name"], "jobId": job["id"], "conclusion": job["conclusion"],
                        "actualCheckout": checkout, "path": target.name,
                        "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data),
                        "rustResults": tests})
    summary = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": run_id,
        "attempt": 1, "event": run["event"], "status": run["status"], "conclusion": run["conclusion"],
        "snapshot": snapshot.name, "requiredJobLogs": records,
        "wholeToolCiAdmitted": run["conclusion"] == "success",
        "hostProductStarted": False, "hostTracingStarted": False,
        "oldFailureCausalityProved": False, "originalBd2FailureCleared": False,
        "rssFailureCleared": False, "fullProductGateAccepted": False,
        "newDispatchRequested": False, "ciRerunRequested": False,
    }
    target = root / "ci-complete-admission.json"
    if target.exists():
        saved = json.loads(target.read_text(encoding="utf-8"))
        assert saved["source"] == source and saved["runId"] == run_id
        assert saved["requiredJobLogs"] == records and saved["conclusion"] == summary["conclusion"]
        return saved
    with target.open("x", encoding="utf-8") as out:
        json.dump(summary, out, indent=2)
        out.write("\n")
    return summary


keys = sys.argv[1:] or list(RUNS)
assert len(set(keys)) == len(keys) and set(keys) <= set(RUNS)
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    for future in [pool.submit(collect, key) for key in keys]:
        try:
            print(json.dumps(future.result()))
        except Exception as exc:
            print(json.dumps({"error": type(exc).__name__, "detail": str(exc)}))
            raise
