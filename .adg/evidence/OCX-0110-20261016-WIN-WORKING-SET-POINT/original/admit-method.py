"""Read the existing Windows method job and its immutable evidence artifact."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
SOURCE = "3a2ff08a6b480d237e93502626861090aa14423d"
RUN = 38910385568
JOB = 117004428361


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write(name, value):
    path = ROOT / name
    assert not path.exists(), name
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


api = load("method_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("method_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, job = api.request(f"actions/jobs/{JOB}", token)
_, artifacts = api.request(f"actions/runs/{RUN}/artifacts?per_page=100", token)
assert run["head_sha"] == job["head_sha"] == SOURCE and run["run_attempt"] == job["run_attempt"] == 1
assert run["event"] == "push" and job["run_id"] == RUN and job["name"] == "windows-smoke-evidence"
assert job["status"] == "completed" and job["conclusion"] == "success"
step, = [s for s in job["steps"] if s["name"] == "Verify bound read-only working-set metadata"]
assert step["status"] == "completed" and step["conclusion"] == "success"
write("method-job-and-artifacts.original.json", {"run": run, "job": job, "artifacts": artifacts})
artifact, = [a for a in artifacts["artifacts"] if a["name"] == "windows-process-token-evidence"]
assert artifact["workflow_run"]["head_sha"] == SOURCE and not artifact["expired"]
assert re.fullmatch("sha256:[0-9a-f]{64}", artifact["digest"])
zip_path = ROOT / "method-evidence.original.zip"
safe.download(artifact["archive_download_url"], zip_path, 16 * 1024 * 1024,
              token=token, expected=artifact["digest"][7:])
assert zip_path.stat().st_size == artifact["size_in_bytes"]
with zipfile.ZipFile(zip_path) as archive:
    safe.checked_zip(archive)
    for base in ["result.json", "controlled-self.original.json", "controlled-pages.original.json"]:
        entry, = [m for m in archive.infolist() if m.filename.endswith("working-set/" + base)]
        path = ROOT / ("ci-" + base)
        assert not path.exists()
        path.write_bytes(archive.read(entry))
result = json.loads((ROOT / "ci-result.json").read_text(encoding="utf-8-sig"))
controlled = json.loads((ROOT / "ci-controlled-self.original.json").read_text(encoding="utf-8-sig"))
assert result["passed"] and result["checks"] == 34 and not result["nativeProductStarted"]
assert not result["productSnapshotTaken"] and result["pageProbeOwnAllocationsReleased"]
blob = subprocess.check_output(["git", "show", SOURCE + ":test/smoke/windows-working-set.ps1"], cwd=REPO)
assert b"\r\n" not in blob
variants = {
    "git-lf": hashlib.sha256(blob).hexdigest(),
    "git-checkout-crlf": hashlib.sha256(blob.replace(b"\n", b"\r\n")).hexdigest(),
}
assert controlled["sourceMethodSha256"] in variants.values(), "CI helper bytes must match the same Git source or checkout newline transform"
log = ROOT / "method-job.original.log"
safe.download(api.BASE + f"actions/jobs/{JOB}/logs", log, 16 * 1024 * 1024, token=token)
text = log.read_text(encoding="utf-8-sig")
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
value = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": RUN, "jobId": JOB,
    "windowsMethodJobSucceeded": True, "actualCheckout": checkout, "controlledChecks": 34,
    "ciMethodSha256": controlled["sourceMethodSha256"],
    "ciMethodCheckoutVariant": next(k for k, v in variants.items() if v == controlled["sourceMethodSha256"]),
    "localMethodSha256": hashlib.sha256((REPO / "test/smoke/windows-working-set.ps1").read_bytes()).hexdigest(),
    "artifactId": artifact["id"], "artifactSha256": artifact["digest"][7:],
    "artifactBytes": artifact["size_in_bytes"], "zipCrcVerified": True,
    "logSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
    "wholeCiStatusAtRead": run["status"], "wholeCiConclusionAtRead": run["conclusion"],
    "wholeCiAdmitted": False, "diagnosticMetadataOnly": True, "productGateAccepted": False,
    "originalQueryReplayRequested": False, "hostProductStarted": False,
}
write("method-admission.json", value)
print(json.dumps(value))
