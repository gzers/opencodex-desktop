"""Verify the prepared offline bytes; never infer profile or performance acceptance."""
import hashlib
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
source = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
build = read("kit-build.original.json")
kit = Path(build["kitPath"])
manifest_bytes = (root / "kit-manifest.original.json").read_bytes()
assert (kit / "kit-manifest.json").read_bytes() == manifest_bytes
assert sha(manifest_bytes) == build["kitManifestSha256"]
manifest = json.loads(manifest_bytes)
assert build["source"] == manifest["source"] == source
assert manifest["candidateRun"] == 38737628201
assert manifest["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert len(manifest["files"]) == build["kitFiles"] == 663
assert sum(row["bytes"] for row in manifest["files"]) == build["kitBytes"]
actual_paths = {path.relative_to(kit).as_posix() for path in kit.rglob("*") if path.is_file()}
assert actual_paths == {row["path"] for row in manifest["files"]} | {"kit-manifest.json"}
assert len(actual_paths) == len(manifest["files"]) + 1
for entry in manifest["files"]:
    path = kit / entry["path"]
    assert path.resolve().is_relative_to(kit.resolve())
    assert not path.is_symlink() and not path.is_junction()
    data = path.read_bytes()
    assert len(data) == entry["bytes"] and sha(data) == entry["sha256"], entry["path"]
    if entry["path"].startswith("runtime/"):
        assert not path.name.lower().endswith("profile.ps1") and path.name.lower() != "powershell.config.json"
runtime = read("runtime-source.original.json")
assert runtime["signatureStatus"] == "Valid" and "Microsoft Corporation" in runtime["signerSubject"]
assert manifest["runtimeVersion"] == runtime["version"] == "7.6.5"
assert manifest["hostDailySidSha256"] == runtime["hostDailySidSha256"]
assert sha((kit / "runtime/powershell/pwsh.exe").read_bytes()) == runtime["executableSha256"]
assert len([row for row in manifest["files"] if row["path"].startswith("runtime/")]) == 655
assert build["runtimeFilesCopied"] == 655 and build["runtimeProfileAndConfigurationFilesExcluded"]
for name in ("windows-process-token.ps1", "windows-acceptance-profile.ps1", "process-user-sid.ps1"):
    assert (kit / "methods" / name).is_file()
assert (kit / "methods/inspect-profile.ps1").read_bytes() == (root / "methods/inspect-profile.ps1").read_bytes()
expected_main = {
    "baseline019": "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608",
    "candidate0110": "7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9",
}
for label, expected in expected_main.items():
    main, = [row for row in manifest["payloads"][label] if row["path"].endswith("/opencodex-desktop.exe")]
    assert main["sha256"] == expected
    assert sha((kit / main["path"]).read_bytes()) == expected
assert manifest["budgets"] == {
    "startupSamplesPerVersionPerColdAndWarmModeMin": 20,
    "startupP95IncreaseMsMax": 100, "startupP95IncreasePercentMax": 5,
    "idleSecondsPerVersionMin": 600, "cpuIncreasePercentagePointsMax": 0.5,
    "rssIncreaseMiBMax": 10, "mainThreadTaskMsMax": 50,
    "validCacheRoutes": 100, "validCacheRestarts": 10, "automaticQueriesAndQueryChildrenMax": 0,
    "realStablePeriodHours": 24, "realBetaPeriodHours": 6,
}
assert manifest["preflightOnly"] and not manifest["launchPermissionGranted"]
assert manifest["profileApprovalAndFirstLoginRequired"] and manifest["foregroundApprovalRequired"]
assert not manifest["installerOrPerformanceAcceptanceClaimed"]
checks = read("kit-method-checks.original.json")
assert checks["source"] == source and checks["result"] == "passed" and checks["checksPassed"] == 3
assert checks["kitManifestSha256"] == build["kitManifestSha256"]
assert checks["childPolicyMode"] == "process-remotesigned" and checks["parentExecutionPoliciesUnchanged"]
assert checks["parentExecutionPoliciesBefore"] == checks["parentExecutionPoliciesAfter"]
assert checks["copiedRuntimeSignatureStatus"] == "Valid"
assert checks["nativeBeforeCount"] == checks["nativeAfterCount"] == 0
assert checks["sourceMethodSha256"] == sha((root / "methods/inspect-profile.ps1").read_bytes())
assert {row["name"] for row in checks["checks"]} == {
    "daily-profile-refusal", "wrong-manifest-refusal", "changed-payload-refusal"}
for entry in checks["checks"]:
    assert entry["result"] == "passed" and entry["helperOnly"] and not entry["productStarted"]
daily = read("process-remotesigned-daily-profile-refusal.stdout.original.txt")
assert daily["status"] == "rejected" and daily["reason"] == "daily-account" and daily["kitBytesVerified"]
assert daily["sidSha256"] == manifest["hostDailySidSha256"]
assert not daily["rawSidPublished"] and not daily["launchPermissionGranted"]
assert "running scripts is disabled" in (root / "daily-profile-refusal.stderr.original.txt").read_text()
assert not (root / "daily-profile-refusal.stdout.original.txt").read_bytes()
assert "Kit manifest does not match" in (
    root / "process-remotesigned-wrong-manifest-refusal.stderr.original.txt").read_text()
assert "Kit file bytes do not match" in (
    root / "process-remotesigned-changed-payload-refusal.stderr.original.txt").read_text()
for field in ("profilePreparedOrAdmitted", "accountOrProfileCreated", "productStarted",
              "registryOrAclOrKnownFoldersChanged", "performanceMeasured", "fullVersionGatePassed"):
    assert checks[field] is False
protection = read("final-protection.original.json")
assert protection["source"] == source and protection["codeWorktreeClean"] and protection["nativeCount"] == 0
assert protection["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protection["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protection["originalTaskBytes"] == 3878 and len(protection["protectedSandboxHashes"]) == 4
assert not protection["isolatedAcceptanceAccountExists"]
for field in ("localProductOrOriginalLibraryOrFixedNodeStarted", "hostInstallerExecuted",
              "newAccountOrProfileCreated", "localStartupChoiceRegistryWritten",
              "permissionsOrSystemConfigurationChanged", "fullVersionGatePassed", "releaseRequested"):
    assert protection[field] is False
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
report = {
    "offlineKitPreparationEvidence": "passed", "source": source, "manifestFiles": len(entries),
    "kitInventoryFiles": len(manifest["files"]), "kitBytes": build["kitBytes"],
    "kitManifestSha256": build["kitManifestSha256"], "profileMethodRefusalChecks": 3,
    "initialExecutionPolicyFailureRetained": True, "systemAndUserExecutionPoliciesChanged": False,
    "originalBudgetsUnchanged": True, "kitTransferredToTestProfile": False,
    "testAccountOrProfileCreated": False, "nativeCountAtCheckpoint": 0,
    "productStarted": False, "performanceMeasured": False, "fullVersionGatePassed": False,
    "historicalFailureCleared": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries and read("verification.json") == report
print(json.dumps(report))
