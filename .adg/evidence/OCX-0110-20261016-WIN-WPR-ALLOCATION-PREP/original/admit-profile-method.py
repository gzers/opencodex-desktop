"""Admit only the existing same-SHA read-only profile job, never start recording."""
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
SOURCE = "494fc830e30cc6b56cf62f34884d44e23bfb763d"
RUN = 38920950285
JOB = 117040410248


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def save(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


api = load("profile_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("profile_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, job = api.request(f"actions/jobs/{JOB}", token)
_, artifacts = api.request(f"actions/runs/{RUN}/artifacts?per_page=100", token)
assert run["head_sha"] == job["head_sha"] == SOURCE and run["run_attempt"] == job["run_attempt"] == 1
assert run["event"] == "push" and job["name"] == "windows-smoke-evidence"
assert job["status"] == "completed" and job["conclusion"] == "success" and job["run_id"] == RUN
step, = [s for s in job["steps"] if s["name"] == "Verify read-only WPR allocation profile preparation"]
assert step["status"] == "completed" and step["conclusion"] == "success"
save("ci-method-job-and-artifacts.original.json", {"run": run, "job": job, "artifacts": artifacts})
artifact, = [a for a in artifacts["artifacts"] if a["name"] == "windows-process-token-evidence"]
assert artifact["workflow_run"]["head_sha"] == SOURCE and not artifact["expired"]
assert re.fullmatch(r"sha256:[0-9a-f]{64}", artifact["digest"])
archive_path = ROOT / "ci-method.original.zip"
safe.download(artifact["archive_download_url"], archive_path, 16 * 1024 * 1024,
              token=token, expected=artifact["digest"][7:])
assert archive_path.stat().st_size == artifact["size_in_bytes"]
extracted = []
with zipfile.ZipFile(archive_path) as archive:
    safe.checked_zip(archive)
    for name in ["result.json", "host-gpu-va-export.original.wprp", "ocx-gpu-allocation.wprp",
                 "export.decoded.log", "profile-inventory.decoded.log", "profile-details.decoded.log"]:
        entry, = [m for m in archive.infolist() if m.filename.endswith("memory-profile/" + name)]
        target = ROOT / "ci-profile" / name
        target.parent.mkdir(exist_ok=True)
        with target.open("xb") as out:
            out.write(archive.read(entry))
        extracted.append(name)
result = json.loads((ROOT / "ci-profile/result.json").read_text(encoding="utf-8-sig"))
assert result["passed"] and result["checks"] == 18 and result["readOnlyWprParserValidated"]
assert result["configuredBufferBytes"] == 33554432
assert not any(result[k] for k in ["recordingStarted", "eventLossVerified", "fileGrowthBoundEnforced",
                                  "executionAdmitted", "desktopProductStarted", "queryReplayed",
                                  "fullPerformanceGatePassed"])
for path, field in [("host-gpu-va-export.original.wprp", "exportSha256"),
                    ("ocx-gpu-allocation.wprp", "generatedProfileSha256")]:
    assert hashlib.sha256((ROOT / "ci-profile" / path).read_bytes()).hexdigest() == result[field]
blob = subprocess.check_output(["git", "show", SOURCE + ":test/smoke/windows-memory-profile.ps1"], cwd=REPO)
variants = {"git-lf": hashlib.sha256(blob).hexdigest(),
            "git-checkout-crlf": hashlib.sha256(blob.replace(b"\n", b"\r\n")).hexdigest()}
assert b"\r\n" not in blob and result["sourceMethodSha256"] in variants.values()
log = ROOT / "ci-method-job.original.log"
safe.download(api.BASE + f"actions/jobs/{JOB}/logs", log, 16 * 1024 * 1024, token=token)
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log.read_text(encoding="utf-8-sig"))
assert checkout == SOURCE
admission = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": RUN, "jobId": JOB,
    "actualCheckout": checkout, "windowsReadOnlyPreparationJobSucceeded": True, "checks": 18,
    "configuredBufferBytes": 33554432, "artifactId": artifact["id"],
    "artifactSha256": artifact["digest"][7:], "artifactBytes": artifact["size_in_bytes"],
    "archiveCrcVerified": True, "extractedMembers": extracted,
    "ciSourceMethodSha256": result["sourceMethodSha256"],
    "ciCheckoutVariant": next(k for k, v in variants.items() if v == result["sourceMethodSha256"]),
    "jobLogSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
    "wholeCiStatusAtRead": run["status"], "wholeCiConclusionAtRead": run["conclusion"],
    "wholeCiAdmitted": False, "recordingExecutionAdmitted": False, "eventLossVerified": False,
    "fileGrowthBoundEnforced": False, "fullProductGatePassed": False, "newDispatchRequested": False,
}
save("method-admission.json", admission)
print(json.dumps(admission))
