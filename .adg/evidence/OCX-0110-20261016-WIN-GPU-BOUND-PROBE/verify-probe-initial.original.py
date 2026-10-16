"""Verify controlled binding evidence without promoting native acceptance."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


proof = read("code-provenance.json")
source = "768e231811a2363b32f5fa67c8e74cbc31d751e1"
assert proof["source"] == source and proof["pushedAndReadBack"] and proof["appsTreeUnchanged"]
assert proof["etwRemoteSourcePreserved"] == "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
for item in proof["originalCopies"]:
    data = (ROOT / item["path"]).read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
for name, expected in proof["sourceBlobs"].items():
    data = subprocess.check_output(["git", "show", f"{source}:{name}"], cwd=PROJECT)
    assert hashlib.sha256(data).hexdigest() == expected
launch, snapshot = read("probe/controlled-launch.original.json"), read("probe/controlled-snapshot.original.json")
stop, result = read("probe/controlled-exit.original.json"), read("probe/result.json")
assert snapshot["rootPid"] == launch["pid"] == stop["pid"]
assert snapshot["rootCreationUtc"] == launch["creationTimeUtc"]
assert snapshot["exeSha256"] == launch["exeSha256"]
token = snapshot["actualRootToken"]
assert token["status"] == "observed" and token["schemaVersion"] == 1
assert token["processId"] == launch["pid"] and token["creationTimeUtc"] == launch["creationTimeUtc"]
assert token["executable"] == launch["executable"]
assert isinstance(token["elevated"], bool) and token["elevationType"] in (1, 2, 3)
members = {(m["pid"], m["creationUtc"]) for m in snapshot["members"]}
assert len(members) == len(snapshot["members"]) >= 1
assert sum(m["pid"] == launch["pid"] and m["role"] == "controlled-root"
           for m in snapshot["members"]) == 1
assert all(m["workingSetBytes"] > 0 for m in snapshot["members"])
assert snapshot["controlledSnapshotOnly"] and not snapshot["desktopRootVerified"]
assert snapshot["retainedHandlesUsed"] and snapshot["snapshotOnly"]
assert not snapshot["rawCommandLinesSaved"] and not snapshot["unrelatedProcessRowsSerialized"]
assert not snapshot["uiInputInjected"] and not snapshot["processesStartedOrStopped"]
assert not snapshot["individualGlBuffersAttributed"] and not snapshot["formalIdleSpanMeasured"]
assert not snapshot["fullPerformanceGatePassed"]
assert snapshot["gpuCounterReadStatus"] in ("available", "unavailable-class")
if snapshot["gpuCounterReadStatus"] == "unavailable-class":
    assert snapshot["gpuCounterInstances"] is None
else:
    assert all(row["pid"] in {m[0] for m in members} for row in snapshot["gpuCounterInstances"])
assert snapshot["missingCounterInstancesAreNotZero"]
assert result["status"] == "passed" and result["checks"] == 37
assert result["controlledSnapshotExecutionTested"] and result["controlledNormalExit"]
assert not result["productStarted"] and not result["nativeSnapshotExecutionTested"]
assert stop["exitObserved"] and stop["exitCode"] == 0 and not stop["forcedTermination"]
cleanup = read("controlled-cleanup-readback.json")
assert cleanup["allCapturedControlledGenerationsGone"] and cleanup["nativeCount"] == 0
assert {(m["capturedOwnedPid"], m["capturedCreationUtc"])
        for m in cleanup["ownedGenerations"]} == members
assert all(not m["sameGenerationStillRunning"] and not m["forcedTermination"]
           for m in cleanup["ownedGenerations"])
binding = read("ci-binding.json")
assert binding["source"] == source and binding["runId"] == 38864269467 and binding["runAttempt"] == 1
ci = read("ci-terminal.json" if (ROOT / "ci-terminal.json").exists() else "ci-progress.json")
assert ci["source"] == source and ci["runId"] == binding["runId"]
job, = [j for j in ci["jobs"] if j["name"] == "windows-smoke-evidence"]
assert job["status"] == "completed" and job["conclusion"] == "success"
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        data = (ROOT / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
print(json.dumps({
    "status": "verified-controlled-binding-only", "source": source,
    "localChecks": result["checks"], "controlledExitCode": 0,
    "ownedGenerationsAfterCleanup": 0,
    "gpuCounterReadStatus": snapshot["gpuCounterReadStatus"],
    "gpuInstanceCount": None if snapshot["gpuCounterInstances"] is None
        else len(snapshot["gpuCounterInstances"]),
    "missingGpuRowsInterpretedAsZero": False, "ciRunId": binding["runId"],
    "ciStatus": ci["status"], "ciConclusion": ci["conclusion"],
    "sourceCiRawLogsFullyVerified": False, "nativeSnapshotAccepted": False,
    "individualGlBuffersAttributed": False, "fullPerformanceGatePassed": False,
    "historicalFailureCleared": False, "releaseAccepted": False,
}, indent=2))
