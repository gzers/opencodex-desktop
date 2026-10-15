"""Verify real installer retention results without accepting unrelated native gates."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "3c51367f1be89abf66f69805d29b2a660a570a2c"
candidate = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
preparation = read(root / "preparation-result.json")
assert preparation["offlineAdmissionTests"] == 6 and preparation["hostExecuteRejected"]
assert not preparation["installerExecutedOnHost"] and not preparation["productLaunched"]
assert not preparation["vmOrAccountCreated"] and preparation["parseErrors"] == []
assert preparation["baselineNsisExpectedExeHashNotInferred"]
assert "Ran 6 tests" in (root / "admission-final.stderr.original.log").read_text(encoding="utf-8-sig")
syntax = read(root / "final-syntax-check.json")
assert syntax["parseErrors"] == [] and not syntax["hostInstallerExecuted"]
assert not syntax["newStandaloneWorkflowRequired"]
provenance = read(root / "source-provenance.json")
assert provenance["harnessSource"] == source and provenance["candidateSource"] == provenance["parent"] == candidate
assert not provenance["productSourceChanged"] and provenance["harnessAndCandidateSourcesKeptSeparate"]
assert provenance["applicationTree"] == "bf4365586d874d1a9268636e5227efad4621bad1"
changed = set(subprocess.check_output(["git", "diff", "--name-only", candidate, source], cwd=code).decode().splitlines())
assert len(changed) == 5 and changed == {row["path"] for row in provenance["files"]}
assert not any(path.startswith(("apps/", "docs/", ".adg/")) for path in changed)
for row in provenance["files"]:
    assert subprocess.check_output(["git", "rev-parse", source + ":" + row["path"]], cwd=code).decode().strip() == row["gitBlob"]
for run_id, event in ((38713455586, "push"), (38713465220, "pull_request")):
    terminal = read(root / f"run-{run_id}-terminal.original.json")
    run = terminal["run"]
    assert run["head_sha"] == source and run["event"] == event and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        found = [job for job in terminal["jobs"]["jobs"] if job["name"] == name]
        assert len(found) == 1 and found[0]["conclusion"] == "success"
    logs = read(root / f"run-{run_id}-log-verification.json")
    archive_path = root / f"run-{run_id}-logs.original.zip"
    assert archive_path.stat().st_size == logs["logs"]["bytes"]
    assert sha(archive_path.read_bytes()) == logs["logs"]["sha256"]
    assert logs["rustResults"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    expected = source if event == "push" else read(root / "pr-actual-tree-verification.json")["prActualCheckout"]
    assert set(logs["actualCheckout"].values()) == {expected}
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for row in logs["selectedLogs"]:
            data = gzip.decompress((root / row["file"]).read_bytes())
            assert data == archive.read(row["member"]) and len(data) == row["bytes"] and sha(data) == row["sha256"]
tree = read(root / "pr-actual-tree-verification.json")
assert tree["headTree"] == tree["prActualTree"] and tree["treesMatch"] and not tree["binaryEqualityInferred"]
dispatch = read(root / "dispatch-attempt.json")
assert dispatch["harnessSource"] == source and dispatch["fixedCandidateSource"] == candidate
assert dispatch["requestsAllowed"] == 1 and dispatch["freshRunnerPerInstallerKind"]
assert not any(dispatch[key] for key in ("automaticRetryAllowed", "desktopOrPerformanceRequested",
                                        "hostInstallerExecuted", "releaseRequested"))
assert read(root / "dispatch-result.json")["httpStatus"] == 204
assert read(root / "dispatch-payload.json")["inputs"] == {
    "purpose": "installer-retention", "admission_run": "38713455586"}
terminal = read(root / "retention-terminal.json")
assert terminal["run"]["head_sha"] == source and terminal["run"]["run_attempt"] == 1
assert terminal["run"]["status"] == "completed"
assert read(root / "retention-run-binding.json")["runId"] == terminal["run"]["id"]
for name in ("package", "query-etw"):
    assert next(job for job in terminal["jobs"]["jobs"] if job["name"] == name)["conclusion"] == "skipped"
retention_logs = read(root / "retention-job-log-verification.json")
assert retention_logs["harnessSource"] == source and retention_logs["runId"] == terminal["run"]["id"]
assert set(retention_logs["actualCheckouts"].values()) == {source}
assert len(set(retention_logs["distinctRunnerIds"])) == 2
archive_path = root / "retention-job-logs.original.zip"
assert archive_path.stat().st_size == retention_logs["archiveBytes"]
assert sha(archive_path.read_bytes()) == retention_logs["archiveSha256"]
with zipfile.ZipFile(archive_path) as archive:
    assert archive.testzip() is None
    for row in retention_logs["entries"]:
        data = gzip.decompress((root / row["savedAs"]).read_bytes())
        assert data == archive.read(row["member"]) and len(data) == row["bytes"] and sha(data) == row["sha256"]
for kind in ("msi", "nsis"):
    saved = root / kind
    metadata = read(saved / "artifact.original.json")
    download = read(saved / "download-verification.json")
    archive_path = saved / "artifact.original.zip"
    assert download["harnessSource"] == source and download["kind"] == kind
    assert metadata["id"] == download["artifactId"] and not metadata["expired"]
    assert metadata["digest"] == "sha256:" + download["artifactSha256"]
    assert archive_path.stat().st_size == metadata["size_in_bytes"] == download["artifactBytes"]
    assert sha(archive_path.read_bytes()) == download["artifactSha256"]
    assert download["apiDigestSafePathsAndCrcVerified"] and not download["executionRepeated"]
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for row in download["entries"]:
            data = (saved / "original" / row["path"]).read_bytes()
            assert data == archive.read(row["path"]) and sha(data) == row["sha256"] and len(data) == row["bytes"]
result = subprocess.run([sys.executable, "-B", "-X", "utf8", str(root / "analyze-retention.py")],
                        capture_output=True, text=True, check=True)
analysis = read(root / "retention-analysis.json")
assert json.loads(result.stdout) == analysis
assert analysis["harnessSource"] == source and analysis["fixedCandidateSource"] == candidate
post = read(root / "final-protection.original.json")
assert post["source"] == source and post["codeWorktreeClean"] and post["originalBd2ApplicationTreeIdentical"]
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert post["originalTaskBytes"] == 3878 and len(post["protectedSandboxHashes"]) == 4
for name in ("localProductOrOriginalLibraryOrFixedNodeStarted", "localEtwSessionStarted", "localUiInputInjected",
             "hostInstallerExecuted", "newAccountOrProfileCreated", "defenseOrGlobalEnvironmentChanged",
             "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[name] is False
pr = read(root / "pr5-terminal-after.original.json")
payload = read(root / "pr5-terminal-update-payload.json")
assert pr["head"]["sha"] == source and pr["state"] == "open" and pr["draft"]
assert not pr["merged"] and pr["auto_merge"] is None
assert pr["title"] == payload["title"] and pr["body"] == payload["body"]
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "harnessSource": source,
    "fixedCandidateSource": candidate, "offlineAdmissionTests": 6,
    "bothHarnessCiRunsSuccessful": True, "retentionRunId": terminal["run"]["id"],
    "retentionRunConclusion": terminal["run"]["conclusion"],
    "installerResults": analysis["installerResults"],
    "bothInstallerUpgradeAndDefaultUninstallFixtureRetentionPassed":
        analysis["bothInstallerUpgradeAndDefaultUninstallFixtureRetentionPassed"],
    "applicationRootResolutionOrOrdinaryPermissionsAccepted": False,
    "explicitPurgeAccepted": False, "hostInstallerExecuted": False,
    "nativeCountAtCheckpoint": 0, "historicalQueryFailureCleared": False,
    "pairedPerformancePassed": False, "pr5DraftOpenUnmerged": True,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path) == report
print(json.dumps(report))
