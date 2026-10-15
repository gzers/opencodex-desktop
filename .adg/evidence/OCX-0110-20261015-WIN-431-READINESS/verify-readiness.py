"""Verify candidate identity, reuse limits and read-only environment preparation."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
source = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
admission = read("candidate-admission.original.json")
assert admission["source"] == source
assert admission["ciRun"]["id"] == 38705201081 and admission["ciRun"]["head_sha"] == source
assert admission["ciRun"]["event"] == "push" and admission["ciRun"]["status"] == "completed"
assert admission["ciRun"]["conclusion"] == "success" and admission["ciRun"]["run_attempt"] == 1
assert admission["priorCandidateRuns"]["workflow_runs"] == []
assert admission["priorCandidateRuns"]["total_count"] == 0
assert admission["remoteRef"]["object"]["sha"] == source
for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
    matches = [job for job in admission["ciJobs"]["jobs"] if job["name"] == name]
    assert len(matches) == 1 and matches[0]["conclusion"] == "success"
attempt = read("candidate-dispatch-attempt.json")
assert attempt["source"] == source and attempt["candidateRequestsAllowed"] == 1
assert not attempt["automaticRetryAllowed"] and not attempt["etwRerunRequested"]
assert not attempt["hostProductStarted"] and not attempt["hostInstallerExecuted"] and not attempt["releaseRequested"]
assert read("candidate-dispatch-result.json")["httpStatus"] == 204
assert read("candidate-dispatch-payload.json")["inputs"]["purpose"] == "candidate"
assert read("candidate-run-binding.json")["runId"] == 38709701621
terminal = read("candidate-terminal.json")
assert terminal["source"] == terminal["run"]["head_sha"] == source
assert terminal["run"]["id"] == 38709701621 and terminal["run"]["run_attempt"] == 1
assert terminal["run"]["event"] == "workflow_dispatch"
assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
assert next(job for job in terminal["jobs"]["jobs"] if job["name"] == "query-etw")["conclusion"] == "skipped"
package = next(job for job in terminal["jobs"]["jobs"] if job["name"] == "package")
assert package["conclusion"] == "success"
for name in ("Windows regression with the package lock", "Build NSIS and MSI candidates",
             "Install, verify final EXE and uninstall on isolated runner"):
    assert next(step for step in package["steps"] if step["name"] == name)["conclusion"] == "success"
download = read("candidate-download-result.json")
assert download["source"] == source and download["runId"] == 38709701621
assert not download["hostProductStarted"] and not download["hostInstallerExecuted"] and not download["releaseRequested"]
assert len(download["artifacts"]) == 2
for row in download["artifacts"]:
    item = next(item for item in terminal["artifacts"]["artifacts"] if item["id"] == row["artifactId"])
    assert row["apiDigestAndSafePathsAndCrcVerified"] and not row["crossHostCredentialsForwarded"]
    assert row["bytes"] == item["size_in_bytes"] and item["digest"] == "sha256:" + row["sha256"]
    archive_path = Path(row["zipPath"])
    assert archive_path.stat().st_size == row["bytes"] and sha(archive_path.read_bytes()) == row["sha256"]
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for entry in row["entries"]:
            data = archive.read(entry["path"])
            assert len(data) == entry["bytes"] and sha(data) == entry["sha256"]
receipt = read("installer-verification.raw.original.json")
environment = read("candidate-environment.raw.original.json")
assert receipt["sourceCommit"] == environment["source"] == source
assert receipt["result"] == "pass" and receipt["version"] == "0.1.10"
assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
assert {row["kind"] for row in receipt["installers"]} == {"msi", "nsis"}
for row in receipt["installers"]:
    assert row["result"] == row["firstScreen"] == "pass" and row["uninstallRemovedExecutable"]
    assert row["installExitCode"] == row["uninstallExitCode"] == 0
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
logs = read("candidate-log-verification.json")
log_zip = root / "candidate-job-logs.original.zip"
assert logs["actualCheckout"] == logs["source"] == source and logs["runId"] == 38709701621
assert logs["rustResults"] == {"suites": 35, "passed": 627, "failed": 0, "ignored": 7}
assert sha(log_zip.read_bytes()) == logs["archiveSha256"] and log_zip.stat().st_size == logs["archiveBytes"]
with zipfile.ZipFile(log_zip) as archive:
    assert archive.testzip() is None
    for row in logs["selectedLogs"]:
        data = gzip.decompress((root / row["file"]).read_bytes())
        assert data == archive.read(row["member"]) and len(data) == row["bytes"] and sha(data) == row["sha256"]
extraction = read("msi-readonly.original.json")
msi = next(row for row in receipt["installers"] if row["kind"] == "msi")
assert extraction["result"] == "pass" and extraction["databaseMode"] == "read-only"
assert not extraction["installerExecuted"] and extraction["packageSha256"] == msi["sha256"]
assert extraction["properties"]["ProductVersion"] == "0.1.10"
executable = next(row for row in extraction["extractedExecutables"] if row["name"] == "opencodex-desktop.exe")
identity = read("candidate-payload-identity.json")
assert identity["source"] == source and identity["applicationSourceLiteralFound"]
assert identity["sha256"] == executable["sha256"] == msi["installedExecutableSha256"]
assert identity["bytes"] == executable["bytes"] and identity["version"] == "0.1.10"
assert not identity["applicationExecuted"] and not identity["hostInstallerExecuted"]
result = subprocess.run([sys.executable, "-B", "-X", "utf8", str(root / "audit-reuse.py")],
                        capture_output=True, text=True, check=True)
assert json.loads(result.stdout)["historicalFilesHashVerified"] == 8
reuse = read("historical-reuse-analysis.json")
assert reuse["uiSchedulerBlobUnchanged"] and reuse["backendScheduleBlobsChanged"]
assert not reuse["oldSourceExecutionRelabelledAsCurrentSource"] and not reuse["fullCurrentNativeErrorPathAccepted"]
assert not reuse["continuousSingleProcessTwentyFourHoursRequired"]
assert not reuse["betaSixHourSuccessfulBaselineExistsInSelectedEvidence"]
public = read("beta-source-prerequisite.json")
metadata = next(row for row in public["observations"] if row["kind"] == "public-metadata")
assert metadata["statusCode"] == 404
assert not public["bothEndpointsObserved404"]  # Earlier API transport error stays an error.
beta = read("beta-api-followup.json")
assert beta["statusCode"] == 404 and beta["authenticated"]
assert not beta["releaseOrSourceChanged"] and not beta["productCheckInvoked"] and not beta["betaSixHourStarted"]
for filename, item in (("beta-public-metadata-body.original.txt", metadata),
                       ("beta-api-followup-body.original.txt", beta)):
    data = (root / filename).read_bytes()
    assert len(data) == item["bodyBytes"] and sha(data) == item["bodySha256"]
host = read("environment-readiness.original.json")
assert host["source"] == source and host["codeWorktreeClean"] and host["nativeCount"] == 0
assert host["nativeProcesses"] == [] and not host["vmUsabilityProven"]
assert host["hyperVModuleAvailable"] and host["vmmsStatus"] == "Running" and host["hypervisorPresent"]
assert not host["vmInventory"]["querySucceeded"] and host["vmInventory"]["virtualMachineCount"] is None
assert host["vmInventory"]["errorType"] == "VirtualizationException"
assert not host["windowsSandboxExecutablePresent"] and not host["isolatedAcceptanceAccountExists"]
assert host["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
for field in ("accountOrProfileCreated", "vmCreatedOrStarted", "systemConfigurationOrAclChanged",
              "productStarted", "foregroundInputIssued", "hostInstallerExecuted", "fullVersionGatePassed", "releaseRequested"):
    assert host[field] is False
post = read("final-protection.original.json")
assert post["source"] == source and post["nativeCount"] == 0 and post["codeWorktreeClean"]
assert post["originalTaskSha256"] == host["originalTaskSha256"]
assert post["dailyExeSha256"] == host["dailyExeSha256"] and post["protectedSandboxHashes"] == host["protectedSandboxHashes"]
assert not post["localProductOrOriginalLibraryOrFixedNodeStarted"] and not post["hostInstallerExecuted"]
assert not post["newAccountOrProfileCreated"] and not post["macNativeHandoffPerformed"]
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source,
    "candidateRun": 38709701621, "candidateSuccessful": True, "sameSourceCheckoutVerified": True,
    "artifactsApiDigestVerified": True, "runnerMsiAndNsisFreshInstallFirstScreenUninstall": "passed",
    "candidateExecutableSha256": executable["sha256"], "candidateSourceLiteralVerified": True,
    "hostProductOrInstallerStarted": False, "nativeCountAtCheckpoint": 0,
    "historicalFilesHashVerified": 8, "historicalStablePanelIntervalsPreserved": 2,
    "oldTimerFloorMs": 1800001.5, "oldExecutionRelabelledAsNewSource": False,
    "betaSourcePublicAndAuthenticatedApi404": True, "betaSixHourPassed": False,
    "profileAccountProvisioned": False, "usableInstallerVmVerified": False,
    "historicalFailureWindowEtwObtained": False, "pairedPerformancePassed": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("verification.json") == report
print(json.dumps(report))
