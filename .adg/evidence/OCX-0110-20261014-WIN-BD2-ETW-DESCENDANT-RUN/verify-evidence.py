"""Verify the zero-drop partial family trace; completeness remains unproven."""
import collections
import gzip
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
capture = read("recording-original/capture/capture-result.json")
invocation = read("recording-original/invocation-result.json")
launcher = read("launcher-result.json")
assert launcher["controllerIsAdministrator"] and launcher["entryReturned"] and launcher["launcherExitCode"] == 0
assert capture["testPid"] == 73392 and capture["testExitCode"] == 101
assert capture["boundedOriginalRunCount"] == 1 and capture["originalLibraryCompleted"]
assert not capture["forcedOuterCleanup"] and capture["ownedRootExited"]
assert capture["rootProcessStartSeen"] and capture["processTraceStarted"] and capture["processTraceJoined"]
assert capture["writerJoined"] and capture["ownedSessionStopSucceeded"]
assert capture["droppedRecords"] == capture["decoderCallbackErrors"] == capture["kernelEventsLost"] == 0
assert capture["captureError"] is capture["runError"] is capture["writerError"] is None
assert capture["cleanupErrors"] == []
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert capture["actualSerializedBytes"] == capture["queuedSerializedBytes"] == 2818320
assert capture["actualSerializedBytes"] < capture["maximumSerializedBytes"] == 67108864
assert capture["serializedBytesWithinLimit"] and capture["jsonLinesNewline"] == "LF"
assert capture["hotEventsScope"] == "admitted Node/cmd descendants only"
assert capture["rootSchedulingEventsIntentionallyOmitted"] == 905134
assert capture["rootFileIoInitiationsIntentionallyOmitted"] == 312907
for row in read("recording-source-manifest.json")["files"]:
    stored = (root / row["savedAs"]).read_bytes()
    assert len(stored) == row["savedBytes"] and hashlib.sha256(stored).hexdigest() == row["savedSha256"]
    raw = gzip.decompress(stored) if row["savedAs"].endswith(".gz") else stored
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
raw = gzip.decompress((root / "recording-original/capture/owned-etw.events.jsonl.gz").read_bytes())
assert len(raw) == capture["actualSerializedBytes"] and b"\r\n" not in raw and not raw.startswith(b"\xef\xbb\xbf")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"] == 8673
analysis = read("initial-event-analysis.json")
assert dict(collections.Counter(row["kind"] for row in events)) == analysis["eventKinds"]
starts = [row for row in events if row["kind"] == "process-start"]
stops = [row for row in events if row["kind"] == "process-stop"]
assert len(starts) == len(stops) == 8 and sum(row["role"] == "node" for row in starts) == 7
assert not any(row["role"] == "command" for row in starts)
assert {row["pid"] for row in starts} == {row["pid"] for row in stops}
assert all(row["apiIdentityObservation"]["retained"] and row["apiIdentityObservation"]["exitFileTime100ns"] == 0
           for row in starts)
assert all(abs(row["apiIdentityObservation"]["creationVsOriginalEventDeltaMs"]) < 1 for row in starts)
binding = next(row for row in events if row["kind"] == "root-handle-binding")
assert binding["actualToken"] == {"elevated": True, "elevationType": 2, "pid": 73392}
assert binding["fullLibraryArgumentCount"] == 0 and binding["defaultConcurrency"] and binding["originalCapture"]
assert not analysis["targetFailureReproduced"] and not analysis["allExpectedQueryProcessFamiliesVerified"]
tests = read("test-result-analysis.json")
assert tests["passed"] == 529 and tests["failed"] == 1 and tests["ignored"] == 4
assert len(tests["queries"]) == 7 and all(row["result"] == "ok" for row in tests["queries"])
assert tests["failedAssertion"] == "license.opened" and not tests["targetFailureReproduced"]
assert not tests["fullQueryFamilyCoverageProven"] and not tests["cmdOrQueryDescendantSeen"]
assert invocation["protectedBefore"] == invocation["protectedAfter"]
assert invocation["dailyExeBefore"] == invocation["dailyExeAfter"]
post = read("postcheck.json")
assert post["sessionQueryExitCode"] == 0 and post["ownedSessions"] == post["matchingOwnedProcesses"] == []
assert post["desktopCount"] == post["officialNodeCount"] == post["processesTerminatedByPostcheck"] == 0
fix = read("image-admission-fix-boundary.json")
assert fix["source"] == "0407fdb2328038dbda0d6b5947a00d5a6ecfdb44" and fix["codePushed"]
assert not fix["fixedCollectorRecorded"] and not fix["missedCommandProcessExactCauseProven"]
prepare = read("image-admission-prepare.stdout.original.txt")
assert prepare["status"] == "passed" and prepare["executableAdmissionChecks"] == 11
assert not prepare["etwSessionStarted"] and not prepare["testOrNodeStarted"]
after = read("pr5-after-readback.json")
assert after["head"]["sha"] == fix["source"] and after["draft"] and after["state"] == "open"
assert not after["merged"] and not after["auto_merge"]
assert after["body"] == (root / "pr5-body.md").read_text(encoding="utf-8")
files = sorted((path for path in root.rglob("*") if path.is_file()
                and path.name not in {"manifest.json", "archive-verification.json"}),
               key=lambda path: path.relative_to(root).as_posix())
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert {row["path"]: row for row in read("manifest.json")} == {row["path"]: row for row in manifest}
report = {"archiveIntegrity": "passed", "files": len(files), "recordedEvents": 8673,
          "bytesWithin64MiB": True, "serializedAndKernelDrops": 0,
          "capturedLiveNodeCount": 7, "capturedNodeLifecyclesComplete": True,
          "fullQueryFamilyCoverageProven": False, "originalFailureReproduced": False,
          "imageAdmissionFix": fix["source"], "imageAdmissionFixRecorded": False,
          "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
