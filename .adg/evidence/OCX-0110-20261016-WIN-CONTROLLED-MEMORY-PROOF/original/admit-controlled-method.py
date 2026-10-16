"""Verify this first CI job's self-allocation proof, never replay a trace."""
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
SOURCE = "a5a1aba290793b23c6d27c35edd0ab227d4f5e50"
RUN = 38925613291
JOB = 117056254920


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def save(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


api = load("controlled_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("controlled_download", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, job = api.request(f"actions/jobs/{JOB}", token)
_, artifacts = api.request(f"actions/runs/{RUN}/artifacts?per_page=100", token)
assert run["head_sha"] == job["head_sha"] == SOURCE and run["run_attempt"] == job["run_attempt"] == 1
assert run["event"] == "push" and job["run_id"] == RUN and job["name"] == "windows-smoke-evidence"
assert job["status"] == "completed" and job["conclusion"] == "success"
step, = [s for s in job["steps"] if s["name"] == "Verify controlled self-process memory allocation capture"]
assert step["conclusion"] == "success" and step["status"] == "completed"
save("ci-method-job-and-artifacts.original.json", {"run": run, "job": job, "artifacts": artifacts})
artifact, = [a for a in artifacts["artifacts"] if a["name"] == "windows-process-token-evidence"]
assert artifact["workflow_run"]["head_sha"] == SOURCE and not artifact["expired"]
assert re.fullmatch(r"sha256:[0-9a-f]{64}", artifact["digest"])
archive_path = ROOT / "ci-method.original.zip"
safe.download(artifact["archive_download_url"], archive_path, 16 * 1024 * 1024,
              token=token, expected=artifact["digest"][7:])
assert archive_path.stat().st_size == artifact["size_in_bytes"]
members = [
    "result.json", "guards.decoded.log", "capture.decoded.log", "capture/result.json",
    "capture/owned-memory.events.jsonl", "existing-refusal.decoded.log",
    "boundary-refusal.decoded.log", "decoder-refusal.decoded.log",
]
with zipfile.ZipFile(archive_path) as archive:
    safe.checked_zip(archive)
    for name in members:
        entry, = [m for m in archive.infolist() if m.filename.endswith("controlled-memory/" + name)]
        target = ROOT / "ci-controlled" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as out:
            out.write(archive.read(entry))
read = lambda name: json.loads((ROOT / "ci-controlled" / name).read_text(encoding="utf-8-sig"))
result, capture = read("result.json"), read("capture/result.json")
assert result["passed"] and result["compiledGuardChecks"] == 20 and result["administratorToken"]
assert result["controlledRecordingRequested"] and result["controlledRecordingPassed"]
assert not result["ordinaryTokenRefusalTested"], "CI must not pretend its elevated token exercised the ordinary branch"
assert capture["passed"] and capture["probeBytes"] == 8388608 and capture["probeFreed"]
assert capture["matchingProbeAllocationEvents"] > 0 and capture["matchingAllocationStacks"] > 0
assert capture["nativeStop"]["Status"] == 0
assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
assert capture["readerJoined"] and capture["ownedSessionAbsent"] and capture["retainedSelfGenerationUnchanged"]
assert capture["retainedSelfProcessHandle"] and not capture["stopProcessingUsed"]
assert capture["omittedRecords"] == capture["callbackErrors"] == 0 and capture["cleanupErrors"] == []
assert not any(capture[k] for k in [
    "systemWideEtlWritten", "unrelatedEventPayloadsSerialized", "rawAllocationAddressSaved",
    "targetMemoryContentsSaved", "nativeDesktopStarted", "originalTestLibraryExecuted",
    "officialQueryNodeExecuted", "fullProductGatePassed", "genericSessionStopInvoked",
    "globalSampleIntervalExplicitlyWritten", "globalSampleIntervalBeforeAfterMeasured",
])
raw = (ROOT / "ci-controlled/capture/owned-memory.events.jsonl").read_bytes()
assert b"\r" not in raw and len(raw) <= capture["maximumSerializedBytes"]
rows = [json.loads(line) for line in raw.splitlines()]
assert len(rows) == capture["records"] and len(raw) == capture["serializedBytes"]
assert all(row["pid"] == capture["selfPid"] for row in rows)
assert any(row["kind"] == "virtual-allocation" and row["length"] == 8388608 for row in rows)
assert any(row["kind"] == "allocation-stack" and row["frameCount"] > 0 for row in rows)
for label, name in [("recorder", "windows-controlled-memory.cs"), ("wrapper", "windows-controlled-memory.ps1"),
                    ("sharedNativeStop", "windows-owned-etw.cs")]:
    blob = subprocess.check_output(["git", "show", SOURCE + ":test/smoke/" + name], cwd=REPO)
    assert b"\r\n" not in blob
    assert result["sourceSha256"][label] in {
        hashlib.sha256(blob).hexdigest(), hashlib.sha256(blob.replace(b"\n", b"\r\n")).hexdigest()}
log = ROOT / "ci-method-job.original.log"
safe.download(api.BASE + f"actions/jobs/{JOB}/logs", log, 20 * 1024 * 1024, token=token)
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log.read_text(encoding="utf-8-sig"))
assert checkout == SOURCE
admission = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": RUN, "jobId": JOB,
    "actualCheckout": checkout, "controlledSelfCaptureJobSucceeded": True,
    "outerChecks": result["checks"], "compiledGuardChecks": 20,
    "probeBytes": capture["probeBytes"], "matchingProbeAllocationEvents": capture["matchingProbeAllocationEvents"],
    "matchingAllocationStacks": capture["matchingAllocationStacks"],
    "selfEventRecords": len(rows), "serializedEventBytes": len(raw),
    "zeroLostOmittedAndDecoderErrors": True, "ownedSessionAbsent": True,
    "readerDrainedWithoutForcedStop": True, "probeFreed": True,
    "artifactId": artifact["id"], "artifactSha256": artifact["digest"][7:],
    "artifactBytes": artifact["size_in_bytes"], "zipCrcVerified": True,
    "jobLogSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
    "wholeCiStatusAtRead": run["status"], "wholeCiConclusionAtRead": run["conclusion"],
    "wholeCiAdmitted": False, "hostRecordingExecuted": False, "productAllocationAttributed": False,
    "originalBd2FailureCleared": False, "fullProductGatePassed": False, "ciRerunRequested": False,
}
save("method-admission.json", admission)
print(json.dumps(admission))
