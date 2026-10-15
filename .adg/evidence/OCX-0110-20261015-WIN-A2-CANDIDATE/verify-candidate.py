"""Verify one exact-source package and readonly payload; native gates stay open."""
import gzip
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path(__file__).resolve().parent
source = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
run_id = 38729632145
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
admission = read("candidate-admission.original.json")
assert admission["source"] == admission["remoteRef"]["object"]["sha"] == source
assert admission["priorCandidateRuns"]["total_count"] == 0
assert admission["priorCandidateRuns"]["workflow_runs"] == []
for run_key, jobs_key, identifier, event in (
    ("ciRun", "ciJobs", 38726727330, "push"), ("prCiRun", "prCiJobs", 38726732452, "pull_request"),
):
    run = admission[run_key]
    assert run["id"] == identifier and run["event"] == event and run["head_sha"] == source
    assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        job, = [row for row in admission[jobs_key]["jobs"] if row["name"] == name]
        assert job["conclusion"] == "success"
attempt = read("candidate-dispatch-attempt.json")
assert attempt["source"] == source and attempt["candidateRequestsAllowed"] == 1
assert not attempt["automaticRetryAllowed"] and not attempt["etwRerunRequested"]
assert read("candidate-dispatch-result.json")["httpStatus"] == 204
assert read("candidate-dispatch-payload.json")["inputs"]["purpose"] == "candidate"
assert read("candidate-run-binding.json") == {"runId": run_id, "source": source}
terminal = read("candidate-terminal.json")
run = terminal["run"]
assert terminal["source"] == run["head_sha"] == source and run["id"] == run_id
assert run["status"] == "completed" and run["conclusion"] == "success"
assert run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
for name in ("query-etw", "retention"):
    assert next(row for row in terminal["jobs"]["jobs"] if row["name"] == name)["conclusion"] == "skipped"
package = next(row for row in terminal["jobs"]["jobs"] if row["name"] == "package")
assert package["conclusion"] == "success"
for name in ("Windows regression with the package lock", "Build NSIS and MSI candidates",
             "Install, verify final EXE and uninstall on isolated runner"):
    assert next(step for step in package["steps"] if step["name"] == name)["conclusion"] == "success"
download = read("candidate-download-result.json")
assert download["source"] == source and download["runId"] == run_id
assert len(download["artifacts"]) == 2
for row in download["artifacts"]:
    metadata, = [item for item in terminal["artifacts"]["artifacts"] if item["id"] == row["artifactId"]]
    assert metadata["workflow_run"]["head_sha"] == source and metadata["workflow_run"]["id"] == run_id
    assert metadata["digest"] == "sha256:" + row["sha256"] and not metadata["expired"]
    archive_path = Path(row["zipPath"])
    assert archive_path.stat().st_size == metadata["size_in_bytes"] == row["bytes"]
    assert sha(archive_path.read_bytes()) == row["sha256"]
    assert row["apiDigestAndSafePathsAndCrcVerified"] and not row["crossHostCredentialsForwarded"]
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for entry in row["entries"]:
            data = archive.read(entry["path"])
            assert len(data) == entry["bytes"] and sha(data) == entry["sha256"]
receipt = read("installer-verification.raw.original.json")
assert receipt["sourceCommit"] == read("candidate-environment.raw.original.json")["source"] == source
assert receipt["result"] == "pass" and receipt["version"] == "0.1.10"
assert not receipt["codeSigned"] and not receipt["releaseAccepted"]
assert {row["kind"] for row in receipt["installers"]} == {"msi", "nsis"}
for row in receipt["installers"]:
    assert row["result"] == row["firstScreen"] == "pass"
    assert row["installExitCode"] == row["uninstallExitCode"] == 0 and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
logs = read("candidate-log-verification.json")
archive_path = root / "candidate-job-logs.original.zip"
assert logs["source"] == logs["actualCheckout"] == source and logs["runId"] == run_id
assert logs["rustResults"] == {"suites": 35, "passed": 643, "failed": 0, "ignored": 7}
assert sha(archive_path.read_bytes()) == logs["archiveSha256"] and archive_path.stat().st_size == logs["archiveBytes"]
with zipfile.ZipFile(archive_path) as archive:
    assert archive.testzip() is None
    for row in logs["selectedLogs"]:
        data = gzip.decompress((root / row["file"]).read_bytes())
        assert data == archive.read(row["member"]) and sha(data) == row["sha256"]
extraction = read("msi-readonly.original.json")
msi = next(row for row in receipt["installers"] if row["kind"] == "msi")
assert extraction["databaseMode"] == "read-only" and extraction["result"] == "pass"
assert not extraction["installerExecuted"] and extraction["packageSha256"] == msi["sha256"]
assert extraction["properties"]["ProductVersion"] == "0.1.10"
identity = read("candidate-payload-identity.json")
executable = identity["executable"]
assert identity["source"] == source and identity["runId"] == run_id
assert executable["sha256"] == msi["installedExecutableSha256"] == (
    "4295367e8dbf235c0f221d2fff127624faa201d20d6c905af8d7265d305fe44f")
assert executable["bytes"] == 8807936 and executable["productVersion"] == "0.1.10"
data = Path(executable["path"]).read_bytes()
assert sha(data) == executable["sha256"] and source.encode() in data
assert read("candidate-source-literal.json")["applicationSourceLiteralFound"]
for field in ("hostInstallerExecuted", "hostProductStarted", "nativeRootRecoveryPassed",
              "official019HostPairedPerformancePassed", "fullVersionGatePassed", "releaseRequested"):
    assert identity[field] is False
protection = read("final-protection.original.json")
assert protection["source"] == source and protection["codeWorktreeClean"] and protection["nativeCount"] == 0
assert protection["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protection["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protection["originalTaskBytes"] == 3878 and len(protection["protectedSandboxHashes"]) == 4
assert not protection["hostInstallerExecuted"] and not protection["localProductOrOriginalLibraryOrFixedNodeStarted"]
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
report = {
    "candidateIdentityEvidence": "passed", "manifestFiles": len(entries), "source": source, "runId": run_id,
    "dispatchesAllowed": 1, "sameSourceCiAndPackageVerified": True, "artifactsVerified": 2,
    "packageRegression": {"passed": 643, "ignored": 7}, "readonlyExecutableSha256": executable["sha256"],
    "freshRunnerMsiNsisSmoke": "passed", "hostInstallerExecuted": False, "hostProductStarted": False,
    "nativeRootRecoveryAcceptance": "pending", "official019HostPairedPerformance": "pending",
    "historicalFixedArtifactFailureWindowEtwObtained": False, "fullVersionGatePassed": False,
    "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries and read(report_path) == report
print(json.dumps(report))
