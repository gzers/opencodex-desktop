"""Verify lifecycle preparation while keeping privileged recording unproven."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
prior = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
old = read("old-order-source-binding.json")
assert old["oldCollectorSource"] == prior and old["traceEventVersion"] == "3.1.21"
for key in ["oldCollectorReadsSourceBeforeKernelEnable", "sourceGetterStartsInactiveSession",
            "kernelEnableRejectsAlreadyStartedSession"]:
    assert old[key] is True
provider = (root / "pinned-provider-source.txt").read_bytes()
assert hashlib.sha256(provider).hexdigest() == old["providerSourceSha256"]
assert b"TraceSampledProfileIntervalInfo" in provider and b"TraceSetInformation(0" in provider
assert not old["actualEtwRunToProveFailurePerformed"] and not old["productOrFixtureDefectClaimed"]
native = read("safe-handle-dispose-source-review.json")
assert hashlib.sha256((root / "pinned-native-source.txt").read_bytes()).hexdigest() == native["sha256"]
assert "CloseTrace(_handle)" in native["fragment"]
checks = read("compiled-final-preparation-checks.json")
assert checks["status"] == "passed"
assert checks["pureScopeChecks"] == 38 and checks["pureLifecycleChecks"] == 7 and checks["pureNativeStopChecks"] == 8
assert checks["digestGuard"] == checks["outputScopeGuard"] == checks["ordinaryPrivilegeGuard"] == "passed"
for key in ["etwSessionStarted", "testOrNodeStarted", "recordingRootCreated", "diagnosticRecordExecutionTested"]:
    assert checks[key] is False
initial_checks = read("compiled-scope-and-lifecycle-checks.json")
assert initial_checks["pureScopeChecks"] == 38 and initial_checks["pureLifecycleChecks"] == 7
result = read("lifecycle-final-result.json")
assert result["collectorSource"] == source and result["oldCollector"] == prior and result["originalTestSource"] == base
assert len(result["changedCodeFiles"]) == 3
assert {item["path"] for item in result["changedCodeFiles"]} == {
    "test/smoke/windows-owned-etw.cs", "test/smoke/windows-owned-etw.ps1",
    "test/smoke/windows-owned-etw.test.ps1",
}
assert result["totalPureChecks"] == 53
for key in ["oldCollectorRuntimeStartupOrderBugConfirmedAgainstPinnedProvider",
            "kernelBeforeSourceGetterFixed", "providerGenericStopWouldWriteGlobalSampleInterval",
            "providerGenericStopAvoidedInNewCollector", "ownNativeStopDoesNotCallTraceSetInformation",
            "backgroundWriterExceptionsCaptured", "cleanupStagesContinueAfterIndependentStageFailure",
            "newRecordingEntryPrepared", "goalActiveAndOriginalFullScopePreserved"]:
    assert result[key] is True
for key in ["actualNativeControlTraceStopExecuted", "actualEtwSessionStarted", "originalTestOrNodeStarted",
            "liveRecordingOrFailureReproductionProved", "decoderDependencyPackageModified",
            "globalEnvironmentChanged", "productOrOriginalFixtureChanged", "prMerged",
            "administratorRecordingApproved", "previousFrozenPreparationAndCiEvidenceModified",
            "originalCiOrMacPerformanceFailureClosed", "fullVersionGatePromoted", "releaseAccepted"]:
    assert result[key] is False
plan = read("record-request-review-only.json")
assert plan["collectorSource"] == source and plan["testSource"] == base
assert plan["originalLibraryRuns"] == 1 and plan["testDeadlineSeconds"] == 120
assert plan["preparationScopeChecks"] + plan["preparationLifecycleChecks"] + plan["preparationNativeStopChecks"] == 53
assert plan["kernelEnabledBeforeSourceGetter"] and plan["nativeStopOnlyUniqueNamedSession"]
assert not plan["traceEventGenericStopInvoked"] and not plan["globalCpuSampleIntervalWritten"]
assert plan["testBinarySha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert plan["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
method = (root / "record-request.ps1").read_text(encoding="utf-8-sig")
assert source in method and "../OCX-0110-20261014-WIN-BD2-CAPTURE-PREPARATION/traceevent-package-provenance.json" in method
assert "if (!$ApprovedByUser)" in method and "taskPrincipal.IsInRole" in method
assert "rev-parse HEAD" in method and "diff --quiet HEAD" in method
pr = read("pr5-after-description-update.json")
assert pr["number"] == 5 and pr["head"]["sha"] == source and pr["draft"] and pr["state"] == "open"
assert not pr["merged"] and not pr["auto_merge"]
assert pr["body"] == (root / "pr-body.md").read_text(encoding="utf-8")
wait = read("ci-wait-status.json")
assert wait["source"] == source and wait["runIds"] == [38561752521, 38561759502]
assert wait["status"] == "bounded-wait-ended-remote-state-not-guessed" and wait["finishedAtUtc"]
assert wait["elapsedSeconds"] == 600
assert not wait["manuallyDispatchedOrRerun"] and not wait["testsOrEtwStarted"]
assert not wait["recordingApprovalInferredFromGoalContinuation"]
readback = read("new-sha-ci-final-readback.json")
assert readback["source"] == source and len(readback["runs"]) == 2
assert not readback["manuallyDispatchedOrRerun"] and not readback["recordApprovedInvoked"]
for run in readback["runs"]:
    assert run["run"]["head_sha"] == source and run["run"]["status"] == "in_progress"
    assert run["run"]["conclusion"] is None
    jobs = {job["name"]: job for job in run["jobs"]["jobs"]}
    assert all(jobs[name]["conclusion"] == "success"
               for name in ["frontend", "backend", "windows-smoke-evidence", "release-tools"])
    assert jobs["backend-windows"]["status"] == "in_progress"
boundary = read("new-code-local-boundary.json")
assert boundary["localDesktopProcesses"] == boundary["officialNodeProcesses"] == []
for key in ["recordingRootExists", "recordingApprovalReceived", "nativeControlTraceStopInvoked",
            "wholeMachineEtlWritten", "systemProfileIntervalChanged", "goalScopeNarrowed"]:
    assert boundary[key] is False
assert boundary["originalLibraryRunCount"] == 0
limits = read("source-basis-and-execution-limits.json")
assert limits["backgroundWriterExceptionGuardCompiledNotFaultInjected"]
assert limits["cleanupContinueAfterStageFailurePureCheckPassed"]
assert not limits["actualKernelEnableOrControlTraceStopProved"] and not limits["adminApprovalReceived"]
assert limits["failedStopOrIncompleteDrainInvalidatesCapture"] and not limits["goalScopeReducedToCollectorOnly"]
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
          "helperDefectsCorrected": 2, "pureChecksPassed": 53, "actualEtwRecorded": False,
          "administratorApprovalPending": True, "newCiFinalSuccessProved": False,
          "ciSnapshotStatus": "in_progress", "historicalFailureClosed": False,
          "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
