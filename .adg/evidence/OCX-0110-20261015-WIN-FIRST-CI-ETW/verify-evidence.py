"""Verify first-execution captures without promoting historical or native gates."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
prep = read(root / "preparation-result.json")
assert prep["pythonAdmissionTests"] == 12 and prep["collectorChecks"] == 108
assert prep["workflowParseErrors"] == [] and prep["runnerHostInvocationRejected"]
assert not prep["originalLibraryOrNodeStarted"] and not prep["etwSessionStarted"] and not prep["cargoExecutedLocally"]
assert "Ran 12 tests" in (root / "admission-tests.stderr.original.log").read_text(encoding="utf-8-sig")
guards = read(root / "collector-guards.stdout.original.json")
assert guards["status"] == "passed" and guards["regressionEventChecks"] == 7
assert sum(guards[name] for name in (
    "pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks", "controllerProcessImageChecks",
    "descendantHotScopeChecks", "utf8LineBudgetChecks", "executableAdmissionChecks",
    "runnerEnvironmentChecks", "regressionEventChecks")) == 108
assert not guards["etwSessionStarted"] and not guards["testOrNodeStarted"] and not guards["recordingRootCreated"]
assert guards["ordinaryPrivilegeGuard"] == guards["runnerEnvironmentHostAndEventGuard"] == "passed"
assert read(root / "runner-config-contract.json")["argumentArrayRoundtripPassed"]
provenance = read(root / "preparation-source.json")
assert provenance["source"] == source and provenance["parent"] == "8e266abac78ff6d1b78cb05e680c7c5dbe1f0e56"
assert provenance["productAndOriginalFixtureTreeUnchanged"]
assert provenance["applicationTree"] == "bf4365586d874d1a9268636e5227efad4621bad1"
changed = set(subprocess.check_output(
    ["git", "diff", "--name-only", provenance["parent"], source], cwd=code).decode().splitlines())
assert len(changed) == 7 and changed == {row["path"] for row in provenance["files"]}
assert not any(path.startswith(("apps/", "docs/", ".adg/")) for path in changed)
for row in provenance["files"]:
    blob = subprocess.check_output(["git", "rev-parse", source + ":" + row["path"]], cwd=code).decode().strip()
    assert blob == row["gitBlob"]
assert read(root / "remote-code-readback.json")["sha"] == source
analyses = []
for run_id, event in ((38705201081, "push"), (38705207794, "pull_request")):
    terminal = read(root / f"run-{run_id}-terminal.original.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    assert run["event"] == event and run["head_sha"] == source and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        found = [job for job in jobs if job["name"] == name]
        assert len(found) == 1 and found[0]["conclusion"] == "success"
    windows = next(job for job in jobs if job["name"] == "backend-windows")
    for name in ("Prepare first-regression ETW only on the Windows diagnostic branch", "Windows backend regression"):
        assert next(step for step in windows["steps"] if step["name"] == name)["conclusion"] == "success"
    logs = read(root / f"run-{run_id}-log-verification.json")
    log_path = root / f"run-{run_id}-logs.original.zip"
    assert log_path.stat().st_size == logs["logs"]["bytes"] and sha(log_path.read_bytes()) == logs["logs"]["sha256"]
    assert logs["safeEntriesAndCrcVerified"] and not logs["historicalFailureCleared"]
    expected_checkout = source if event == "push" else read(root / "pr-actual-tree-verification.json")["prActualCheckout"]
    assert set(logs["actualCheckout"].values()) == {expected_checkout}
    assert logs["rustResults"]["backend"] == {"suites": 35, "passed": 900, "failed": 0, "ignored": 6}
    assert logs["rustResults"]["backend-windows"] == {"suites": 35, "passed": 627, "failed": 0, "ignored": 7}
    with zipfile.ZipFile(log_path) as archive:
        assert archive.testzip() is None
        for row in logs["selectedLogs"]:
            data = gzip.decompress((root / row["file"]).read_bytes())
            assert archive.read(row["member"]) == data and sha(data) == row["sha256"]
    saved = root / f"run-{run_id}"
    download = read(saved / "download-verification.json")
    metadata = read(saved / "artifact.original.json")
    artifact = saved / "artifact.original.zip"
    assert artifact.stat().st_size == download["artifactBytes"] == metadata["size_in_bytes"]
    assert sha(artifact.read_bytes()) == download["artifactSha256"] == metadata["digest"][7:]
    assert download["apiDigestMatches"] and download["safePathsAndCrcVerified"]
    with zipfile.ZipFile(artifact) as archive:
        assert archive.testzip() is None
        for row in download["selectedEntries"]:
            stored = (saved / "selected-original" / row["savedAs"]).read_bytes()
            assert sha(stored) == row["savedSha256"] and len(stored) == row["savedBytes"]
            data = gzip.decompress(stored) if row["savedAs"].endswith(".jsonl.gz") else stored
            assert data == archive.read(row["path"]) and sha(data) == row["sha256"]
    result = subprocess.run([sys.executable, "-B", "-X", "utf8", str(root / "analyze-first-ci.py"), str(run_id)],
                            capture_output=True, text=True, check=True)
    analysis = read(saved / "analysis.json")
    assert json.loads(result.stdout) == analysis
    assert analysis["actualSource"] == expected_checkout
    assert analysis["firstOrdinaryCargoLibraryExecutionCaptured"] and analysis["executions"] == 1
    assert analysis["libraryExitCode"] == 0
    assert analysis["libraryResult"] == {"passed": 530, "failed": 0, "ignored": 4}
    assert not analysis["queryFailureObservedInCurrentBuild"] and not analysis["currentBuildFailureWindowEtwObtained"]
    assert not analysis["sameHistoricalFailedBinary"] and not analysis["historicalFixedArtifactFailureWindowEtwObtained"]
    assert not analysis["rootCauseDetermined"] and not analysis["historicalFailuresCleared"]
    analyses.append(analysis)
tree = read(root / "pr-actual-tree-verification.json")
assert tree["headTree"] == tree["prActualTree"] and tree["treesMatch"] and not tree["binaryEqualityInferred"]
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
assert pr["head"]["sha"] == source and pr["state"] == "open" and pr["draft"]
assert not pr["merged"] and pr["auto_merge"] is None
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
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source,
    "localAdmissionTests": 12, "pureCollectorChecks": 108, "bothExactCiRunsSuccessful": True,
    "firstCargoLibraryExecutionCapturedPerRun": 1, "runs": [
        {key: item[key] for key in ("runId", "event", "actualSource", "actualLibrarySha256", "rootPid",
                                   "rootCreationUtc", "recordedEvents", "eventBytes", "processStartsAndStops")}
        for item in analyses],
    "currentLibraryCasesPerRun": {"passed": 530, "failed": 0, "ignored": 4},
    "sameHistoricalFailedBinary": False, "currentFailureWindowEtwObtained": False,
    "historicalFixedArtifactFailureWindowEtwObtained": False, "historicalFailuresCleared": False,
    "rootCauseDetermined": False, "nativeCountAtCheckpoint": 0, "pr5DraftOpenUnmerged": True,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path) == report
print(json.dumps(report))
