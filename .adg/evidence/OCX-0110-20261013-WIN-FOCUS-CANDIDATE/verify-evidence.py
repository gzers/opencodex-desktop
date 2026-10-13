"""Verify candidate admission, real workflow/ZIP receipts and read-only local EXE identity."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
native = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
msi_sha = "6a500186d89c8a2d73216c5ac7b47b9ab27372938f1ea0048f95c6abc11f5573"
nsis_sha = "09b2526cfd2bc15ff17da753d8ce4bf5d03c71833cd61f31df3abecb4a570e4c"
exe_sha = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
data_hashes = {
    "preferences.json": "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa",
    "data-root.json": "8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25",
    "backup-cleanup-policy.json": "85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11",
    "update-schedule.json": "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20",
}
admission = read("candidate-admission.json")
assert admission["sourceCommit"] == source and admission["exactCiRunId"] == 38352525092
assert admission["exactCiConclusion"] == "success" and admission["requiredCiJobsPassed"]
assert admission["existingExactSourceRuns"] == 0 and admission["dispatchRequested"]
assert admission["dispatchStatusCode"] == 204 and not admission["existingRunReused"]
assert admission["remoteSourceMatchedBeforeDispatch"] and admission["remoteSourceMatchedAfterDispatch"]
assert not admission["mainIntegrated"] and not admission["releasePublished"] and not admission["localExecutableStarted"]
assert read("ci-admission-run.json")["head_sha"] == source and read("ci-admission-run.json")["conclusion"] == "success"
for name in ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]:
    assert next(job for job in read("ci-admission-jobs.json")["jobs"] if job["name"] == name)["conclusion"] == "success"
for filename in ["remote-feature-ref-before.json", "remote-feature-ref-after.json"]:
    assert read(filename)["object"]["sha"] == source
for filename in ["candidate-run-terminal.json", "candidate-run-audited-terminal.json"]:
    run = read(filename)
    assert run["id"] == 38355803684 and run["head_sha"] == source and run["conclusion"] == "success"
    assert run["status"] == "completed" and run["event"] == "workflow_dispatch"
    assert run["workflow_id"] == 376008755 and run["head_branch"] == "feature/0.1.10-update-center-focus"
for filename in ["candidate-jobs-terminal.json", "candidate-jobs-audited-terminal.json"]:
    jobs = read(filename)["jobs"]
    assert len(jobs) == 1 and jobs[0]["id"] == 115158292064 and jobs[0]["name"] == "package"
    assert jobs[0]["conclusion"] == "success" and all(step["conclusion"] == "success" for step in jobs[0]["steps"])
log = (root / "candidate-package-terminal.log").read_text(encoding="utf-8-sig")
results = [tuple(map(int, match)) for match in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)]
assert len(results) == 34 and results[0] == (505, 0, 4)
assert tuple(sum(value[index] for value in results) for index in range(3)) == (601, 0, 7)
observer = read("candidate-observer-terminal.json")
assert observer["runId"] == 38355803684 and observer["sourceCommit"] == source and observer["conclusion"] == "success"
assert observer["actualRunSourceVerified"] and observer["observationErrors"] == 0
history = [json.loads(line) for line in (root / "candidate-observations.ndjson").read_text(encoding="utf-8-sig").splitlines()]
assert history and history[-1]["status"] == "completed"
assert all(item["observerPid"] == 10164 and item["sourceCommit"] == source and item["runId"] == 38355803684 for item in history)
download = read("candidate-download-audit.json")
assert download["runConclusion"] == "success" and download["packageConclusion"] == "success"
assert len(download["artifactDownloads"]) == 2 and all(item["digestMatched"] for item in download["artifactDownloads"])
expected_zips = {
    11792974377: "747fb1415a5dd41ddaa3e8eb457b6e83b1a1e28872b4020386fd3e598c4089d0",
    11792959449: "2e8639d5921a58499d73edc4ec1a8ee7dcbae1ff2169d3045d042ba5040b7d9f",
}
assert {item["artifactId"]: item["zipSha256"] for item in download["artifactDownloads"]} == expected_zips
content = read("candidate-content-audit.json")
assert content["result"] == "passed" and content["sourceCommit"] == source and content["runId"] == 38355803684
assert content["artifactZipDigestsAndCrcVerified"] and content["installerReceiptCopiesByteIdentical"]
assert content["runnerInstallUninstallAndFirstScreenAccepted"] and len(content["selectedRawWitnesses"]) == 20
assert {item["kind"]: item["sha256"] for item in content["packages"]} == {"msi": msi_sha, "nsis": nsis_sha}
for item in content["selectedRawWitnesses"]:
    path = root / item["path"]
    assert path.stat().st_size == item["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"]
for value in content["runnerActualApplicationTokens"].values():
    assert value["elevated"] is True and value["elevationType"] == 1
    assert value["mode"] == "any" and not value["specificModeMatched"]
msi = read("msi-readonly.json")
assert msi["result"] == "pass" and msi["databaseMode"] == "read-only" and not msi["installerExecuted"]
assert msi["packageSha256"] == msi_sha and msi["properties"]["ALLUSERS"] == "1"
assert msi["properties"]["UpgradeCode"] == "{30FC0B19-5C02-5849-A234-D548E7AFC0DE}"
assert next(item for item in msi["extractedExecutables"] if item["name"] == "opencodex-desktop.exe")["sha256"] == exe_sha
identity = read("offline-identity-result.json")
assert identity["sourceCommit"] == source and identity["result"] == "passed" and identity["localExtractedExeSha256"] == exe_sha
assert identity["normalizedInMemoryHashMatchesCiBuiltExe"] and identity["bothPackagePatchHashesMatchRunnerReceipts"]
assert identity["ciUnpatchedBuiltExeSha256"] == content["builtExecutableSha256"] == \
    "8e2e9b3e003a136b9b7a5599976607b4aee9ced1c7e26ccc845ffefa45825f6a"
assert identity["embeddedFullCommitPresent"] and not identity["normalizedExecutableWritten"]
assert identity["updateCenterBlob"] == "a5d587f4b7a4828cac9234feb9c7335ba0c38155"
assert identity["updateCenterTestsBlob"] == "3da03dbbd22ac21e05b55b27b7790a794d784dc8"
correction = read("offline-checker-correction.json")
assert correction["actualInstalledMsiMarkerOffsets"] == [7987543, 7987597]
assert correction["ciReportedPatchOffset"] == identity["bundleMarkerOffset"] == 7987597
assert not correction["rawArtifactOrExeChanged"]
signature = read("extracted-authenticode.json")
assert signature["sha256"] == exe_sha and signature["status"] == "NotSigned"
assert not signature["signerCertificatePresent"] and not signature["executableStarted"]
formatter = read("isolated-rustfmt-result.json")
assert formatter["sourceCommit"] == source and formatter["exitCode"] == 0
assert formatter["isolatedFormatterLocated"] and formatter["mode"] == "read-only --check"
assert not formatter["fullLocalCargoBuildAccepted"] and not formatter["newCodeMutationIssued"]
life = read("retained-lifecycle-after-candidate.json")
assert life["sourceCommit"] == native and life["desktopApplicationCount"] == 1
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
cycle = read("monitor-state-after-candidate.json")
assert cycle["status"] == "observing-real-deadlines" and cycle["sourceCommit"] == native
assert cycle["failedObservations"] == 0 and cycle["managerDeadlineEpoch"] == 1791903339
assert cycle["panelDeadlineEpoch"] == 1791903340 and not cycle["fullCycleAccepted"]
disposition = read("candidate-disposition.json")
assert disposition["protectedDataHashes"] == data_hashes and disposition["currentNativeSource"] == native
assert disposition["candidateLocalExeSha256"] == exe_sha and disposition["candidateLocalExePresent"]
assert disposition["candidateCiAndPackageResult"] == "success" and disposition["candidateArtifactIdentityVerified"]
for key in ["candidateLocalExeStarted", "localInstallerInvoked", "foregroundInputIssued", "sourceOrCycleReplaced",
            "clockOrTimerChanged", "mainIntegrated", "originalFailedCiFailureCleared", "originalFailedCiCauseIsolated",
            "localOrdinaryAdminPairAccepted", "localDefaultDataCustomRootAndUninstallAccepted",
            "nativeFocusFixAccepted", "pairedPerformanceAccepted", "realCyclesAccepted", "finalNativeGateAccepted",
            "formalTaskReceipt", "releasePublished", "channelPromoted", "releaseAccepted"]:
    assert disposition[key] is False
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "sourceCommit": source,
    "scope": "candidate admitted after exact CI; real package regression/runner smoke, both ZIP receipts and read-only extracted MSI EXE identity",
    "exactCandidateWorkflowAccepted": True, "runnerFreshInstallerSmokeAccepted": True,
    "localReadOnlyExeIdentityAccepted": True, "localInstallerOrAppStarted": False,
    "mainIntegrated": False, "originalFailedCiFailureCleared": False, "originalFailedCiCauseIsolated": False,
    "localOrdinaryAdminTokenPairAccepted": False, "nativeFocusFixAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releasePublished": False, "releaseAccepted": False,
})
print(json.dumps({
    "archiveIntegrity": "passed", "files": len(entries), "exactCandidateWorkflow": "success",
    "readOnlyMsiExeIdentity": "passed", "localInstallerOrAppStarted": False, "fullNativeAccepted": False,
}))
