"""Freeze read-only transition prerequisites; never accept cycle or product gates."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
dependency = read(root / "method-dependencies.json")
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
guard = (root / "audit-transition.py").read_text(encoding="utf-8")
assert "--external-snapshot" in guard and 'Source archive is frozen' in guard
assert "subprocess.CREATE_NO_WINDOW" in guard
assert 'existingCycleAuditorSha256' in guard
assert not any(value in guard for value in ["SetForegroundWindow", "TerminateProcess", "stop.request", "__TAURI_INTERNALS__"])
contract = (root / "transition_contract.py").read_text(encoding="utf-8")
assert 'value.get("queryError") != 87' in contract
tests = read(root / "transition-contract-self-test.json")
assert tests["result"] == "passed" and len(tests["cases"]) == 14
assert all(item["passed"] and item["nativeEvidence"] is False for item in tests["cases"])
assert not tests["nativeCycleAccepted"] and not tests["processTransitionExecuted"]
for folder, expected_count in [
    ("actual-background-20261013-1455", 192),
    ("actual-background-20261013-1458", 193),
]:
    path = root / folder
    initial_raw = (path / "native-initialization.json").read_bytes()
    baseline_raw = (path / "schedule-baseline.json").read_bytes()
    assert hashlib.sha256(initial_raw).hexdigest() == dependency["originalInitializationSha256"]
    assert hashlib.sha256(baseline_raw).hexdigest() == dependency["originalBaselineSha256"]
    audit = read(path / "transition-audit.json")
    assert audit["status"] == "pending_original_owners_or_zero_instance"
    assert audit["currentLifecycleSource"] == source and audit["observationsAudited"] == expected_count
    assert len(audit["blockingConditions"]) == 10
    assert audit["independentWholeHostDesktopInventory"]["count"] == 1
    assert audit["independentWholeHostDesktopInventory"]["items"][0]["ProcessId"] == 37276
    assert {value["pid"] for value in audit["actualOriginalProcessHandles"]} == {37276, 24300, 3920}
    assert all(value["actualLiveHandle"] and value["exactCreationMatched"] for value in audit["actualOriginalProcessHandles"])
    assert audit["monitorStatus"] == "observing-real-deadlines" and audit["supervisorStatus"] == "running"
    assert audit["monitorFinishedAtUtc"] is None and audit["supervisorFinishedAtUtc"] is None
    assert not (path / "native-cleanup.json").exists()
    for key in ["automaticProcessTransitionAllowed", "foregroundAuthorityGranted",
                "candidateNativeSourceMatchedByThisOldCycle", "realFullCycleAccepted",
                "fullNativeGateAccepted", "pairedPerformanceAccepted", "formalTaskReceipt", "releaseAccepted"]:
        assert audit[key] is False
    observations = [json.loads(line) for line in (path / "observations-prefix.ndjson").read_bytes().splitlines() if line.strip()]
    assert len(observations) == expected_count
    assert all(item["originalLifecycleLive"] and item["native"]["sourceCommit"] == source for item in observations)
    prefix = read(path / "cycle-prefix-audit.json")
    assert prefix["status"] == "pending_original_deadlines" and not prefix["realFullCycleAccepted"]
    assert prefix["observationsAudited"] == expected_count
    monitor = read(path / "monitor-state.json")
    assert monitor["sourceCommit"] == source and monitor["failedObservations"] == 0
    assert monitor["managerDeadlineEpoch"] == 1791903339 and monitor["panelDeadlineEpoch"] == 1791903340
    assert monitor["maximumHours"] == 26 and not monitor["fullCycleAccepted"]
    preferences = read(path / "disk-preferences.json")
    assert preferences["updates"]["app_update_channel"] == "stable"
    assert preferences["updates"]["app_update_auto_check"] is True
environment = read(root / "environment-readmission.json")
assert not environment["getVmSucceeded"] and environment["getVmError"]
assert not environment["proposedTestAccountExists"] and not environment["proposedTestRootExists"]
for key in ["testProfileOrAccountCreated", "vmCreatedOrStarted", "permissionsChanged",
            "safeInstallerEnvironmentAdmitted", "official019ProfileIsolationAdmitted", "pairedPerformanceAccepted"]:
    assert environment[key] is False
prerequisites = read(root / "prerequisite-disposition.json")
assert not prerequisites["originalTaskPresent"] and not prerequisites["taskReconstructed"]
assert not prerequisites["scriptApprovalReceived"] and not prerequisites["candidateCurrentlyStarted"]
assert prerequisites["candidateSourceCommit"] == "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert prerequisites["candidateExeSha256"] == "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read(root / "manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "toolOnlyCases": 14,
    "actualReadOnlySamples": 2, "latestOriginalObservationsAudited": 193,
    "scope": "transition prerequisite guard, tool-only refusal cases, actual original owner/whole-host process snapshots and unchanged outstanding prerequisites",
    "transitionExecuted": False, "foregroundAuthorityGranted": False, "originalCycleRestarted": False,
    "originalOwnerTerminalAccepted": False, "candidateNativeVerified": False,
    "safeInstallerEnvironmentAdmitted": False, "official019ProfileIsolationAdmitted": False,
    "originalTaskRecovered": False, "runtimeScriptApprovalReceived": False,
    "realFullCycleAccepted": False, "pairedPerformanceAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({
    "archiveIntegrity": "passed", "files": len(entries), "toolOnlyCases": 14,
    "actualTransitionStatus": "pending", "transitionExecuted": False, "fullNativeAccepted": False,
}))
