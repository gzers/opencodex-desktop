"""Archive available logs from the same attempt; never request another run."""
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("idle_bound_ci_reader",
           ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
download = load("idle_bound_ci_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request("actions/runs/38834762492", token)
_, jobs = api.request("actions/runs/38834762492/jobs?per_page=100", token)
SOURCE = "f2271f361b9da562b62c71afcdbeb15e61d183e2"
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1 and run["event"] == "push"
assert jobs["total_count"] == len(jobs["jobs"])
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"ci-collection-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert {j["name"] for j in jobs["jobs"]} == required | {"build"}
build, = [j for j in jobs["jobs"] if j["name"] == "build"]
assert build["status"] == "completed" and build["conclusion"] == "skipped"
results = []
for name in ("frontend", "backend", "backend-windows"):
    job, = [j for j in jobs["jobs"] if j["name"] == name]
    if job["status"] != "completed":
        results.append({"name": name, "status": job["status"], "logArchived": False})
        continue
    log = ROOT / (name + ".original.log")
    receipt = ROOT / (name + "-download.json")
    if not log.exists():
        saved = download.download(
            api.BASE + f"actions/jobs/{job['id']}/logs", log, 16 * 1024 * 1024, token=token)
        receipt.write_text(json.dumps(saved, indent=2) + "\n", encoding="utf-8")
    assert receipt.exists(), "Retain partial files and investigate; do not overwrite"
    data = log.read_bytes()
    saved = json.loads(receipt.read_text(encoding="utf-8"))
    assert len(data) == saved["bytes"] and hashlib.sha256(data).hexdigest() == saved["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    results.append({
        "name": name, "jobId": job["id"], "status": job["status"],
        "conclusion": job["conclusion"], "logArchived": True,
        "actualCheckout": checkout, **saved,
    })
summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(),
    "source": SOURCE, "runId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{k: j[k] for k in ("id", "name", "status", "conclusion")} for j in jobs["jobs"]],
    "archivedLogs": results,
    "wholeCiSucceeded": run["status"] == "completed" and run["conclusion"] == "success"
    and all(j["status"] == "completed" and j["conclusion"] == "success"
            for j in jobs["jobs"] if j["name"] in required),
    "expectedMainOnlyBuildSkipped": True,
    "smokeOriginalsReusedFrom": "../OCX-0110-20261016-WIN-IDLE-COLLECTOR-FIX/",
    "ciRerunRequested": False, "productStarted": False, "etwReplayRequested": False,
    "fullPerformanceGatePassed": False,
}
(ROOT / "ci-collection.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
