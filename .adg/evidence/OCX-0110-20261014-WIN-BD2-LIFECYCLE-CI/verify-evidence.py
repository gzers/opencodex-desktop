"""Validate new head CI original bytes, preserving the pending record boundary."""
import gzip
import hashlib
import json
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
merge = "16c781ea3e4abee9e7a7bf89efe38f3a24b187a9"
result = read("ci-final-result.json")
assert result["source"] == source and result["base"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert result["originalProductAndWorkflowUnchanged"] and result["localPureChecksCoveredByPriorLifecycleEvidence"] == 53
assert len(result["ci"]) == 2
for row in result["ci"]:
    run_id = row["runId"]
    assert row["head"] == source and row["status"] == "completed" and row["conclusion"] == "success"
    assert row["actualTreeEqualsHead"] and row["tree"] == "0aa5440ba89a4c31c6239b2fbe372e81a269f1b3"
    assert row["actualCheckout"] == (source if row["event"] == "push" else merge)
    assert row["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    assert all(row["jobs"][name] == "success" for name in
               ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"])
    assert row["jobs"]["build"] == "skipped"
    assert not row["liveCollectorOrPureCSharpTestsRunByCi"]
    terminal = read(f"ci-terminal-{run_id}.json")
    assert terminal["run"]["head_sha"] == source and terminal["run"]["conclusion"] == "success"
    assert terminal["run"]["event"] == row["event"] and terminal["run"]["status"] == "completed"
    summary = read(f"ci-log-summary-{run_id}.json")
    archive = root / f"ci-original-logs-{run_id}.zip"
    raw = archive.read_bytes()
    assert len(raw) == row["logArchiveBytes"] == summary["archiveBytes"]
    assert hashlib.sha256(raw).hexdigest() == row["logArchiveSha256"] == summary["archiveSha256"]
    assert summary["zipCrcPassed"] and summary["safeEntries"] and not summary["failedTestLines"]
    with zipfile.ZipFile(archive) as bundle:
        assert bundle.testzip() is None
        for entry in bundle.infolist():
            path = PurePosixPath(entry.filename.replace("\\", "/"))
            assert not path.is_absolute() and ".." not in path.parts
        assert len(summary["selectedOriginalLogs"]) == 4
        for selected in summary["selectedOriginalLogs"]:
            original = bundle.read(selected["member"])
            saved = gzip.decompress((root / selected["savedAs"]).read_bytes())
            assert original == saved and len(saved) == selected["bytes"]
            assert hashlib.sha256(saved).hexdigest() == selected["sha256"]
    assert not summary["collectorRecordOrScopeTestsExecutedByCi"] and not summary["originalBd2FailedRunCleared"]
    assert not summary["authorizationHeaderForwardedAcrossHosts"]
for key in ["originalTestOrNodeExecutedThisStep", "administratorRecordingApproved", "etwRecorded",
            "originalFailureReproduced", "originalFailureClosed", "nativeInstanceStarted",
            "newCandidateOrManualCiDispatched", "goalScopeNarrowed", "fullVersionGatePromoted", "releaseAccepted"]:
    assert result[key] is False
before, after = read("pr5-before-ci-writeback.json"), read("pr5-after-ci-writeback.json")
assert before["head"]["sha"] == after["head"]["sha"] == source
assert before["base"]["sha"] == after["base"]["sha"]
assert after["number"] == 5 and after["draft"] and after["state"] == "open"
assert not after["merged"] and not after["auto_merge"]
assert before["title"] == after["title"] and after["body"] == (root / "pr5-final-body.md").read_text(encoding="utf-8")
local = read("local-current-boundary.json")
assert local["localDesktopProcesses"] == local["officialNodeProcesses"] == []
for key in ["recordingOutputExists", "administratorRecordingApprovalReceived", "recordApprovedInvoked",
            "foregroundWindowAuthorizedNow", "formalMacNativeHandoffIssued"]:
    assert local[key] is False
assert local["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert local["protectedSandboxSha256"] == {
    "preferences.json": "8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a",
    "data-root.json": "8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25",
    "backup-cleanup-policy.json": "85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11",
    "update-schedule.json": "045991fb0a7a71604632e5c9255c3fe008e8e5f21c7e28feabe0d4178bc4ffd2",
}
dependencies = read("current-execution-dependencies.json")
assert dependencies["currentCollectorHead"] == source and len(dependencies["nextActions"]) == 6
assert not dependencies["fullGateStatesPromoted"] and not dependencies["allVersionRequirementsCompleted"]
assert not dependencies["ciGreenClearsHistoricalFailure"] and not dependencies["historicalPerformanceFailedCleared"]
assert dependencies["nextActions"][0]["approvedNow"] is False
assert dependencies["nextActions"][1]["mayAutoStartForeground"] is False
files = sorted((path for path in root.rglob("*") if path.is_file() and path.name not in
                {"manifest.json", "archive-verification.json"}), key=lambda path: path.relative_to(root).as_posix())
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert {item["path"]: item for item in read("manifest.json")} == {item["path"]: item for item in manifest}
report = {"archiveIntegrity": "passed", "files": len(files), "source": source,
          "exactHeadCiSuccessCount": 2, "actualCheckoutTreesMatched": True,
          "windowsPassedPerRun": 627, "macPassedPerRun": 900,
          "collectorRecorded": False, "adminApprovalPending": True,
          "historicalFailureClosed": False, "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
