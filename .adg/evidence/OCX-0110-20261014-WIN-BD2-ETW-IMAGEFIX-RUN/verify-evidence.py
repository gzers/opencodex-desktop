"""Freeze the completed but lossy trace; never infer original failure from cleanup."""
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
authority = read("launch-request.json")
assert authority["persistentAuthorization"] and authority["userText"] == "批准修正版后续都可以管理员后台 ETW 录制"
assert not authority["unboundedRepeatToGreenAuthorized"] and authority["originalLibraryMaximumRunsThisAttempt"] == 1
assert launcher["controllerIsAdministrator"] and launcher["entryReturned"] and launcher["launcherExitCode"] == 0
assert launcher["controllerPid"] == 22652
assert invocation["plan"]["collectorSource"] == "629b9c6f913b5e04badc9ce747f6bb8ba247fd89"
assert capture["testPid"] == 55592 and capture["testExitCode"] == capture["testExitCodeAfterCleanup"] == 101
assert capture["originalLibraryCompleted"] and capture["ownedRootExited"] and not capture["forcedOuterCleanup"]
assert capture["stdoutSaved"] and capture["stderrSaved"] and capture["rootProcessStartSeen"]
assert capture["processTraceStarted"] and capture["processTraceJoined"] and capture["writerJoined"]
assert capture["ownedSessionStopSucceeded"] and capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert capture["kernelEventsLost"] == 0 and capture["droppedRecords"] == 1339173
assert capture["cleanupErrors"] == [] and capture["decoderCallbackErrors"] == 0
assert capture["captureError"] is capture["runError"] is capture["writerError"] is None
assert capture["boundedOriginalRunCount"] == 1 and not capture["nodePrewarmed"]
assert not capture["globalCpuSampleIntervalWritten"] and not capture["systemWideEtlWritten"]
provenance = read("events-archive-provenance.json")
compressed = (root / "recording-original/capture/owned-etw.events.jsonl.gz").read_bytes()
raw = gzip.decompress(compressed)
assert len(raw) == provenance["sourceBytes"] == 67291856
assert len(compressed) == provenance["compressedBytes"]
assert hashlib.sha256(raw).hexdigest() == provenance["sourceSha256"] == "710bc329c01453fef1f8118eeeef46481fb008e16102429119a3844b8a7554aa"
assert hashlib.sha256(compressed).hexdigest() == provenance["compressedSha256"]
assert provenance["decompressedBytesEqualOriginal"] and provenance["sourcePreserved"]
events = [json.loads(line) for line in raw.splitlines()]
analysis = read("initial-event-analysis.json")
assert len(events) == analysis["rowCount"] == capture["recordedEvents"] == 183278
assert dict(collections.Counter(row["kind"] for row in events)) == analysis["eventKinds"]
assert dict(collections.Counter(str(row["pid"]) for row in events if "pid" in row)) == analysis["pidRecordCounts"]
assert len(raw) > capture["maximumSerializedBytes"] and len(raw) - len(events) <= capture["maximumSerializedBytes"]
assert raw.count(b"\r\n") == len(events)
binding = next(row for row in events if row["kind"] == "root-handle-binding")
assert binding["actualToken"] == {"elevated": True, "elevationType": 2, "pid": 55592}
assert binding["fullLibraryArgumentCount"] == 0 and binding["defaultConcurrency"] and binding["originalCapture"]
starts = [row for row in events if row["kind"] == "process-start"]
assert len(starts) == 8 and sum(row["role"] == "node" for row in starts) == 7
assert not any(row["kind"] == "process-stop" for row in events)
allowed_pids = {row["pid"] for row in starts}
assert all(row.get("pid", capture["testPid"]) in allowed_pids for row in events)
for row in events:
    assert "CommandLine" not in row and "sid" not in row
    if "counterpartThreadIdSaved" in row:
        assert row["counterpartThreadIdSaved"] is False
    if "readyingThreadIdSaved" in row:
        assert row["readyingThreadIdSaved"] is False
test = read("test-result-analysis.json")
assert test["counts"] == {"passed": 529, "failed": 1, "ignored": 4, "filtered": 0}
assert len(test["queryCleanupFixtureResults"]) == 7
assert all(row["result"] == "ok" for row in test["queryCleanupFixtureResults"])
assert test["soleFailure"] == "commands::workspace::tests::local_documents_are_frozen"
assert test["failedAssertion"] == "license.opened"
assert not test["originalQueryFailureReproduced"] and not test["completeEventCoverage"]
stdout = (root / "recording-original/capture/test-stdout.original.log").read_text(encoding="utf-8-sig")
assert "529 passed; 1 failed; 4 ignored;" in stdout and "assertion failed: license.opened" in stdout
assert (root / "recording-original/capture/test-stderr.original.log").read_bytes() == b""
assert invocation["protectedBefore"] == invocation["protectedAfter"]
assert invocation["dailyExeBefore"] == invocation["dailyExeAfter"]
post = read("postcheck.json")
assert post["sessionQueryExitCode"] == 0 and post["ownedSessions"] == post["matchingOwnedProcesses"] == []
assert post["desktopCount"] == post["officialNodeCount"] == post["processesTerminatedByPostcheck"] == 0
assert not post["unrelatedSessionInventorySaved"]
for row in read("recording-small-source-manifest.json"):
    original = (root / "recording-original" / row["path"].replace("\\", "/")).read_bytes()
    assert len(original) == row["bytes"] and hashlib.sha256(original).hexdigest() == row["sha256"]
prepare = read("volume-fix-prepare.stdout.original.txt")
assert prepare["status"] == "passed"
assert sum(prepare[key] for key in ("pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks",
                                  "controllerProcessImageChecks", "descendantHotScopeChecks",
                                  "utf8LineBudgetChecks")) == 78
assert not prepare["etwSessionStarted"] and not prepare["testOrNodeStarted"]
fix = read("volume-fix-boundary.json")
assert fix["source"] == "3a91c175a6c42d1cec16ef3eeb1d799d9abe4aac" and fix["codePushed"]
assert fix["recordingAuthorizationIsStanding"] and not fix["requiresNewPerRunApproval"]
assert not fix["correctedCollectorRecorded"] and not fix["historicalFailureClosed"]
after = read("pr5-after-readback.json")
assert after["head"]["sha"] == fix["source"] and after["draft"] and after["state"] == "open"
assert not after["merged"] and not after["auto_merge"]
assert after["body"] == (root / "pr5-body.md").read_text(encoding="utf-8")
plan = read("descendant-review-only.json")
assert plan["usesStandingUserAuthorization"] and plan["collectorSource"] == fix["source"]
assert plan["originalLibraryRuns"] == 1 and plan["testDeadlineSeconds"] == 120
assert plan["highVolumeEventsScope"] == "admitted Node/cmd descendants only; root lifecycle retained"
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
report = {"archiveIntegrity": "passed", "files": len(files), "originalEvents": len(events),
          "realElevatedTestTokenVerified": True, "originalLibraryCompleted": True,
          "queryFixturesPassed": 7, "targetFailureReproduced": False,
          "captureCoverageComplete": False, "serializedDrops": 1339173, "hardByteLimitExceeded": True,
          "volumeFixSource": fix["source"], "volumeFixLiveRecorded": False,
          "standingBackgroundEtwAuthorization": True, "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
