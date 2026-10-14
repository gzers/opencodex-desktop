"""Verify the aborted recording, without treating successful cleanup as coverage."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
capture = read("recording-original/capture/capture-result.json")
invocation = read("recording-original/invocation-result.json")
launcher = read("launcher-result.json")
source = read("source-fix-boundary.json")
assert launcher["controllerIsAdministrator"] is True and launcher["controllerPid"] == 26048
assert launcher["status"] == "failed-review-required" and launcher["launcherExitCode"] == 1
assert "Root image mismatch." in launcher["error"] and not launcher["entryReturned"]
assert capture["testPid"] == 55376 and capture["boundedOriginalRunCount"] == 1
assert capture["sessionName"] == "OpenCodex-BD2-15be5566681e4798928fec4a58e27a73"
assert not capture["processTraceStarted"] and not capture["processTraceJoined"]
assert not capture["rootProcessStartSeen"] and capture["recordedEvents"] == 0
assert capture["forcedOuterCleanup"] and capture["testExitCode"] is None
assert capture["ownedSessionStopSucceeded"] and capture["writerJoined"]
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert capture["cleanupErrors"] == [] and capture["kernelEventsLost"] is None
assert capture["captureError"] is None and capture["decoderCallbackErrors"] == 0
assert (root / "recording-original/capture/owned-etw.events.jsonl").read_bytes() == b""
assert not (root / "recording-original/capture/test-stdout.original.log").exists()
assert not (root / "recording-original/capture/test-stderr.original.log").exists()
assert invocation["plan"]["collectorSource"] == "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
assert invocation["plan"]["testBinarySha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert invocation["plan"]["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert invocation["protectedBefore"] == invocation["protectedAfter"]
assert invocation["dailyExeBefore"] == invocation["dailyExeAfter"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
post = read("protected-postcheck.json")
assert {key: value for key, value in post.items() if key != "dailyExe"} == invocation["protectedBefore"]
assert post["dailyExe"] == invocation["dailyExeBefore"]
qualified = read("process-postcheck-qualified.json")
assert qualified["matchingOwnedProcesses"] == []
assert qualified["desktopCount"] == qualified["officialNodeCount"] == 0
assert qualified["processesTerminatedByThisCheck"] == 0
assert qualified["originalRawPostcheckPreserved"]
for observed in qualified["reusedPidObservations"]:
    assert not observed["sameCreation"] and not observed["sameImage"]
    assert not observed["classifiedAsOwned"] and observed["actionTaken"] == "none"
raw_postcheck = read("process-postcheck.json")
assert len(raw_postcheck["ownedOrDesktopProcesses"]) == 1
assert raw_postcheck["ownedOrDesktopProcesses"][0]["ProcessId"] == launcher["controllerPid"]
assert raw_postcheck["ownedOrDesktopProcesses"][0]["ExecutablePath"] != launcher["controllerImage"]
assert read("initial-verification-failure.json")["result"] == "failed"
assert read("owned-session-postcheck.json")["ownedSessionEntries"] == []
assert not read("owned-session-postcheck.json")["unrelatedSessionNamesSaved"]
assert source["collectorFix"] == "629b9c6f913b5e04badc9ce747f6bb8ba247fd89" and source["sourceClean"]
assert source["totalOriginalLibraryStartsThisAuthorization"] == 1
assert source["totalOriginalLibraryCompletionsThisAuthorization"] == 0
assert not source["fixedCodeRealRecordingExecuted"] and not source["originalQueryFailureReproduced"]
assert source["newOriginalRunApprovalRequired"] and not source["globalGitIdentityChanged"]
prepare = read("imagefix-prepare.stdout.original.txt")
assert prepare["status"] == "passed"
assert (prepare["pureScopeChecks"], prepare["pureLifecycleChecks"], prepare["pureNativeStopChecks"],
        prepare["controllerProcessImageChecks"]) == (38, 7, 8, 5)
assert prepare["ordinaryPrivilegeGuard"] == "passed"
assert not prepare["etwSessionStarted"] and not prepare["testOrNodeStarted"]
assert not prepare["recordingRootCreated"] and not prepare["diagnosticRecordExecutionTested"]
for row in read("recording-source-manifest.json")["files"]:
    relative = row["path"].replace("\\", "/")
    path = root / "recording-original" / relative
    assert path.is_relative_to(root) and ".." not in Path(relative).parts
    raw = path.read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
pr = read("pr5-after-readback.json")
assert pr["head"]["sha"] == source["collectorFix"] and pr["draft"] and pr["state"] == "open"
assert not pr["merged"] and not pr["auto_merge"]
assert pr["body"] == (root / "pr5-body.md").read_text(encoding="utf-8")
ci = read("imagefix-ci-initial.json")
assert ci["source"] == source["collectorFix"] and not ci["manuallyDispatched"]
assert all(row["run"]["head_sha"] == source["collectorFix"] for row in ci["runs"])
plan = read("imagefix-review-only.json")
assert plan["collectorSource"] == source["collectorFix"]
assert plan["requiresNewExplicitApproval"] and plan["previousAuthorizedOriginalStarts"] == 1
assert plan["originalLibraryRuns"] == 1 and plan["testDeadlineSeconds"] == 120
assert (plan["originalQuerySeconds"], plan["originalCleanupSeconds"],
        plan["originalObserverSeconds"], plan["originalWatchdogSeconds"]) == (2, 4, 5, 8)
assert "../OCX-0110-20261014-WIN-BD2-IMAGEFIX-CI/ci-admission.json" in (
    root / "record-request-imagefix.ps1").read_text(encoding="utf-8-sig")
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
result = {"archiveIntegrity": "passed", "files": len(files), "controllerElevationVerified": True,
          "originalLibraryStarts": 1, "originalLibraryCompletions": 0,
          "originalQueryResultKnown": False, "eventConsumerStarted": False, "recordedEvents": 0,
          "successfulCleanupIsNotCoverage": True, "originalFailureClosed": False,
          "correctedCollectorRecorded": False, "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
