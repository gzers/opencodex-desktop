"""Check original exact-head logs and CI admission without promoting live ETW coverage."""
import gzip
import hashlib
import json
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "3a91c175a6c42d1cec16ef3eeb1d799d9abe4aac"
tree = "5a9555c074c3ee72eddd568b03fff514f5cfd19c"
result = read("ci-final-result.json")
assert result == read("diagnostic-admission.json")
assert not (root / "ci-admission.json").exists()
assert result["diagnosticOnly"] and not result["releaseAdmission"] and not result["allCiPassed"]
assert result["newCiFailureRetained"] and result["greenOnlyHelperNotExecuted"]
assert result["source"] == source and result["base"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert len(result["runs"]) == 2 and result["productFixtureWorkflowSharedFilesUnchanged"]
for key in ("collectorLiveRecordTested", "originalFailureClosed", "fullVersionGatePromoted"):
    assert result[key] is False
assert result["newLocalOriginalLibraryRuns"] == 0
assert result["standingUserAuthorization"] == "批准修正版后续都可以管理员后台 ETW 录制"
scope = read("execution-counter-scope.json")
assert scope["remoteCiRustTestsExecuted"] and scope["remoteCiFinalExeSmokeExecuted"]
assert not scope["remoteCiCollectorRecordExecuted"]
assert scope["newLocalOriginalLibraryStarts"] == scope["newLocalEtwSessions"] == scope["newLocalDesktopInstances"] == 0
assert scope["originalAbortedLocalLibraryStartsRetained"] == 1
assert scope["originalAbortedLocalLibraryCompletionsRetained"] == 0
assert all(row["status"] == "completed" for row in result["runs"])
assert read("observer-result.json")["newTestRuns"] == read("observer-result.json")["newEtwSessions"] == 0
for row in result["runs"]:
    run_id = row["runId"]
    expected = "failure" if row["event"] == "push" else "success"
    assert row["head"] == source and row["status"] == "completed" and row["conclusion"] == expected
    assert row["actualTree"] == tree and row["actualTreeEqualsHead"] and row["originalLogsVerified"]
    if row["event"] == "push":
        assert row["actualCheckout"] == source
    commit = read(f"actual-commit-{row['actualCheckout']}.json")
    assert commit["sha"] == row["actualCheckout"] and commit["tree"]["sha"] == tree
    terminal = read(f"ci-terminal-{run_id}.json")
    assert terminal["run"]["head_sha"] == source and terminal["run"]["conclusion"] == expected
    assert terminal["run"]["event"] == row["event"] and terminal["run"]["status"] == "completed"
    jobs = {job["name"]: job for job in terminal["jobs"]["jobs"]}
    for name in ("frontend", "backend", "release-tools", "windows-smoke-evidence"):
        assert jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success"
    assert jobs["build"]["conclusion"] == "skipped"
    assert jobs["backend-windows"]["conclusion"] == expected
    smoke = next(step for step in jobs["backend-windows"]["steps"] if step["name"] == "Windows final EXE smoke")
    assert smoke["conclusion"] == ("skipped" if expected == "failure" else "success")
    summary = read(f"ci-log-summary-{run_id}.json")
    archive = root / f"ci-original-logs-{run_id}.zip"
    raw = archive.read_bytes()
    assert len(raw) == row["archiveBytes"] == summary["archiveBytes"]
    assert hashlib.sha256(raw).hexdigest() == row["archiveSha256"] == summary["archiveSha256"]
    assert summary["zipCrcPassed"] and summary["safeEntries"]
    assert len(summary["failedTestLines"]) == (3 if expected == "failure" else 0)
    assert len(row["failedTests"]) == (3 if expected == "failure" else 0)
    assert row["unexpectedFailedTests"] == []
    assert summary["rustCounts"] == row["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": ({"suites": 1, "passed": 527, "failed": 3, "ignored": 4} if expected == "failure"
                            else {"suites": 35, "passed": 627, "failed": 0, "ignored": 7})}
    assert not summary["collectorRecordOrScopeTestsExecutedByCi"]
    assert not summary["originalBd2FailedRunCleared"] and not summary["authorizationHeaderForwardedAcrossHosts"]
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
before, after = read("pr5-before-ci-writeback.json"), read("pr5-after-ci-readback.json")
assert before["head"]["sha"] == after["head"]["sha"] == source
assert before["base"]["sha"] == after["base"]["sha"] == result["base"]
assert before["title"] == after["title"] and after["state"] == "open" and after["draft"]
assert not after["merged"] and not after["auto_merge"]
assert after["body"] == (root / "pr5-final-body.md").read_text(encoding="utf-8")
local = read("local-final-boundary.json")
assert local["desktopCount"] == local["officialNodeCount"] == 0
assert not local["nextRecordingRootExists"] and local["standingUserAuthorization"]
assert local["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert local["priorAbortedRecordingArchiveVerified"] and local["dailyAndSandboxDigestsUnchanged"]
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
report = {"archiveIntegrity": "passed", "files": len(files), "source": source,
          "exactHeadCiSuccessCount": 1, "exactHeadCiFailureCount": 1, "diagnosticOnly": True, "actualCheckoutTreesMatched": True,
          "windowsPassedPush": 527, "windowsFailedPush": 3, "windowsPassedPr": 627, "macPassedPerRun": 900, "collectorLiveRecordTested": False,
          "standingUserAuthorization": True, "originalFailureClosed": False,
          "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
