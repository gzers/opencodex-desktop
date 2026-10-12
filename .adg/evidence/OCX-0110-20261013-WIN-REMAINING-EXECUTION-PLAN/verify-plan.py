"""Check original budgets and observed prerequisites; never certify execution."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
plan = read("remaining-execution-plan.json")
observed = read("actual-environment-prerequisites.json")
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert plan["sourceCommit"] == observed["sourceCommit"] == source
assert plan["status"] == "reviewable_prerequisite_plan_not_executed"
assert plan["authority"]["originalObjectivePreserved"]
assert not plan["authority"]["originalTaskRecovered"] and plan["authority"]["notAReplacementTaskOrRequiredChecks"]
assert not plan["authority"]["releaseAuthorized"]
assert observed["applicationPid"] == 37276 and observed["desktopApplicationCount"] == 1
assert {item["processId"] for item in observed["retainedProcesses"]} == {37276, 24300, 3920}
assert all(item["actualLiveHandle"] for item in observed["retainedProcesses"])
assert observed["applicationToken"]["elevated"] is False and observed["applicationToken"]["elevationType"] == 3
assert observed["managerDeadlineEpoch"] == 1791903339 and observed["panelDeadlineEpoch"] == 1791903340
assert observed["cycleExecSession"] == 46484 and not observed["cycleAccepted"]
assert not observed["task182LocalExists"]
assert not any(observed[key] for key in ["accountOrProfileCreated", "virtualMachineCreated", "installerExecuted",
    "permissionsChanged", "passwordOrCredentialCollected", "sourceChannelOrClockChanged", "productOrCycleRestarted"])
assert observed["candidatePerformanceUser"] == plan["performanceProfileProposal"]["candidateLocalUserName"] == "ocx0110_accept"
assert observed["proposedPerformanceRoot"] == plan["performanceProfileProposal"]["candidateRoot"] == r"D:\OpenCodex-Acceptance\0.1.10-20261013"
assert not observed["candidatePerformanceUserExists"] and not observed["proposedPerformanceRootExists"]
assert not plan["performanceProfileProposal"]["created"] and plan["performanceProfileProposal"]["candidateOnly"]
assert plan["performanceProfileProposal"]["userDecisionOrProvisioningNeeded"]
assert len(plan["performanceProfileProposal"]["admissionBeforeLaunchingBaseline"]) == 5
assert not observed["currentVmInventoryQueried"] and not observed["vmServiceAvailabilityProvesUsableVm"]
assert all(item["Size"] > item["FreeSpace"] > 0 for item in observed["currentFixedDisks"])
msi = read("final-msi-readonly-identity.json")
assert msi["databaseMode"] == "read-only" and not msi["installerExecuted"]
assert msi["packageSha256"] == observed["finalCandidateMsiSha256"] == "17f70d357f163a881a41b740614b7506a654207041bc2ac3e7ad57717b6c73f6"
assert msi["properties"]["ALLUSERS"] == plan["installerEnvironment"]["candidateMsiAllUsers"] == "1"
assert msi["properties"]["UpgradeCode"] == plan["installerEnvironment"]["candidateAndDailyUpgradeCode"] == "{30FC0B19-5C02-5849-A234-D548E7AFC0DE}"
assert not any(plan["installerEnvironment"][key] for key in ["currentHostInstallationAllowed", "separateInstallDirIsIsolation",
    "separateUserProfileIsInstallerIsolation", "vmMayBeCreatedAutomatically", "vmServiceOrFreeDiskProvesReadyEnvironment",
    "vmMaySubstitutePhysicalDpiOrPhysicalPairedEvidence"])
assert observed["dailyRegistration"]["displayVersion"] == "0.1.9" and observed["dailyRegistration"]["windowsInstaller"] == 1
assert observed["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
script = read("original-pending-script-plan.json")
assert not script["approvalProvided"] and not script["scriptsExecuted"]
assert script["dependencyInstallHook"] == plan["runtimeInstallScriptApproval"]["hook"]
assert script["package"] + "@" + script["version"] == plan["runtimeInstallScriptApproval"]["package"]
assert script["contractQuote"] == "用户在二次确认框显式开启"
assert not any(plan["runtimeInstallScriptApproval"][key] for key in ["approvalProvided", "scriptsExecuted", "automaticGoalContinuationIsApproval"])
original = read("original-baseline-budget-witness.json")
assert original["baselineExecutableSha256"] == plan["performanceProfileProposal"]["baselineExeSha256"]
budgets = plan["performanceProfileProposal"]["unchangedOriginalBudgets"]
original_budgets = original["originalPerformanceContract"]
assert budgets["coldAndHotSamplesPerVersionMinimum"] == original_budgets["coldStartsMinimum"] == original_budgets["hotStartsMinimum"] == 20
assert budgets["startupP95IncreaseMaxMilliseconds"] == original_budgets["p95IncrementMaxMilliseconds"] == 100
assert budgets["startupP95IncreaseMaxPercent"] == original_budgets["p95IncrementMaxPercent"] == 5
assert budgets["bothStartupConditionsRequired"]
assert budgets["idleWindowSecondsMinimum"] == original_budgets["idleMinutes"] * 60 == 600
assert budgets["idleCpuIncrementMaxPercentagePoints"] == original_budgets["idleCpuIncrementMaxPercentagePoints"] == 0.5
assert budgets["idleResidentIncrementMaxMiB"] == original_budgets["rssIncrementMaxMiB"] == 10
assert budgets["mainThreadCheckTaskMustNotExceedMilliseconds"] == original_budgets["mainThreadTaskMaxMilliseconds"] == 50
assert budgets["freshCacheRouteEntries"] == original_budgets["cacheRoutes"] == 100
assert budgets["freshCacheRestarts"] == original_budgets["cacheRestarts"] == 10
assert budgets["freshCacheAutomaticNetworkAndQueryProcesses"] == original_budgets["unexpiredCacheAutoNetworkOrQuerySubprocessMax"] == 0
timed = plan["remainingTimedAndSourceGates"]
assert timed["currentStablePanelDeadlineUtcPlus8"] == ["2026-10-13T22:55:39+08:00", "2026-10-13T22:55:40+08:00"]
assert not any(timed[key] for key in ["stablePanel24hAccepted", "finalSourceErrorFloor30minAccepted",
    "errorFloorFixtureMayMutateCurrentLiveSchedule", "beta6hStarted", "beta6hAccepted", "createOrPublishBetaSourceIsAuthorized"])
assert timed["betaApiAndDownloadStatus"] == [404, 404]
assert plan["physicalUi"]["requiredPhysicalScalesPercent"] == [100, 125, 150]
assert not plan["physicalUi"]["browserOrInterfaceZoomMayReplacePhysicalDpi"]
assert not any(plan["gateBoundaries"][key] for key in ["existingMacosPerformanceFailureCleared",
    "recentNativeBackupSubsetsReplaceCompleteGate", "formalReceiptOrCertificateCreated", "fullGoalComplete", "releaseAccepted"])
assert plan["gateBoundaries"]["oneTestDesktopAtATime"] and plan["gateBoundaries"]["dailyInstallationAndRealDataPreserved"]
source_files = [
    ("final-msi-readonly-identity.json", "OCX-0110-20261012-WIN-VERSION-PROBE-INTEGRATED/msi-identity.json"),
    ("original-pending-script-plan.json", "OCX-0110-20261012-WIN-NPM-RETEST/install-script-confirmation-plan.json"),
    ("original-baseline-budget-witness.json", "OCX-0110-20261012-WIN-BASELINE-PREFLIGHT/official-baseline-safety-and-budget.json"),
]
for local, original_path in source_files:
    assert (root / local).read_bytes() == (root.parent / original_path).read_bytes()
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "plan-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".msi", ".zip"} for p in files)
entries = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("plan-verification.json", {
    "result": "consistent-with-observed-prerequisites-and-original-budgets",
    "scope": "reviewable remaining execution plan, not executed gates or reconstructed TASK",
    "sourceCommit": source, "filesVerified": len(entries),
    "currentLifecycleLive": True, "newAccountOrVmCreated": False,
    "separateProfileDoesNotIsolateMachineWideInstaller": True,
    "originalBudgetsPreserved": True, "formalTaskRecovered": False,
    "scriptApprovalProvided": False, "pairedPerformanceAccepted": False,
    "fullNativeGateAccepted": False, "realCyclesAccepted": False,
    "formalReceiptOrCertificate": False, "goalComplete": False, "releaseAccepted": False,
})
print(json.dumps({"planIntegrity": "consistent", "files": len(entries), "originalBudgetsPreserved": True,
                  "planExecuted": False, "goalComplete": False}))
