"""Validate bounded Windows comparison evidence while preserving the CI failure."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "bd2f6eb10d58115235e35d575507bb69f44666da"
exe_sha = "2709775efbce5a08a02827c61a74abb1adfb3170653898b8c614eebf9c367400"
node_sha = "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
admission = read("test-artifact-admission.json")
assert admission["source"] == source and admission["runId"] == 38537161445
assert admission["testExecutableSha256"] == exe_sha
assert admission["artifactApiSourceMatches"] and admission["runnerTestExecutableHashMatches"]
assert admission["zipCrcPassed"] and admission["independentHeadCheckoutLogMatches"]
assert admission["compiledCommitLiteralPresent"] is False
assert admission["oldFailedPrRun"] == 38537166560 and not admission["oldFailureCleared"]
node = read("official-node-admission.json")
assert node["version"] == "v24.21.0" and node["versionMatchesCi"] and node["officialChecksumMatches"]
assert node["exeSha256"] == node_sha and not node["nodeStartedOrPrewarmed"]
assert not node["npmInstalled"] and not node["lifecycleScriptsExecuted"] and not node["globalNodeOrPathChanged"]
run = read("single-run-result.json")
assert run["source"] == source and run["testBinarySha256"] == exe_sha and run["nodeSha256"] == node_sha
assert run["arguments"] == [] and run["defaultConcurrencyPreserved"] and run["testCapturePreserved"]
assert run["status"] == "single-original-run-ended" and run["testExitCode"] == 101
assert run["testPid"] == 11640 and run["testCreationUtc"] == "2026-10-14T09:33:37.9039668Z"
assert run["rootHandleRetained"] and run["samples"] == 80 and run["processesObserved"] == 7
assert not run["forcedOuterCleanup"] and not run["fileWatcherBufferOverflow"]
assert not run["nodePrewarmed"] and not run["desktopStarted"] and not run["wprOrSchedulingEtwRecorded"]
assert run["protectedSandboxBefore"] == run["protectedSandboxAfter"]
assert run["dailyExeBefore"] == run["dailyExeAfter"] == \
    "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
analysis = read("owned-trace-analysis.json")
assert analysis["source"] == source and analysis["fullLibrary"] == {
    "passed": 529, "failed": 1, "ignored": 4, "filtered": 0, "exitCode": 101,
}
assert len(analysis["queryFixtureCasesPassed"]) == 7
for key in ["queryFixtureFailureReproduced", "originalFailedCiTraceObtained",
            "originalCiFailureCleared", "originalCiRootCauseIdentified", "etwSchedulingOrKernelFileTraceObtained"]:
    assert analysis[key] is False
raw = (root / "owned-process-thread-io.raw.jsonl").read_bytes()
assert hashlib.sha256(raw).hexdigest() == analysis["rawTraceSha256"]
rows = [json.loads(line) for line in raw.decode().splitlines()]
assert len(rows) == 836 and analysis["rawRecordCounts"] == {
    "observer-start": 1, "fixture-file-notification": 274,
    "owned-process-sample": 554, "owned-node-discovered": 6, "observer-finished": 1,
}
assert analysis["wctOutcomeCounts"] == {"True:0": 78, "False:1444": 2}
assert len(analysis["processSummaries"]) == 7
assert analysis["maximumSampleReadMs"] > 1400 and analysis["observerInitializedAfterTestStartedMs"] > 300
assert analysis["nativeBeforeAfter"] == [0, 0] and analysis["sameProtectedSandboxAndDailyBytes"]
census = read("final-owned-census.json")
assert census["ownedOfficialNodeProcesses"] == [] and census["localDesktopProcesses"] == []
assert census["officialNodeSignatureStatus"] == "Valid" and "OpenJS Foundation" in census["officialNodeSigner"]
assert not census["formalMacNativeHandoffIssued"] and not census["wprRecordingStarted"]
goal = read("status-only-resume.json")
assert goal["status"] == "not-resumed-readback-verified" and goal["readback"]["status"] == "paused"
assert goal["originalAccountingPreserved"] and goal["before"] == goal["readback"]
assert not goal["objectiveReplacementIssued"] and not goal["budgetChangeIssued"] and not goal["writerLockOverrideIssued"]
assert "already has an active writer" in goal["setResponse"]["error"]["message"]
scope = read("execution-boundaries.json")
assert scope["wholeLibraryExecutedOnce"] and not scope["oldFailureReproducedLocally"]
assert not scope["rootCauseIdentified"] and not scope["fullVersionGatePromoted"]
assert not scope["wprEtwDeniedByOs"]  # Status was read; no recording start was attempted.
path_failure = read("compiled-workspace-path-diagnosis.json")
assert not path_failure["compiledWorkspaceExistsOnUserHost"] and not path_failure["artificialRunnerPathCreated"]
assert not path_failure["testOrBinaryAltered"] and not path_failure["licenseFailureCountedAsQueryFailure"]
files = sorted(path for path in root.rglob("*") if path.is_file() and
               path.name not in {"manifest.json", "archive-verification.json"})
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == manifest, "Frozen evidence bytes changed"
report = {
    "archiveIntegrity": "passed", "files": len(files), "source": source,
    "originalRunCount": 1, "rawSnapshotRecords": 836, "queryCasesPassedLocally": 7,
    "failedSceneReproduced": False, "schedulerEtwRecorded": False,
    "goalStatus": "paused", "nativeInstanceCount": 0,
    "originalCiFailureClosed": False, "fullVersionGatePromoted": False,
}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
