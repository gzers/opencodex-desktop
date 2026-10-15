"""Verify exact CI terminal/log bytes, not administrator ETW functionality."""
import gzip
import hashlib
import json
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
result = read("ci-review-final-result.json")
assert result["source"] == source and result["productAndOriginalWorkflowBytesUnchangedSinceBd2"]
assert result["administratorApprovalStillPending"] and result["goalAutomaticallyContinuingDoesNotApproveRecording"]
assert result["helperWaitTerminal"] and result["helperWaitExecSession"] == 67983
assert 0 < result["boundedWaitElapsedSeconds"] < 600
assert len(result["ciResults"]) == 2
for run in result["ciResults"]:
    identifier = run["runId"]
    assert run["associatedHead"] == source and run["status"] == "completed" and run["conclusion"] == "success"
    assert run["actualTreeEqualsHelperHead"] and len(run["queryFixtureCasesPassed"]) == 7
    assert run["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    assert run["jobs"]["build"] == "skipped"
    assert not run["recorderOrPureScopeTestsExecutedByCi"]
    terminal = read(f"ci-terminal-{identifier}.json")
    assert terminal["run"]["head_sha"] == source
    assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
    summary = read(f"ci-log-summary-{identifier}.json")
    raw = (root / f"ci-original-logs-{identifier}.zip").read_bytes()
    assert len(raw) == run["archiveBytes"] == summary["archiveBytes"]
    assert hashlib.sha256(raw).hexdigest() == run["archiveSha256"] == summary["archiveSha256"]
    with zipfile.ZipFile(root / f"ci-original-logs-{identifier}.zip") as archive:
        assert archive.testzip() is None
        for entry in archive.infolist():
            path = PurePosixPath(entry.filename.replace("\\", "/"))
            assert not path.is_absolute() and ".." not in path.parts
        assert len(summary["selectedOriginalLogs"]) == 4
        for selected in summary["selectedOriginalLogs"]:
            original = archive.read(selected["member"])
            saved = gzip.decompress((root / selected["savedAs"]).read_bytes())
            assert original == saved and len(saved) == selected["bytes"]
            assert hashlib.sha256(saved).hexdigest() == selected["sha256"]
    assert summary["zipCrcPassed"] and summary["safeEntries"] and not summary["failedTestLines"]
    assert not summary["originalBd2FailedRunCleared"] and not summary["authorizationHeaderForwardedAcrossHosts"]
for key in ["ciManuallyDispatchedOrRerun", "rawEtwRecordedOnUserHost", "originalFailureReproducedOrClosed",
            "earlierPreparationEvidenceModified", "fullVersionGatePromoted", "pr5Merged", "releaseAccepted"]:
    assert result[key] is False
assert result["localNativeInstanceCount"] == 0
pr = read("pr5-current-readback.json")
assert pr["head"]["sha"] == source and pr["number"] == 5 and pr["draft"]
assert pr["state"] == "open" and not pr["merged"] and not pr["auto_merge"]
boundary = read("local-recording-boundary-readback.json")
assert boundary["localDesktopProcesses"] == boundary["ownedOfficialNodeProcesses"] == []
for key in ["recordingOutputRootExists", "explicitRecordingApprovalReceived", "etwRecorderInvoked",
            "ciWatcherProcessStillPresent", "goalContinuationTreatedAsRecordingApproval"]:
    assert boundary[key] is False
wait = read("ci-wait-status.json")
assert wait["status"] == "both-existing-runs-terminal-verified" and wait["finishedAtUtc"]
assert wait["runIds"] == [38555890377, 38556292712]
assert not wait["manuallyDispatchedOrRerun"] and not wait["testsOrEtwStarted"]
assert not wait["recordingApprovalInferredFromGoalContinuation"]
files = sorted((path for path in root.rglob("*") if path.is_file() and path.name not in
                {"manifest.json", "archive-verification.json"}), key=lambda path: path.relative_to(root).as_posix())
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert {item["path"]: item for item in read("manifest.json")} == {item["path"]: item for item in manifest}
report = {"archiveIntegrity": "passed", "files": len(files), "collectorSource": source,
          "ciTerminalSuccessCount": 2, "macTestsPassedPerRun": 900, "windowsTestsPassedPerRun": 627,
          "recorderExecuted": False, "adminApprovalPending": True,
          "oldFailureClosed": False, "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
