"""Check original exact-head logs and CI admission without promoting live ETW coverage."""
import gzip
import hashlib
import json
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "629b9c6f913b5e04badc9ce747f6bb8ba247fd89"
tree = "30e3be09e903f9c9bc8adf2866f65ed3e85aedec"
result = read("ci-final-result.json")
assert result == read("ci-admission.json")
assert result["source"] == source and result["base"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert len(result["runs"]) == 2 and result["productAndWorkflowUnchanged"]
for key in ("collectorLiveRecordTested", "administratorRecordingApproved", "originalFailureReproduced",
            "originalFailureClosed", "nativeInstanceStarted", "candidateOrCiManuallyDispatched",
            "fullVersionGatePromoted"):
    assert result[key] is False
assert result["newOriginalLibraryRuns"] == 0
scope = read("execution-counter-scope.json")
assert scope["remoteCiRustTestsExecuted"] and scope["remoteCiFinalExeSmokeExecuted"]
assert not scope["remoteCiCollectorRecordExecuted"]
assert scope["newLocalOriginalLibraryStarts"] == scope["newLocalEtwSessions"] == scope["newLocalDesktopInstances"] == 0
assert scope["originalAbortedLocalLibraryStartsRetained"] == 1
assert scope["originalAbortedLocalLibraryCompletionsRetained"] == 0
assert read("observer-result.json")["status"] == "all-terminal"
assert read("observer-result.json")["newTestRuns"] == read("observer-result.json")["newEtwSessions"] == 0
for row in result["runs"]:
    run_id = row["runId"]
    assert row["head"] == source and row["status"] == "completed" and row["conclusion"] == "success"
    assert row["actualTree"] == tree and row["actualTreeEqualsHead"] and row["originalLogsVerified"]
    if row["event"] == "push":
        assert row["actualCheckout"] == source
    commit = read(f"actual-commit-{row['actualCheckout']}.json")
    assert commit["sha"] == row["actualCheckout"] and commit["tree"]["sha"] == tree
    terminal = read(f"ci-terminal-{run_id}.json")
    assert terminal["run"]["head_sha"] == source and terminal["run"]["conclusion"] == "success"
    assert terminal["run"]["event"] == row["event"] and terminal["run"]["status"] == "completed"
    jobs = {job["name"]: job for job in terminal["jobs"]["jobs"]}
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        assert jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success"
    assert jobs["build"]["conclusion"] == "skipped"
    assert any(step["name"] == "Windows final EXE smoke" and step["conclusion"] == "success"
               for step in jobs["backend-windows"]["steps"])
    summary = read(f"ci-log-summary-{run_id}.json")
    archive = root / f"ci-original-logs-{run_id}.zip"
    raw = archive.read_bytes()
    assert len(raw) == row["archiveBytes"] == summary["archiveBytes"]
    assert hashlib.sha256(raw).hexdigest() == row["archiveSha256"] == summary["archiveSha256"]
    assert summary["zipCrcPassed"] and summary["safeEntries"] and not summary["failedTestLines"]
    assert summary["rustCounts"] == row["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7}}
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
assert not local["secondRecordingRootExists"] and not local["newRecordingApprovalReceived"]
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
          "exactHeadCiSuccessCount": 2, "actualCheckoutTreesMatched": True,
          "windowsPassedPerRun": 627, "macPassedPerRun": 900, "collectorLiveRecordTested": False,
          "newRecordingApprovalReceived": False, "originalFailureClosed": False,
          "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
