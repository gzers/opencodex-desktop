"""Verify exact successful diagnostic CI while preserving the earlier unexplained failure."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
native = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
expected_data = {
    "preferences.json": "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa",
    "data-root.json": "8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25",
    "backup-cleanup-policy.json": "85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11",
    "update-schedule.json": "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20",
}
for filename in ["run-terminal.json", "run-audited-terminal.json"]:
    run = read(filename)
    assert run["head_sha"] == source and run["id"] == 38352525092
    assert run["head_branch"] == "feature/0.1.10-update-center-focus" and run["event"] == "push"
    assert run["status"] == "completed" and run["conclusion"] == "success"
for filename in ["jobs-terminal.json", "jobs-audited-terminal.json"]:
    jobs = read(filename)
    for name in ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]:
        job = next(job for job in jobs["jobs"] if job["name"] == name)
        assert job["status"] == "completed" and job["conclusion"] == "success"
    windows = next(job for job in jobs["jobs"] if job["name"] == "backend-windows")
    assert windows["id"] == 115147923845 and all(step["conclusion"] == "success" for step in windows["steps"])
    assert next(job for job in jobs["jobs"] if job["name"] == "build")["conclusion"] == "skipped"
log = (root / "backend-windows-terminal.log").read_text(encoding="utf-8-sig")
assert "505 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out" in log
summaries = [tuple(map(int, match)) for match in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)]
assert len(summaries) == 34
assert tuple(sum(item[index] for item in summaries) for index in range(3)) == (601, 0, 7)
assert '"logicalProcessors": 4,' in log and '"rustTestThreads": null,' in log
assert "version probe fixture=" not in log and "fixture descendant did not start" not in log
for name in [
    "version_success_reaps_owned_helper_and_preserves_version_output",
    "version_nonzero_exit_reaps_owned_helper",
    "version_timeout_cleans_owned_tree_and_preserves_unrelated_process",
]:
    assert f"test infrastructure::official_version_source::windows_tests::{name} ... ok" in log
watcher = read("watcher-terminal.json")
assert watcher["runId"] == 38352525092 and watcher["sha"] == source and watcher["conclusion"] == "success"
assert watcher["observationErrors"] == 0
assert not watcher["originalCiFailureCleared"] and not watcher["mainIntegrated"] and not watcher["foregroundInputIssued"]
history = [json.loads(line) for line in (root / "watcher-observations.ndjson").read_text(encoding="utf-8-sig").splitlines()]
assert history and history[-1]["status"] == "completed" and history[-1]["conclusion"] == "success"
assert all(item["sha"] == source and item["watcherPid"] == 38920 for item in history)
terminal = read("terminal-audit.json")
assert terminal["zipSha256"] == "3fae1e04e42f85ee681356c27464fbf6e4a93bd8d50ecaa612874fe385eca808"
assert terminal["runConclusion"] == "success" and terminal["artifactId"] == 11792440963
assert terminal["zipDigestMatchesApi"] and not terminal["executableStarted"]
artifact = read("artifact-content-audit.json")
assert artifact["sourceCommit"] == source and artifact["runId"] == 38352525092
assert artifact["zipDigestMatchesApi"] and artifact["zipCrcVerified"] and artifact["zipEntries"] == 165
assert artifact["ciTestExeSha256"] == "bcb9d346a829da51b50b4b85ae32258488e66aa9c25d5868dace0d3a21276a76"
assert artifact["ciFinalExeSha256FromSmokeReceipt"] == "8651ba8375835ca22802bfc2d6192da588f184be5a2395d0f417182db3c8c251"
assert len(artifact["selectedRawWitnesses"]) == 11 and not artifact["finalDesktopExePresentInZip"]
for item in artifact["selectedRawWitnesses"]:
    path = root / item["archivedPath"]
    assert path.stat().st_size == item["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"]
startup = read("artifact-raw/windows-startup/startup.json")
assert startup["expectedCommit"] == source and startup["sourceCommitMatched"] and startup["result"] == "pass"
assert startup["applicationTokenVerified"] and startup["mode"] == "any" and not startup["applicationModeMatched"]
assert startup["applicationToken"]["elevated"] is True and startup["applicationToken"]["elevationType"] == 1
assert artifact["cdpVersionEndpointTimeoutRetained"] and artifact["cdpListTargetAndMainOverviewSucceeded"]
local = read("local-stage-result.json")
assert local["sourceCommit"] == source and local["ciRunId"] == 38352525092 and local["requestedTestThreads"] == 7
assert local["testExeSha256"] == artifact["ciTestExeSha256"] and local["actualNodeVersion"] == "v24.21.0"
assert local["status"] == "actual-local-stage-tests-passed" and local["exitCode"] == 0
for key in ["helperToken", "testToken"]:
    assert local[key]["elevated"] is False and local[key]["elevationType"] == 3
assert local["sameOriginalDesktopLifecycle"] and local["originalDataBytesPreserved"]
assert local["originalDataHashes"] == local["afterDataHashes"] == expected_data
assert not local.get("outerCleanupUsed", False) and local["productionProbeSeconds"] == 2
for key in ["fixtureSourceChanged", "testTimeoutBudgetAltered", "desktopApplicationStarted",
            "inheritedCredentials", "foregroundInputIssued", "originalCiCauseIsolated", "originalCiFailureCleared"]:
    assert local[key] is False
stdout = (root / "local-stage-stdout.log").read_text(encoding="utf-8")
stderr = (root / "local-stage-stderr.log").read_text(encoding="utf-8")
assert "9 passed; 0 failed; 0 ignored" in stdout and "500 filtered out" in stdout
assert stdout.count(" ... ok") == 9 and stderr.count("query fixture ") == 7
phases = {}
for line in stderr.splitlines():
    match = re.fullmatch(r"version probe fixture=(.+) phase=([a-z-]+) elapsed=(.+)", line)
    if match:
        phases.setdefault(match[1], {})[match[2]] = match[3]
assert len(phases) == 3 and all(set(value) == {
    "runtime-ready", "spawned", "assigned-and-resumed", "waited-and-terminated"
} for value in phases.values())
assert read("local-stage-residual.json")["remainingOwnedFixtureNodeProcesses"] == 0
for filename in ["retained-lifecycle-after-ci.json", "retained-lifecycle-after-stage.json"]:
    value = read(filename)
    assert value["sourceCommit"] == native and value["desktopApplicationCount"] == 1
    assert {item["processId"] for item in value["processReadbacks"]} == {37276, 24300, 3920}
    assert all(item["liveHandleVerified"] and not item["hasExited"] for item in value["processReadbacks"])
    assert value["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
preservation = read("preservation-after-ci.json")
assert preservation["protectedDataHashes"] == expected_data and not preservation["foregroundInputIssued"]
cycle = read("monitor-state-after-ci.json")
assert cycle["sourceCommit"] == native and cycle["status"] == "observing-real-deadlines"
assert cycle["failedObservations"] == 0 and cycle["managerDeadlineEpoch"] == 1791903339
assert cycle["panelDeadlineEpoch"] == 1791903340 and not cycle["fullCycleAccepted"]
disposition = read("diagnostic-ci-disposition.json")
assert disposition["exactCiResult"] == "success" and disposition["sourceCommit"] == source
for key in ["originalCiFailureCleared", "originalCiCauseIsolated", "ciFinalExeAvailableForLocalRun",
            "mainIntegrated", "candidateInstallerBuiltOrDispatched", "nativeFocusFixAccepted", "fullNativeAccepted",
            "performanceAccepted", "realCyclesAccepted", "formalTaskReceipt", "releaseAccepted"]:
    assert disposition[key] is False
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
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "exact successful diagnostic CI, digest/CRC/raw artifact witnesses and9-test ordinary local stage telemetry",
    "sourceCommit": source, "exactCiAccepted": True, "ciArtifactWitnessesAccepted": True,
    "originalCiFailureCleared": False, "originalCiCauseIsolated": False,
    "ciFinalDesktopExeLocallyAvailable": False, "mainIntegrated": False,
    "nativeFocusFixAccepted": False, "ordinaryAdminPairedTokenGateAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({
    "archiveIntegrity": "passed", "files": len(entries), "exactDiagnosticCi": "success",
    "ordinaryLocalStageTests": 9, "originalCiCauseIsolated": False, "fullNativeAccepted": False,
}))
