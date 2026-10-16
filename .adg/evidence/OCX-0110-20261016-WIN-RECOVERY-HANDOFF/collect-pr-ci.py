"""Collect completed jobs from the same PR run and bind its actual checkout."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
RUN = 38838969102


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("handoff_pr_ci_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
download = load("handoff_pr_ci_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["event"] == "pull_request" and run["run_attempt"] == 1
assert jobs["total_count"] == len(jobs["jobs"])
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"pr-ci-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert required <= {j["name"] for j in jobs["jobs"]} <= required | {"build"}
head_tree = subprocess.run(
    ["git", "-c", "core.longpaths=true", "-C", str(api.CODE), "rev-parse", SOURCE + "^{tree}"],
    check=True, capture_output=True).stdout.decode().strip()
results = []
for name in ("frontend", "backend", "backend-windows"):
    job, = [j for j in jobs["jobs"] if j["name"] == name]
    if job["status"] != "completed":
        results.append({"name": name, "jobId": job["id"], "status": job["status"], "logArchived": False})
        continue
    job_path = ROOT / ("pr-" + name + "-job.original.json")
    if not job_path.exists():
        job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    log, receipt = ROOT / ("pr-" + name + ".original.log"), ROOT / ("pr-" + name + "-download.json")
    if not log.exists():
        saved = download.download(api.BASE + f"actions/jobs/{job['id']}/logs", log,
                                  16 * 1024 * 1024, token=token)
        receipt.write_text(json.dumps(saved, indent=2) + "\n", encoding="utf-8")
    assert receipt.exists(), "Keep any incomplete original download for diagnosis"
    data = log.read_bytes()
    saved = json.loads(receipt.read_text(encoding="utf-8"))
    assert len(data) == saved["bytes"] and hashlib.sha256(data).hexdigest() == saved["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    commit_path = ROOT / ("checkout-" + checkout + ".original.json")
    if commit_path.exists():
        commit = json.loads(commit_path.read_text(encoding="utf-8"))
    else:
        _, commit = api.request("git/commits/" + checkout, token)
        commit_path.write_text(json.dumps(commit, indent=2) + "\n", encoding="utf-8")
    assert commit["sha"] == checkout and commit["tree"]["sha"] == head_tree
    result = {"name": name, "jobId": job["id"], "status": job["status"],
              "conclusion": job["conclusion"], "logArchived": True,
              "actualCheckout": checkout, "fullTree": head_tree, **saved}
    if name == "frontend" and job["conclusion"] == "success":
        assert re.findall(r"Test Files\s+(\d+) passed", text) == ["99"]
        assert re.findall(r"Tests\s+(\d+) passed", text) == ["535"]
        result.update({"files": 99, "passed": 535})
    elif job["conclusion"] == "success":
        suites = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        passed, failed, ignored = [sum(int(row[i]) for row in suites) for i in range(3)]
        assert len(suites) == 35 and failed == 0
        assert (passed, ignored) == ((923, 6) if name == "backend" else (650, 7))
        result.update({"suites": len(suites), "passed": passed, "failed": failed, "ignored": ignored})
    results.append(result)
summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "runId": RUN, "status": run["status"], "conclusion": run["conclusion"], "headFullTree": head_tree,
    "jobs": [{k: j[k] for k in ("id", "name", "status", "conclusion")} for j in jobs["jobs"]],
    "archivedJobs": results,
    "wholeCiSucceeded": run["status"] == "completed" and run["conclusion"] == "success"
    and all(j["status"] == "completed" and j["conclusion"] == "success"
            for j in jobs["jobs"] if j["name"] in required),
    "companionPushFailureCleared": False, "workflowRerunRequested": False,
    "candidateBuildRequested": False, "nativeRecoveryGatePassed": False,
}
(ROOT / "pr-ci-collection.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
