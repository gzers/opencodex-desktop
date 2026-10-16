"""Verify only the completed collector checks, retaining whole-CI pending state."""
import hashlib
import importlib.util
import json
import re
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "This collector checkpoint is frozen"
SOURCE = "f2271f361b9da562b62c71afcdbeb15e61d183e2"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("idle_check_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
download = load("idle_check_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request("actions/runs/38834762492", token)
_, jobs = api.request("actions/runs/38834762492/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
job, = [j for j in jobs["jobs"] if j["name"] == "windows-smoke-evidence"]
assert job["status"] == "completed" and job["conclusion"] == "success"
(ROOT / "smoke-job.original.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
log = ROOT / "smoke-job.original.log"
receipt = ROOT / "smoke-job-download.json"
if not log.exists():
    saved = download.download(api.BASE + f"actions/jobs/{job['id']}/logs",
                                log, 4 * 1024 * 1024, token=token)
    receipt.write_text(json.dumps(saved, indent=2) + "\n", encoding="utf-8")
assert receipt.exists()
text = re.sub(r"\x1b\[[0-9;]*m", "", log.read_text(encoding="utf-8-sig"))
assert re.search(r'"checks":\s*55', text)
step, = [s for s in job["steps"] if s["name"] == "Verify idle counter integrity without a desktop instance"]
assert step["status"] == "completed" and step["conclusion"] == "success"
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
_, listing = api.request("actions/runs/38834762492/artifacts?per_page=100", token)
artifact, = [a for a in listing["artifacts"] if a["name"] == "windows-process-token-evidence"]
assert not artifact["expired"] and artifact["workflow_run"]["head_sha"] == SOURCE
assert artifact["workflow_run"]["id"] == run["id"]
(ROOT / "smoke-artifact.original.json").write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
archive = ROOT / "smoke-artifact.original.zip"
if not archive.exists():
    download.download(artifact["archive_download_url"], archive, 8 * 1024 * 1024,
                      token=token, expected=artifact["digest"][7:])
payload = archive.read_bytes()
assert len(payload) == artifact["size_in_bytes"]
assert hashlib.sha256(payload).hexdigest() == artifact["digest"][7:]
entries = []
with zipfile.ZipFile(archive) as bundle:
    download.checked_zip(bundle)
    for name in ("idle-family/result.json", "idle-family/failure-records.original.jsonl"):
        matches = [n for n in bundle.namelist() if n == name or n.endswith("/" + name)]
        assert len(matches) == 1
        data = bundle.read(matches[0])
        target = ROOT / ("ci-" + Path(name).name)
        if target.exists():
            assert target.read_bytes() == data
        else:
            target.write_bytes(data)
        entries.append({"archivePath": matches[0], "savedAs": target.name,
                        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
result = json.loads((ROOT / "ci-result.json").read_text(encoding="utf-8-sig"))
assert result["status"] == "passed" and result["checks"] == 55
assert not result["productStarted"] and not result["etwStarted"] and not result["longMeasurementRun"]
summary = {"source": SOURCE, "runId": run["id"], "jobId": job["id"],
           "actualCheckout": checkout, "scope": "completed windows-smoke-evidence job only",
           "checks": 55, "wholeCiStatusAtRead": run["status"],
           "wholeCiConcludedSuccess": run["status"] == "completed" and run["conclusion"] == "success",
           "artifactId": artifact["id"], "artifactSha256": artifact["digest"][7:],
           "entries": entries, "apiDigestAndCrcVerified": True,
           "fullPerformanceGatePassed": False, "productCandidateRebuilt": False}
(ROOT / "smoke-verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
