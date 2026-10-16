"""Verify the retained 391 method originals without launching or recording."""
import hashlib
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
SOURCE = "391e70979ffbb32a548c0d05bdeeb62160d942d3"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


state = read(ROOT / "ci-method-job-and-artifacts.original.json")
run, job = state["run"], state["job"]
assert run["head_sha"] == job["head_sha"] == SOURCE
assert run["id"] == job["run_id"] == 38934816693
assert run["run_attempt"] == job["run_attempt"] == 1
assert job["id"] == 117087534121 and job["conclusion"] == "success"
assert run["event"] == "push"
download = read(ROOT / "ci-method-download.json")
artifact, = [a for a in state["artifacts"]["artifacts"] if a["id"] == download["artifactId"]]
archive = ROOT / "ci-method.original.zip"
assert archive.stat().st_size == artifact["size_in_bytes"] == download["artifactBytes"]
assert sha(archive.read_bytes()) == artifact["digest"][7:] == download["artifactSha256"]
with zipfile.ZipFile(archive) as zipped:
    assert zipped.testzip() is None
    for row in download["extracted"]:
        data = (ROOT / row["path"]).read_bytes()
        assert data == zipped.read(row["member"])
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
log = ROOT / "ci-method-job.original.log"
assert sha(log.read_bytes()) == download["jobLogSha256"]
text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.read_text(encoding="utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
local_controlled = read(ROOT / "final-controlled-tests/result.json")
local_binding = read(ROOT / "final-binding-tests/result.json")
entry = read(ROOT / "entry-refusal-result.json")
ci = ROOT / "ci-method-extracted"
controlled = read(ci / "controlled-memory/result.json")
binding = read(ci / "memory-target/result.json")
capture = read(ci / "controlled-memory/capture/result.json")
assert local_controlled["passed"] and local_controlled["checks"] == 10
assert local_controlled["compiledGuardChecks"] == 20
assert local_controlled["ordinaryTokenRefusalTested"] and not local_controlled["administratorToken"]
assert controlled["passed"] and controlled["checks"] == 15 and controlled["compiledGuardChecks"] == 20
assert controlled["administratorToken"] and controlled["controlledRecordingPassed"]
assert not controlled["ordinaryTokenRefusalTested"]
for result in (local_binding, binding):
    assert result["passed"] and result["checks"] == 18
    assert result["actualCurrentTokenUserMatched"] and result["independentRetainedLeasesTested"]
    assert result["generationTickMismatchRefused"]
    assert not any(result[k] for k in ("productGpuBindingExecuted", "tracingStarted", "productStarted",
                                      "recordingExecutionAdmitted", "rawSidSaved", "rawCommandLineSaved"))
assert entry["passed"] and entry["checks"] == 8
assert all(entry[k] for k in ("ordinaryControllerRefused", "native0PrepareRefused",
                              "oldFullCiRefused", "partialCiRefused"))
paths = {
    "recorder": "test/smoke/windows-controlled-memory.cs",
    "sharedNativeStop": "test/smoke/windows-owned-etw.cs",
    "wrapper": "test/smoke/windows-controlled-memory.ps1",
    "binding": "test/smoke/windows-memory-target.ps1",
    "loader": "test/smoke/windows-memory-recorder-loader.ps1",
    "entry": "test/smoke/windows-bound-gpu-memory.ps1",
    "workflow": ".github/workflows/ci.yml",
}
source_rows = []
for role, relative in paths.items():
    blob = subprocess.check_output(["git", "show", SOURCE + ":" + relative], cwd=REPO)
    assert b"\r\n" not in blob
    variants = {"git-lf": sha(blob), "git-checkout-crlf": sha(blob.replace(b"\n", b"\r\n"))}
    assert sha((REPO / relative).read_bytes()) in variants.values()
    for result, method in ((local_controlled, "sourceSha256"), (controlled, "sourceSha256")):
        if role in result[method]:
            assert result[method][role] in variants.values()
    if role == "binding":
        assert local_binding["sourceMethodSha256"] in variants.values()
        assert binding["sourceMethodSha256"] in variants.values()
    source_rows.append({"role": role, "path": relative, **variants})
assert capture["passed"] and capture["controlledSelf"] and not capture["boundGpuObservation"]
assert capture["targetPid"] == capture["selfPid"] and capture["rootPid"] is None
assert capture["probeBytes"] == 8 * 1024 * 1024 and capture["probeFreed"]
assert capture["matchingProbeAllocationEvents"] == capture["matchingAllocationStacks"] == 1
assert capture["readerJoined"] and capture["ownedSessionAbsent"]
assert capture["retainedSelfGenerationUnchanged"] and capture["retainedTargetGenerationUnchanged"]
assert capture["captureError"] is None and capture["consumerError"] is None and not capture["cleanupErrors"]
assert capture["omittedRecords"] == capture["callbackErrors"] == 0
assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
assert capture["nativeStop"]["StoppedOrAlreadyAbsent"]
assert not any(capture[k] for k in ("stopProcessingUsed", "globalSampleIntervalExplicitlyWritten",
                                  "genericSessionStopInvoked", "systemWideEtlWritten",
                                  "unrelatedEventPayloadsSerialized", "rawAllocationAddressSaved",
                                  "targetMemoryContentsSaved", "nativeDesktopStarted"))
events = (ci / "controlled-memory/capture/owned-memory.events.jsonl").read_bytes()
rows = [json.loads(line) for line in events.splitlines()]
assert len(rows) == capture["records"] and len(events) == capture["serializedBytes"]
assert len(rows) <= 4096 and len(events) <= 4 * 1024 * 1024
assert all(row["pid"] == capture["selfPid"] for row in rows)
probes = [r for r in rows if r["kind"] == "virtual-allocation" and r["length"] == capture["probeBytes"]]
assert len(probes) == 1
probe = probes[0]
stacks = [r for r in rows if r["kind"] == "allocation-stack" and r["tid"] == probe["tid"]
          and r["eventRelativeMilliseconds"] == probe["relativeMilliseconds"]]
assert len(stacks) == 1 and stacks[0]["frameCount"] > 0
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "source": SOURCE, "runId": run["id"], "jobId": job["id"], "actualCheckout": checkout,
    "artifactId": artifact["id"], "artifactSha256": download["artifactSha256"],
    "artifactBytes": download["artifactBytes"], "zipCrcVerified": True,
    "localOrdinaryOuterChecks": 10, "ciAdministratorOuterChecks": 15, "compiledGuardChecks": 20,
    "bindingChecksEach": 18, "entryRefusalChecks": 8, "sourceFiles": source_rows,
    "controlledSelfRecords": len(rows), "controlledSelfBytes": len(events),
    "probeAllocationStackPairs": 1, "controlledSelfSessionCleaned": True,
    "wholeCiStatusAtFrozenRead": run["status"], "wholeCiConclusionAtFrozenRead": run["conclusion"],
    "wholeCiAdmitted": False, "actualProductGpuBindingExecuted": False,
    "hostProductStarted": False, "hostTracingStarted": False,
    "productAllocationCauseAttributed": False, "originalRssFailureCleared": False,
    "fullProductGatePassed": False, "ciRerunRequested": False,
}
if "--verify" in sys.argv:
    saved = read(ROOT / "method-admission.json")
    assert {k: v for k, v in result.items() if k != "atUtc"} == {
        k: v for k, v in saved.items() if k != "atUtc"}
else:
    with (ROOT / "method-admission.json").open("x", encoding="utf-8") as out:
        json.dump(result, out, ensure_ascii=False, indent=2)
        out.write("\n")
print(json.dumps(result))
