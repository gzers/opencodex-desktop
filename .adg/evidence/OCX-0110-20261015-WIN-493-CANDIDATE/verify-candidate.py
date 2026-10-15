"""Verify exact-source candidate evidence and local read-only extraction, not version acceptance."""
import gzip
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
digest = lambda data: hashlib.sha256(data).hexdigest()
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
run_id = 38673288645
admission = read("candidate-admission.original.json")
assert admission["source"] == source and admission["ciRun"]["id"] == 38670771363
assert admission["ciRun"]["head_sha"] == source and admission["ciRun"]["event"] == "push"
assert admission["ciRun"]["status"] == "completed" and admission["ciRun"]["conclusion"] == "success"
assert admission["remoteRef"]["object"]["sha"] == source
assert admission["priorCandidateRuns"]["workflow_runs"] == []
assert not admission["hostProductStarted"] and not admission["hostInstallerExecuted"]
assert not admission["releaseRequested"]
attempt = read("candidate-dispatch-attempt.json")
assert attempt["source"] == source and attempt["oneCandidateRequest"]
assert not attempt["automaticDispatchRetryAllowed"] and not attempt["etwRerunRequested"]
assert read("candidate-dispatch-result.json")["httpStatus"] == 204
assert read("candidate-run-binding.json")["runId"] == run_id
terminal = read("candidate-terminal.json")
run = terminal["run"]
assert run["id"] == run_id and run["head_sha"] == source and run["run_attempt"] == 1
assert run["status"] == "completed" and run["conclusion"] == "success" and run["event"] == "workflow_dispatch"
jobs = {row["name"]: row for row in terminal["jobs"]["jobs"]}
assert jobs["package"]["conclusion"] == "success" and jobs["query-etw"]["conclusion"] == "skipped"
for name in ("Windows regression with the package lock", "Build NSIS and MSI candidates",
             "Install, verify final EXE and uninstall on isolated runner",
             "Upload verified Windows installers", "Preserve package evidence"):
    rows = [step for step in jobs["package"]["steps"] if step["name"] == name]
    assert len(rows) == 1 and rows[0]["conclusion"] == "success" and rows[0]["status"] == "completed"
download = read("candidate-download-result.json")
assert download["source"] == source and download["runId"] == run_id
assert not download["hostInstallerExecuted"] and not download["releaseAccepted"]
assert len(download["artifacts"]) == 2
for row in download["artifacts"]:
    artifact = next(value for value in terminal["artifacts"]["artifacts"] if value["id"] == row["id"])
    assert artifact["workflow_run"]["id"] == run_id and artifact["workflow_run"]["head_sha"] == source
    assert not artifact["expired"] and row["digestMatchesApi"] and row["zipCrcPassed"] and row["safeEntries"]
    assert artifact["digest"] == "sha256:" + row["sha256"] and artifact["size_in_bytes"] == row["bytes"]
    assert not row["authorizationForwardedAcrossHosts"]
runner_artifact = next(row for row in download["artifacts"] if row["name"] == "windows-candidate-evidence")
runner_zip = root / "candidate-runner-evidence.original.zip"
assert runner_zip.stat().st_size == runner_artifact["bytes"] and digest(runner_zip.read_bytes()) == runner_artifact["sha256"]
with zipfile.ZipFile(runner_zip) as archive:
    assert archive.testzip() is None
    for item in archive.infolist():
        path = PurePosixPath(item.filename)
        assert not path.is_absolute() and ".." not in path.parts and "\\" not in item.orig_filename
receipt = read("installer-verification.original.json")
assert receipt == read("installer-verification.raw.original.json")
assert receipt["sourceCommit"] == source and receipt["version"] == "0.1.10" and receipt["result"] == "pass"
assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
assert {row["kind"] for row in receipt["installers"]} == {"msi", "nsis"}
for row in receipt["installers"]:
    assert row["result"] == row["firstScreen"] == "pass"
    assert row["uninstallRemovedExecutable"] and row["productVersion"] == "0.1.10"
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
environment = read("candidate-environment.original.json")
assert environment == read("candidate-environment.raw.original.json") and environment["source"] == source
logs = read("candidate-job-log-verification.json")
log_zip = root / "candidate-original-job-logs.zip"
assert logs["runId"] == run_id and logs["actualCheckout"] == source and logs["zipCrcPassed"]
assert log_zip.stat().st_size == logs["originalArchiveBytes"]
assert digest(log_zip.read_bytes()) == logs["originalArchiveSha256"]
with zipfile.ZipFile(log_zip) as archive:
    assert archive.testzip() is None
    for row in logs["selectedOriginalJobLogs"]:
        data = archive.read(row["member"])
        assert len(data) == row["bytes"] and digest(data) == row["sha256"]
        assert gzip.decompress((root / row["savedAs"]).read_bytes()) == data
regression = next(row for row in logs["selectedOriginalJobLogs"]
                  if "Windows regression with the package lock" in row["member"])
text = gzip.decompress((root / regression["savedAs"]).read_bytes()).decode("utf-8-sig")
counts = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
assert len(counts) == 35 and tuple(sum(int(row[i]) for row in counts) for i in range(3)) == (627, 0, 7)
extraction = read("msi-readonly.original.json")
msi = next(row for row in receipt["installers"] if row["kind"] == "msi")
assert extraction["result"] == "pass" and extraction["databaseMode"] == "read-only"
assert not extraction["installerExecuted"] and extraction["packageSha256"] == msi["sha256"]
assert extraction["properties"]["ProductVersion"] == "0.1.10"
exe = next(row for row in extraction["extractedExecutables"] if row["name"] == "opencodex-desktop.exe")
assert exe["sha256"] == msi["installedExecutableSha256"] and exe["productVersion"] == "0.1.10"
identity = read("local-payload-readonly-check.json")
assert identity["source"] == source and identity["applicationSha256"] == exe["sha256"]
assert identity["applicationSourceLiteralFound"] and not identity["applicationExecuted"] and not identity["installerExecuted"]
post = read("candidate-local-postcheck.json")
assert post["source"] == source and post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"] and post["codeWorktreeClean"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not post["localApplicationStarted"] and not post["localInstallerExecuted"] and not post["foregroundInputIssued"]
assert not post["fullVersionGatePassed"] and not post["historicalFailuresCleared"]
beta = read("beta-source-prerequisite.json")
assert beta["source"] == source and beta["bothEndpointsObserved404"]
assert beta["runtimeDefaultsGitBlob"] == "6d3131e4760fc5135fb32ed3ac583adc02240042"
assert len(beta["observations"]) == 2
for row in beta["observations"]:
    body = root / f"beta-{row['kind']}-body.original.txt"
    assert body.stat().st_size == row["bodyBytes"] and digest(body.read_bytes()) == row["bodySha256"]
    assert row["statusCode"] == 404 and not row["followedRedirect"] and not row["credentialOrHeadersSaved"]
assert not beta["productCheckInvoked"] and not beta["beta6hStarted"] and not beta["sourceOrClockChanged"]
assert not beta["betaSourcePublished"] and not beta["realCycleVerified"] and not beta["fullVersionGatePassed"]
manifest_path, report_path = root / "manifest.json", root / "archive-verification.json"
paths = sorted((path for path in root.rglob("*") if path.is_file() and path not in {manifest_path, report_path}),
               key=lambda path: path.relative_to(root).as_posix())
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": digest(path.read_bytes())} for path in paths]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert entries == read(manifest_path.name)
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source, "runId": run_id,
    "sameHeadSuccessfulCiVerified": True, "singleCandidateDispatch": True,
    "candidateSucceeded": True, "actualRunnerCheckoutVerified": True,
    "artifactApiDigestsAndOriginalEvidenceZipVerified": True, "readOnlyMsiPayloadVerified": True,
    "applicationSha256": exe["sha256"], "applicationVersion": "0.1.10",
    "runnerRegressionPassed": 627, "runnerRegressionIgnored": 7, "runnerRegressionSuites": 35,
    "hostInstallerExecuted": False, "hostApplicationStarted": False, "hostNativeInstances": 0,
    "runnerFreshInstallDoesNotProveHostUpgradeOrDataRetention": True,
    "physicalDpiOrDualPermissionUiAccepted": False, "nativeTestProfileAccepted": False,
    "historicalFailureCleared": False, "pairedPerformanceVerified": False,
    "betaApiAndPublicMetadataObserved404": True, "beta6hStarted": False,
    "realCycleVerified": False, "fullVersionGatePassed": False, "releaseAccepted": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path.name) == report
print(json.dumps(report))
