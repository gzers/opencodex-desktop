"""Verify one changed-condition diagnostic and its exact-source admission."""
from datetime import datetime
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "8e266abac78ff6d1b78cb05e680c7c5dbe1f0e56"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
preparation = read("preparation-result.json")
assert preparation["pythonAdmissionExitCode"] == 0
assert preparation["windowsScriptsParsed"] == 10 and preparation["workflowScriptBlocksParsed"] == 11
assert preparation["parseErrors"] == [] and not preparation["etwSessionStarted"]
assert not preparation["originalLibraryOrNodeStarted"]
tests = (root / "admission-tests.stderr.original.log").read_text(encoding="utf-8-sig")
assert "Ran 11 tests" in tests and tests.rstrip().endswith("OK")
local = read("collector-guards.stdout.original.json")
guard_fields = ("pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks",
                "controllerProcessImageChecks", "descendantHotScopeChecks", "utf8LineBudgetChecks",
                "executableAdmissionChecks", "runnerEnvironmentChecks")
assert sum(local[name] for name in guard_fields) == 101
assert local["status"] == "passed" and local["runnerEnvironmentHostAndEventGuard"] == "passed"
assert local["ordinaryPrivilegeGuard"] == "passed"
assert not local["etwSessionStarted"] and not local["testOrNodeStarted"] and not local["recordingRootCreated"]
provenance = read("preparation-source.json")
assert provenance["source"] == source and provenance["parent"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert provenance["originalBd2ApplicationTreeIdentical"]
assert provenance["applicationTree"] == "bf4365586d874d1a9268636e5227efad4621bad1"
assert not provenance["productSourceChanged"] and not provenance["originalQueryFixtureChanged"]
assert not provenance["originalLibraryOrNodeStarted"] and not provenance["etwStarted"]
expected_paths = {
    ".github/workflows/windows-candidate.yml", "test/smoke/windows-etw-ci-admit.py",
    "test/smoke/windows-owned-etw.cs", "test/smoke/windows-owned-etw.ps1",
    "test/smoke/windows-owned-etw.test.ps1", "test/test_windows_etw_ci.py",
}
assert {row["path"] for row in provenance["changedFiles"]} == expected_paths
changed = set(subprocess.check_output(
    ["git", "diff", "--name-only", provenance["parent"], source], cwd=code).decode().splitlines())
assert changed == expected_paths
for row in provenance["changedFiles"]:
    blob = subprocess.check_output(["git", "show", source + ":" + row["path"]], cwd=code)
    # Working-copy SHA may differ from Git LF on Windows. Both identities are
    # independently retained; a file hash is never mistaken for a Git blob ID.
    actual_blob = subprocess.check_output(["git", "hash-object", "--stdin"], input=blob, cwd=code).decode().strip()
    assert actual_blob == row["gitBlob"]
assert read("source-remote-readback.json")["sha"] == source
tree = read("pr-actual-tree-verification.json")
assert tree["treesMatch"] and tree["headTree"] == tree["prActualTree"]
assert not tree["binaryEqualityInferred"]
for run_id, event in ((38699864356, "push"), (38699869579, "pull_request")):
    terminal = read(f"run-{run_id}-terminal.original.json")
    run = terminal["run"]
    assert run["head_sha"] == source and run["event"] == event and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert terminal["jobs"]["total_count"] == len(terminal["jobs"]["jobs"])
    for name in ("frontend", "backend", "backend-windows", "windows-smoke-evidence", "release-tools"):
        jobs = [job for job in terminal["jobs"]["jobs"] if job["name"] == name]
        assert len(jobs) == 1 and jobs[0]["conclusion"] == "success"
    logs = read(f"run-{run_id}-log-verification.json")
    archive_path = root / f"run-{run_id}-logs.original.zip"
    assert archive_path.stat().st_size == logs["logs"]["bytes"]
    assert sha(archive_path.read_bytes()) == logs["logs"]["sha256"]
    assert logs["safeEntriesAndCrcVerified"] and not logs["historicalFailureCleared"]
    assert logs["rustResults"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    expected_checkout = source if event == "push" else tree["prActualCheckout"]
    assert set(logs["actualCheckout"].values()) == {expected_checkout}
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for row in logs["selectedLogs"]:
            data = gzip.decompress((root / row["file"]).read_bytes())
            assert data == archive.read(row["member"])
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
attempt = read("dispatch-attempt.json")
assert attempt["source"] == source and attempt["originalLibraryRunsAllowed"] == 1
assert not attempt["previousSuccessfulReplayRepeatedUnchanged"]
for key in ("fixtureDeadlinesAndDefaultConcurrencyChanged", "automaticMutationRetryAllowed",
            "artificialStressOrPrewarm", "desktopOrInstallerRequested", "historicalFailureCleared", "releaseRequested"):
    assert attempt[key] is False
assert read("dispatch-result.json")["httpStatus"] == 204
assert read("dispatch-payload.json")["inputs"] == {
    "purpose": "query-etw", "admission_run": "38699864356", "diagnostic_path": "post-build-runner-temp"}
assert read("diagnostic-handle.json")["runId"] == 38702455024
fixed = read("fixed-local-inputs.json")
assert fixed["nodeAuthenticodeStatus"] == "Valid" and "OpenJS" in fixed["nodeSigner"]
assert not fixed["fixedNodeExecuted"] and not fixed["originalLibraryExecuted"]
assert len(fixed["files"]) == 4
original = read("fixed-original-run.original.json")
artifact = read("fixed-original-artifact.original.json")
assert original["id"] == 38537166560 and original["conclusion"] == "failure"
assert artifact["id"] == 11881042603 and not artifact["expired"]
assert artifact["digest"] == "sha256:5f42b60d09e802b640b183cc1043f6a4aab067b4d1714082ad9fe8a4dcac5688"
result = subprocess.run([sys.executable, "-B", "-X", "utf8", str(root / "analyze-diagnostic.py")],
                        capture_output=True, text=True, check=True)
analysis = read("diagnostic-analysis.json")
assert json.loads(result.stdout) == analysis
assert analysis["libraryExitCode"] == 0 and analysis["tests"] == {"passed": 530, "failed": 0, "ignored": 4}
assert not analysis["targetQueryFailureReproduced"] and not analysis["actualFailingQueryExecutionWindowEtwObtained"]
assert analysis["eventBytes"] == 6089687 and analysis["processStarts"] == analysis["processStops"] == 31
assert sum(analysis["eventKinds"].values()) == 16865
download = read("diagnostic-download-verification.json")
metadata = read("diagnostic-artifact.original.json")
assert download["artifactId"] == metadata["id"] == 11957286829
archive = root / f"diagnostic-artifact-{metadata['id']}.original.zip"
assert sha(archive.read_bytes()) == download["artifactSha256"] == metadata["digest"].removeprefix("sha256:")
assert archive.stat().st_size == download["artifactBytes"] == metadata["size_in_bytes"]
with zipfile.ZipFile(archive) as zip_archive:
    assert zip_archive.testzip() is None
assert len(download["entries"]) == 14 and download["safePathsAndCrcVerified"]
pr = read("pr5-terminal-after.original.json")
payload = read("pr5-terminal-update-payload.json")
assert pr["head"]["sha"] == source and pr["title"] == payload["title"] and pr["body"] == payload["body"]
assert pr["state"] == "open" and pr["draft"] and not pr["merged"] and pr["auto_merge"] is None
post = read("final-protection.original.json")
assert post["source"] == source and post["codeWorktreeClean"] and post["originalBd2ApplicationTreeIdentical"]
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert post["originalTaskBytes"] == 3878 and len(post["protectedSandboxHashes"]) == 4
for key in ("isolatedAcceptanceAccountExists", "windowsSandboxExecutablePresent",
            "localProductOrOriginalLibraryOrFixedNodeStarted", "localEtwSessionStarted", "localUiInputInjected",
            "hostInstallerExecuted", "newAccountOrProfileCreated", "defenseOrGlobalEnvironmentChanged",
            "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[key] is False
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": sha(path.read_bytes())} for path in sorted(root.rglob("*"))
           if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source,
    "localAdmissionTests": 11, "pureCollectorChecks": 101,
    "exactCiRuns": [38699864356, 38699869579], "bothCiRunsSuccessful": True,
    "actualCheckoutAndOriginalLogBytesVerified": True, "diagnosticRun": 38702455024,
    "changedConditionSingleOriginalLibraryExecution": True,
    "originalLibrary": {"passed": 530, "failed": 0, "ignored": 4, "exitCode": 0},
    "targetFailureReproduced": False, "actualFailureWindowEtwObtained": False,
    "originalFailuresPreserved": True, "rootCauseDetermined": False,
    "recordedEvents": 16865, "eventBytes": 6089687,
    "processStartsAndStops": [31, 31], "captureLossOrDecodeErrors": 0,
    "fullRootHotEventCoverage": False, "versionSpecificWaitSemanticsVerified": False,
    "productSourceChanged": False, "nativeInstancesAtCheckpoint": 0,
    "profileAndInstallerEnvironmentStillMissing": True,
    "pr5DraftOpenUnmerged": True, "fullVersionGatePassed": False,
    "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("verification.json") == report
print(json.dumps(report))
