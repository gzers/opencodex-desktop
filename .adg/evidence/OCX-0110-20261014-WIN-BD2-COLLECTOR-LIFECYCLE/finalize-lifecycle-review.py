"""Bind actual helper fixes and pure checks; never claim privileged recording."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
old = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout
assert git("rev-parse", "HEAD").decode().strip() == source
files = git("diff", "--name-only", old, source).decode().splitlines()
assert set(files) == {
    "test/smoke/windows-owned-etw.cs", "test/smoke/windows-owned-etw.ps1",
    "test/smoke/windows-owned-etw.test.ps1",
}
assert not git("diff", "--name-only", base, source, "--", "apps", ".github",
               "Cargo.lock", "AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore")
cs = git("show", source + ":test/smoke/windows-owned-etw.cs").decode()
provider = (root / "pinned-provider-source.txt").read_text(encoding="utf-8-sig")
initial = read("old-order-source-binding.json")
assert initial["oldCollectorReadsSourceBeforeKernelEnable"]
assert initial["sourceGetterStartsInactiveSession"] and initial["kernelEnableRejectsAlreadyStartedSession"]
stop_body = provider[provider.index("public bool Stop(bool noThrow = false)"):
                     provider.index("public void Dispose()", provider.index("public bool Stop(bool noThrow = false)"))]
assert "TraceSampledProfileIntervalInfo" in stop_body and "TraceSetInformation(0" in stop_body
assert "ResetWindowsHeapTracingFlags" in stop_body
helper_body = cs[cs.index("public static T KernelBeforeSource<T>"):cs.index("public static List<string> FinishAll")]
assert helper_body.index("enableKernel();") < helper_body.index("return getSource();")
assert "var kernel = OwnedEtwLifecycle.KernelBeforeSource(" in cs
assert "session.Stop(true)" not in cs and "session.Stop(false)" not in cs
assert "session.StopOnDispose = true" not in cs and "session.StopOnDispose = false" in cs
assert "nativeStop = OwnedEtwNativeStop.Stop(sessionName)" in cs
assert "TraceSetInformation(" not in cs
assert "kernelBeforeSourceOrder = true" in cs and "globalCpuSampleIntervalWritten = false" in cs
assert "writerError = error;" in cs and "OwnedEtwLifecycle.FinishAll(new[]" in cs
checks = read("compiled-final-preparation-checks.json")
assert checks["status"] == "passed"
assert checks["pureScopeChecks"] == 38 and checks["pureLifecycleChecks"] == 7 and checks["pureNativeStopChecks"] == 8
assert checks["digestGuard"] == checks["outputScopeGuard"] == checks["ordinaryPrivilegeGuard"] == "passed"
assert not checks["etwSessionStarted"] and not checks["testOrNodeStarted"]
assert not checks["recordingRootCreated"] and not checks["diagnosticRecordExecutionTested"]
plan = read("record-request-review-only.json")
assert plan["collectorSource"] == source and not plan["globalCpuSampleIntervalWritten"]
assert plan["kernelEnabledBeforeSourceGetter"] and plan["nativeStopOnlyUniqueNamedSession"]
assert not plan["traceEventGenericStopInvoked"]
pr = read("pr5-after-description-update.json")
assert pr["head"]["sha"] == source and pr["draft"] and pr["state"] == "open"
assert not pr["merged"] and not pr["auto_merge"]
assert pr["body"] == (root / "pr-body.md").read_text(encoding="utf-8")
native_review = read("safe-handle-dispose-source-review.json")
assert "CloseTrace(_handle)" in native_review["fragment"]
prepared_files = []
for file in files:
    raw = git("show", source + ":" + file)
    prepared_files.append({"path": file, "gitBlob": git("rev-parse", source + ":" + file).decode().strip(),
                           "gitBytesSha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)})
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "oldCollector": old, "collectorSource": source,
    "originalTestSource": base, "changedCodeFiles": prepared_files,
    "oldCollectorRuntimeStartupOrderBugConfirmedAgainstPinnedProvider": True,
    "kernelBeforeSourceGetterFixed": True,
    "providerGenericStopWouldWriteGlobalSampleInterval": True,
    "providerGenericStopAvoidedInNewCollector": True,
    "ownNativeStopDoesNotCallTraceSetInformation": True,
    "backgroundWriterExceptionsCaptured": True,
    "cleanupStagesContinueAfterIndependentStageFailure": True,
    "pureScopeChecks": 38, "pureLifecycleChecks": 7, "pureNativeStopChecks": 8,
    "totalPureChecks": 53,
    "actualNativeControlTraceStopExecuted": False,
    "actualEtwSessionStarted": False, "originalTestOrNodeStarted": False,
    "liveRecordingOrFailureReproductionProved": False,
    "decoderDependencyPackageModified": False, "globalEnvironmentChanged": False,
    "productOrOriginalFixtureChanged": False,
    "draftPr": 5, "prMerged": False,
    "administratorRecordingApproved": False, "newRecordingEntryPrepared": True,
    "previousFrozenPreparationAndCiEvidenceModified": False,
    "goalActiveAndOriginalFullScopePreserved": True,
    "originalCiOrMacPerformanceFailureClosed": False,
    "fullVersionGatePromoted": False, "releaseAccepted": False,
    "initialPrDescriptionReadFailedBeforeMutation": True,
    "nativeStopVerificationScope":
        "x64 structure/layout/own-name checks; actual ControlTrace call remains pending approved recording",
}
(root / "lifecycle-final-result.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"collectorSource": source, "confirmedAndFixedHelperBugs": 2,
                  "pureChecksPassed": 53, "privilegedRecordingExecuted": False}))
