"""Classify the observed dependency transfer failure; do not clear the run."""
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
job = json.loads((ROOT / "push-windows-backend-job.original.json").read_text(encoding="utf-8"))
log = (ROOT / "push-windows-backend.original.log").read_bytes()
receipt = json.loads((ROOT / "push-windows-backend-download.json").read_text(encoding="utf-8"))
assert len(log) == receipt["bytes"] and hashlib.sha256(log).hexdigest() == receipt["sha256"]
assert job["head_sha"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert job["id"] == 116774382572 and job["conclusion"] == "failure"
text = re.sub(r"\x1b\[[0-9;]*m", "", log.decode("utf-8-sig"))
assert "failed to download from `https://static.crates.io/crates/phf_generator/0.13.1/download`" in text
assert "Failed receiving HTTP2 data" in text and "Connection was reset" in text
assert not re.search(r"error\[E\d+\]", text)
assert not re.search(r"test result: (?:ok|FAILED)", text)
failed, = [s for s in job["steps"] if s["conclusion"] == "failure"]
assert "cargo clippy" in failed["name"]
skipped = [s["name"] for s in job["steps"] if s["conclusion"] == "skipped"]
assert {"Windows backend regression", "Windows final EXE smoke",
        "Run cargo build --locked --release --features custom-protocol --bins"} <= set(skipped)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("handoff_failure_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
download = load("handoff_failure_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, listing = api.request("actions/runs/38838964154/artifacts?per_page=100", token)
artifact, = [a for a in listing["artifacts"] if a["id"] == 12019729608]
assert artifact["name"] == "windows-startup-evidence" and not artifact["expired"]
assert artifact["workflow_run"]["head_sha"] == job["head_sha"]
assert artifact["digest"] == "sha256:760ed579418c822a6cfcc0e66e387c269f225dda93b7004378edf2ed9736e2d7"
(ROOT / "push-failure-artifact.original.json").write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
archive = ROOT / "push-failure-artifact.original.zip"
if not archive.exists():
    download.download(artifact["archive_download_url"], archive, 4 * 1024 * 1024,
                      token=token, expected=artifact["digest"][7:])
assert len(archive.read_bytes()) == artifact["size_in_bytes"]
assert hashlib.sha256(archive.read_bytes()).hexdigest() == artifact["digest"][7:]
with zipfile.ZipFile(archive) as bundle:
    download.checked_zip(bundle)
    name, = [n for n in bundle.namelist() if n.endswith("/test-executable-unavailable.txt")]
    unavailable = bundle.read(name)
    assert b"test executable was not produced" in unavailable
    (ROOT / "push-test-executable-unavailable.original.txt").write_bytes(unavailable)
summary = {
    "source": job["head_sha"], "runId": 38838964154, "jobId": job["id"],
    "jobConclusionRetained": "failure", "failureKind": "dependency-download-transport",
    "package": "phf_generator", "packageVersion": "0.13.1",
    "failedStep": failed["name"], "skippedSteps": skipped,
    "rustCompilerDiagnosticObserved": False, "rustClippyCompleted": False,
    "backendTestsExecuted": False, "finalExecutableProduced": False,
    "artifactId": artifact["id"], "artifactSha256": artifact["digest"][7:],
    "fullCiClaimedPassed": False, "workflowRerunRequested": False,
    "nativeRecoveryGatePassed": False, "historicalFailureCleared": False,
}
(ROOT / "push-failure-classification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
