"""Verify the basename-admission trace and preserve the known CI failure."""
import collections
import gzip
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
capture = read("recording-original/capture/capture-result.json")
launcher = read("launcher-result.json")
invocation = read("recording-original/invocation-result.json")
assert launcher["controllerIsAdministrator"] and launcher["entryReturned"]
assert launcher["launcherExitCode"] == 0 and launcher["controllerPid"] == 65712
assert capture["testPid"] == 73328 and capture["testExitCode"] == 101
assert capture["originalLibraryCompleted"] and not capture["forcedOuterCleanup"]
assert capture["processTraceStarted"] and capture["processTraceJoined"]
assert capture["writerJoined"] and capture["ownedSessionStopSucceeded"]
assert capture["droppedRecords"] == capture["decoderCallbackErrors"] == capture["kernelEventsLost"] == 0
assert capture["knownImageChildrenNotAdmitted"] == 0 and not capture["knownImageCoverageGap"]
assert capture["actualSerializedBytes"] == capture["queuedSerializedBytes"] == 9591262
assert capture["serializedBytesWithinLimit"] and capture["jsonLinesNewline"] == "LF"
assert capture["rootSchedulingEventsIntentionallyOmitted"] == 915533
assert capture["rootFileIoInitiationsIntentionallyOmitted"] == 311295
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert capture["captureError"] is capture["runError"] is capture["writerError"] is None
for row in read("recording-source-manifest.json")["files"]:
    stored = (root / row["savedAs"]).read_bytes()
    restored = gzip.decompress(stored) if row["savedAs"].endswith(".gz") else stored
    assert len(stored) == row["savedBytes"] and hashlib.sha256(stored).hexdigest() == row["savedSha256"]
    assert len(restored) == row["bytes"] and hashlib.sha256(restored).hexdigest() == row["sha256"]
raw = gzip.decompress((root / "recording-original/capture/owned-etw.events.jsonl.gz").read_bytes())
assert len(raw) == capture["actualSerializedBytes"] and b"\r\n" not in raw and not raw.startswith(b"\xef\xbb\xbf")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"] == 28001
starts = [row for row in events if row["kind"] == "process-start"]
stops = [row for row in events if row["kind"] == "process-stop"]
assert len(starts) == len(stops) == 31
assert collections.Counter(row["role"] for row in starts) == {
    "test": 1, "node": 22, "command-basename": 8}
assert {row["pid"] for row in starts} == {row["pid"] for row in stops}
assert all(row["apiIdentityObservation"]["fullImageMatches"] for row in starts
           if row["apiIdentityObservation"]["retained"])
assert all("CommandLine" not in row and "sid" not in row for row in events)
analysis = read("final-event-analysis.json")
assert analysis["knownImageChildrenNotAdmitted"] == 0
assert analysis["allProcessStartsHaveStops"] and analysis["fullQueryFamilyCoverageProven"]
assert analysis["allQueriesPassed"] and not analysis["targetFailureReproduced"]
assert analysis["soleFailure"] == "commands::workspace::tests::local_documents_are_frozen"
assert analysis["failedAssertion"] == "license.opened"
stdout = (root / "recording-original/capture/test-stdout.original.log").read_text(encoding="utf-8-sig")
assert "529 passed; 1 failed; 4 ignored;" in stdout
assert "assertion failed: license.opened" in stdout
assert invocation["protectedBefore"] == invocation["protectedAfter"]
assert invocation["dailyExeBefore"] == invocation["dailyExeAfter"]
post = read("postcheck.json")
assert post["ownedSessions"] == post["matchingOwnedProcesses"] == []
assert post["desktopCount"] == post["officialNodeCount"] == post["processesTerminatedByPostcheck"] == 0
prep = read("basename-admission-prepare.stdout.original.txt")
assert prep["status"] == "passed" and prep["executableAdmissionChecks"] == 13
assert sum(prep[key] for key in (
    "pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks",
    "controllerProcessImageChecks", "descendantHotScopeChecks",
    "utf8LineBudgetChecks", "executableAdmissionChecks")) == 91
assert not prep["etwSessionStarted"] and not prep["testOrNodeStarted"]
after = read("pr5-after-readback.json") if (root / "pr5-after-readback.json").exists() else None
if after:
    assert after["head"]["sha"] == "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
    assert after["draft"] and after["state"] == "open" and not after["merged"]
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
report = {"archiveIntegrity": "passed", "files": len(files), "recordedEvents": 28001,
          "commandBasenameProcesses": 8, "allProcessLifecyclesHaveStops": True,
          "knownImageChildrenNotAdmitted": 0, "bytesWithin64MiB": True,
          "serializedAndKernelDrops": 0, "queryFixturesPassed": 7,
          "targetFailureReproduced": False, "fullQueryFamilyCoverageProven": True,
          "fullRootHotEventCoverage": False, "historicalFailureClosed": False,
          "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
