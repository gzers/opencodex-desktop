"""Verify a completed exact-source package and read-only MSI payload, never the full native gate."""
import hashlib
import gzip
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
subprocess.run([sys.executable, str(root / "verify-preparation.py")], check=True, capture_output=True)
terminal = read("candidate-terminal.json")
run = terminal["run"]
assert run["id"] == 38663871103 and run["head_sha"] == source
assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
package = next(row for row in terminal["jobs"]["jobs"] if row["name"] == "package")
assert package["status"] == "completed" and package["conclusion"] == "success"
assert next(row for row in terminal["jobs"]["jobs"] if row["name"] == "query-etw")["conclusion"] == "skipped"
for name in (
    "Windows regression with the package lock", "Build NSIS and MSI candidates",
    "Install, verify final EXE and uninstall on isolated runner", "Upload verified Windows installers",
    "Preserve package evidence",
):
    matches = [step for step in package["steps"] if step["name"] == name]
    assert len(matches) == 1 and matches[0]["status"] == "completed" and matches[0]["conclusion"] == "success"
download = read("candidate-download-result.json")
assert download["source"] == source and download["runId"] == run["id"]
assert not download["hostInstallerExecuted"] and not download["releaseAccepted"]
assert download["candidateRunStatusWhenRetrieved"] == "completed" and download["candidateRunConclusionWhenRetrieved"] == "success"
assert len(download["artifacts"]) == 2
for row in download["artifacts"]:
    assert row["digestMatchesApi"] and row["zipCrcPassed"] and row["safeEntries"]
    assert not row["authorizationForwardedAcrossHosts"]
    original = next(item for item in terminal["artifacts"]["artifacts"] if item["id"] == row["id"])
    assert original["digest"] == "sha256:" + row["sha256"] and original["size_in_bytes"] == row["bytes"]
    assert original["workflow_run"]["head_sha"] == source and original["workflow_run"]["id"] == run["id"]
evidence_artifact = next(row for row in download["artifacts"] if row["name"] == "windows-candidate-evidence")
evidence_zip = root / "candidate-runner-evidence.original.zip"
assert hashlib.sha256(evidence_zip.read_bytes()).hexdigest() == evidence_artifact["sha256"]
assert evidence_zip.stat().st_size == evidence_artifact["bytes"]
with zipfile.ZipFile(evidence_zip) as archive:
    assert archive.testzip() is None
environment = read("candidate-environment.original.json")
assert environment == read("candidate-environment.raw.original.json")
assert environment["source"] == source
job_logs = read("candidate-job-log-verification.json")
assert job_logs["actualCheckout"] == source and job_logs["zipCrcPassed"]
log_zip = root / "candidate-original-job-logs.zip"
assert log_zip.stat().st_size == job_logs["originalArchiveBytes"]
assert hashlib.sha256(log_zip.read_bytes()).hexdigest() == job_logs["originalArchiveSha256"]
for row in job_logs["selectedOriginalJobLogs"]:
    raw = gzip.decompress((root / row["savedAs"]).read_bytes())
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
regression_log = next(row for row in job_logs["selectedOriginalJobLogs"]
                      if "Windows regression with the package lock" in row["member"])
regression_text = gzip.decompress((root / regression_log["savedAs"]).read_bytes()).decode("utf-8-sig")
counts = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", regression_text)
assert len(counts) == 35 and tuple(sum(int(row[index]) for row in counts) for index in range(3)) == (627, 0, 7)
receipt = read("installer-verification.original.json")
assert receipt == read("installer-verification.raw.original.json")
assert receipt["sourceCommit"] == source and receipt["version"] == "0.1.10" and receipt["result"] == "pass"
assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
assert {row["kind"] for row in receipt["installers"]} == {"msi", "nsis"}
for row in receipt["installers"]:
    assert row["result"] == "pass" and row["firstScreen"] == "pass" and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
    assert row["productVersion"] == "0.1.10"
extraction = read("msi-readonly.original.json")
msi = next(row for row in receipt["installers"] if row["kind"] == "msi")
assert extraction["result"] == "pass" and extraction["databaseMode"] == "read-only"
assert not extraction["installerExecuted"] and extraction["packageSha256"] == msi["sha256"]
assert extraction["properties"]["ProductVersion"] == "0.1.10"
exe = next(row for row in extraction["extractedExecutables"] if row["name"] == "opencodex-desktop.exe")
assert exe["sha256"] == msi["installedExecutableSha256"]
assert exe["productVersion"] == "0.1.10"
identity = read("local-payload-readonly-check.json")
assert identity["applicationSha256"] == exe["sha256"] and identity["applicationSourceLiteralFound"]
assert not identity["applicationExecuted"] and not identity["installerExecuted"]
assert identity["source"] == source
post = read("candidate-local-postcheck.json")
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not post["localApplicationStarted"] and not post["localInstallerExecuted"] and not post["foregroundInputIssued"]
assert not post["fullVersionGatePassed"] and not post["historicalFailuresCleared"]
manifest_path = root / "candidate-manifest.json"
report_path = root / "candidate-verification.json"
paths = sorted((path for path in root.rglob("*") if path.is_file()
                and path.name not in {manifest_path.name, report_path.name}),
               key=lambda path: path.relative_to(root).as_posix())
manifest = [
    {"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
     "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    for path in paths
]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
else:
    assert {row["path"]: row for row in read(manifest_path.name)} == {row["path"]: row for row in manifest}
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(manifest), "source": source, "runId": run["id"],
    "sameSourceCandidateArtifactsVerified": True, "readOnlyMsiPayloadVerified": True,
    "applicationSha256": exe["sha256"], "applicationVersion": "0.1.10",
    "hostInstallerExecuted": False, "hostApplicationStarted": False, "hostNativeInstances": 0,
    "runnerFreshInstallSmokeDoesNotProveHumanUpgradeOrDataRetention": True,
    "actualRunnerCheckoutVerified": True, "runnerRegressionPassed": 627, "runnerRegressionIgnored": 7,
    "physicalDpiOrDualPermissionUiAccepted": False, "historicalFailureCleared": False,
    "fullVersionGatePassed": False, "releaseAccepted": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_path.name)
print(json.dumps(report))
