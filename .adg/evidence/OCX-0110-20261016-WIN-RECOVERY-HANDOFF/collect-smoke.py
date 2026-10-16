"""Admit the completed exact-push Windows handoff checks, not a whole CI."""
import hashlib
import importlib.util
import json
import re
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("handoff_smoke_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
download = load("handoff_smoke_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request("actions/runs/38838964154", token)
_, jobs = api.request("actions/runs/38838964154/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["event"] == "push" and run["run_attempt"] == 1
job, = [j for j in jobs["jobs"] if j["name"] == "windows-smoke-evidence"]
assert job["status"] == "completed" and job["conclusion"] == "success"
step, = [s for s in job["steps"] if s["name"] == "Verify recovery restart waits for the old process exit"]
assert step["conclusion"] == "success"
(ROOT / "smoke-job.original.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
log, receipt = ROOT / "smoke-job.original.log", ROOT / "smoke-job-download.json"
if not log.exists():
    saved = download.download(api.BASE + f"actions/jobs/{job['id']}/logs", log, 4 * 1024 * 1024, token=token)
    receipt.write_text(json.dumps(saved, indent=2) + "\n", encoding="utf-8")
assert receipt.exists(), "Retain partial downloads and investigate"
text = re.sub(r"\x1b\[[0-9;]*m", "", log.read_text(encoding="utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE and re.search(r'"checks":\s*40', text)
_, listing = api.request("actions/runs/38838964154/artifacts?per_page=100", token)
artifact, = [a for a in listing["artifacts"] if a["name"] == "windows-process-token-evidence"]
assert not artifact["expired"] and artifact["workflow_run"]["head_sha"] == SOURCE
assert artifact["workflow_run"]["id"] == run["id"]
(ROOT / "smoke-artifact.original.json").write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
archive = ROOT / "smoke-artifact.original.zip"
if not archive.exists():
    download.download(artifact["archive_download_url"], archive, 8 * 1024 * 1024,
                      token=token, expected=artifact["digest"][7:])
assert len(archive.read_bytes()) == artifact["size_in_bytes"]
assert hashlib.sha256(archive.read_bytes()).hexdigest() == artifact["digest"][7:]
entries = []
with zipfile.ZipFile(archive) as bundle:
    download.checked_zip(bundle)
    for wanted in ("restart-handoff/result.json", "restart-handoff/commit-after-exit-target.json"):
        member, = [n for n in bundle.namelist() if n == wanted or n.endswith("/" + wanted)]
        data = bundle.read(member)
        target = ROOT / ("ci-" + Path(wanted).name)
        if target.exists():
            assert target.read_bytes() == data
        else:
            target.write_bytes(data)
        entries.append({"archivePath": member, "savedAs": target.name,
                        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
result = json.loads((ROOT / "ci-result.json").read_text(encoding="utf-8-sig"))
assert result["status"] == "passed" and result["checks"] == 40 and len(result["cases"]) == 5
assert not result["cleanup"] and not result["productStarted"] and not result["nativeRecoveryGatePassed"]
summary = {
    "source": SOURCE, "runId": run["id"], "jobId": job["id"], "actualCheckout": checkout,
    "checks": 40, "controlledPairs": 5, "artifactId": artifact["id"],
    "artifactSha256": artifact["digest"][7:], "entries": entries,
    "wholeCiStatusAtRead": run["status"], "wholeCiConclusionAtRead": run["conclusion"],
    "apiDigestAndCrcVerified": True, "nativeProductStarted": False,
    "nativeRecoveryGatePassed": False, "fullPerformanceGatePassed": False,
}
(ROOT / "smoke-verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
