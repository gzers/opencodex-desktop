"""Bind original Windows release compilation versions to the formal baseline."""
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
SOURCE = "de6862d40389d527527b9db8ef52e186a58b625f"
RUN = 37871899116


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def write(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


api = load("release_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("release_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
write("release-run.original.json", run)
write("release-jobs.original.json", jobs)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
assert run["event"] == "push" and run["head_branch"] == "v0.1.9"
assert run["path"] == ".github/workflows/release.yml"
assert run["status"] == "completed" and run["conclusion"] == "success"
assert jobs["total_count"] == len(jobs["jobs"])
job, = [j for j in jobs["jobs"] if "windows" in j["name"].lower()]
assert job["run_id"] == RUN and job["run_attempt"] == 1 and job["head_sha"] == SOURCE
assert job["status"] == "completed" and job["conclusion"] == "success"
log = ROOT / "baseline-release-windows.original.log"
safe.download(api.BASE + f"actions/jobs/{job['id']}/logs", log, 24 * 1024 * 1024, token=token)
text = log.read_text(encoding="utf-8-sig")
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
versions = {}
for name, version in re.findall(r"\bCompiling\s+([a-zA-Z0-9_-]+)\s+v([0-9][0-9A-Za-z.+_-]*)", text):
    versions.setdefault(name, set()).add(version)
wanted = {k: sorted(v) for k, v in versions.items()
          if k.startswith(("tauri", "webview2", "windows", "wry", "tao"))}
assert "tauri" in wanted and "wry" in wanted and "tauri-runtime-wry" in wanted
asset = json.loads((ROOT / "release-asset.original.json").read_text(encoding="utf-8-sig"))
assert job["started_at"] <= asset["created_at"] <= job["completed_at"]
result = {
    "source": SOURCE, "runId": RUN, "jobId": job["id"], "jobName": job["name"],
    "actualCheckout": checkout, "logBytes": log.stat().st_size,
    "logSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
    "assetId": asset["id"], "assetDigest": asset["digest"],
    "assetCreatedWithinWindowsJob": True, "compiledGraphicsPackageVersions": wanted,
    "versionsAreOriginalBuildLogObservations": True,
    "baselineCargoLockRecovered": False, "individualAllocationAttributed": False,
    "originalRssFailureCleared": False, "newProductStarted": False,
    "ciRerunRequested": False, "signingOrPublicationRequested": False,
}
write("baseline-release-build-binding.json", result)
print(json.dumps(result))
