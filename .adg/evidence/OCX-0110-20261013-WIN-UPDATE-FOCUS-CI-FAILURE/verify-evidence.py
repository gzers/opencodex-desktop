"""Preserve failed exact CI plus bounded local diagnostics; never overwrite failure."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "d975b2351920bc027c931c8135b5605fde7a46c2"
native_source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
run = read("run-failed.json")
jobs = read("jobs-failed.json")
assert run["id"] == 38349316806 and run["head_sha"] == source
assert run["status"] == "completed" and run["conclusion"] == "failure"
assert run["head_branch"] == "feature/0.1.10-update-center-focus"
failed = next(job for job in jobs["jobs"] if job["id"] == 115137831818)
assert failed["name"] == "backend-windows" and failed["conclusion"] == "failure"
assert next(step for step in failed["steps"] if step["name"] == "Windows backend regression")["conclusion"] == "failure"
for fragment in ["npm --prefix ../ui run build", "cargo fmt", "cargo clippy"]:
    assert next(step for step in failed["steps"] if fragment in step["name"])["conclusion"] == "success"
assert next(job for job in jobs["jobs"] if job["name"] == "build")["conclusion"] == "skipped"
assert all(next(job for job in jobs["jobs"] if job["name"] == name)["conclusion"] == "success"
           for name in ["frontend", "backend", "release-tools", "windows-smoke-evidence"])
log = (root / "backend-windows-failed.log").read_text(encoding="utf-8-sig")
assert "502 passed; 3 failed; 4 ignored" in log
assert log.count("fixture descendant did not start") == 3
for detail in ["elapsed=2.1725273s, result=Timeout", "elapsed=2.174039s, result=Timeout",
               "elapsed=2.1548715s, result=Timeout"]:
    assert detail in log
assert "metadata_timeout_cleans_descendant_and_preserves_unrelated_process ... ok" in log
watcher = read("watcher-terminal.json")
assert watcher["runId"] == run["id"] and watcher["sha"] == source
assert watcher["status"] == "completed" and watcher["conclusion"] == "failure"
assert not watcher["productInvokedOrRestarted"]
identity = read("identity-and-source-witness.json")
assert identity["sourceCommit"] == source and identity["ciRunId"] == run["id"]
assert identity["artifactId"] == 11790820992
assert identity["zipSha256"] == "8da4ce67906620da0cfe42b76863922c87b44809766d946d360b4f3eab01607d"
assert identity["zipDigestMatchesApi"] and identity["testExeMatchesCiReceipt"]
assert identity["testExeSha256"] == read("ci-test-exe-sha256.json")["Hash"].lower() == \
    "e5aaed4c66d4bc362c1a63f9485e3fdf806f620e7be6a39d503c5e7c0018a4fc"
artifacts = read("artifacts.json")
artifact = next(item for item in artifacts["artifacts"] if item["id"] == identity["artifactId"])
assert artifact["digest"] == "sha256:" + identity["zipSha256"] and not artifact["expired"]
assert artifact["workflow_run"]["head_sha"] == source
assert identity["sourceDiff"] == [
    "apps/desktop/ui/src/features/updates/UpdateCenter.vue", "apps/desktop/ui/tests/update-center.test.ts"]
assert len(identity["sourceWitnesses"]) == 3
assert all(item["eaeBlob"] == item["d975Blob"] and item["unchanged"] for item in identity["sourceWitnesses"])
assert not any(identity[key] for key in ["productionTimeoutChanged", "testAssertionsChanged",
    "failureCauseIsolated", "localPassClearsCiFailure", "ciRerunRequested", "mainIntegrated"])
assert read("ci-environment.json")["node"] == "v24.21.0" and identity["localNodeVersion"] == "v24.19.0"
node = read("exact-ci-node-identity.json")
assert node["version"] == "v24.21.0" and node["digestMatchesOfficial"] and node["isolatedPath"]
assert not node["dailyNodeReplaced"] and not node["lifecycleScriptsExecuted"]
assert node["zipSha256"] == "158f7685b44de51f6c0df1d153526cbcd3e1bc739a8dfc607721cef75de9e541"
shasums = (root / "node24-21-official-shasums.txt").read_text(encoding="utf-8")
assert re.search(node["zipSha256"] + r"\s+node-v24\.21\.0-win-x64\.zip", shasums)
for mode, threads in [("serial", 1), ("concurrent", 7), ("serial-ci-node", 1), ("concurrent-ci-node", 7)]:
    value = read(mode + "-result.json")
    assert value["sourceCommit"] == source and value["ciRunId"] == run["id"]
    assert value["testExeSha256"] == identity["testExeSha256"]
    assert value["requestedTestThreads"] == threads and value["exitCode"] == 0
    assert value["status"] == "actual-local-tests-passed" and value["sameOriginalDesktopLifecycle"]
    assert not any(value[key] for key in ["inheritedCredentials", "desktopApplicationStarted",
                                        "testTimeoutBudgetAltered", "fixtureSourceModified"])
    assert not value.get("outerCleanupUsed", False) and value["originalProductionProbeSeconds"] == 2
    for token_key in ["helperToken", "testToken"]:
        assert value[token_key]["elevated"] is False and value[token_key]["elevationType"] == 3
    stdout = (root / (mode + "-stdout.log")).read_text(encoding="utf-8")
    stderr = (root / (mode + "-stderr.log")).read_text(encoding="utf-8")
    assert "9 passed; 0 failed; 0 ignored" in stdout and "500 filtered out" in stdout
    assert stdout.count(" ... ok") == 9
    assert stderr.count("query fixture ") == 7
    assert "watchdog" not in stderr and "fixture descendant did not start" not in stderr
    assert "version/success:" in stderr and "result=Success" in stderr
    assert "version/timeout:" in stderr and "result=Timeout" in stderr
    if mode.endswith("-ci-node"):
        assert value["actualNodeVersion"] == node["version"] and value["nodeExecutable"] == node["nodeExecutable"]
for filename in ["retained-lifecycle.json", "retained-lifecycle-after-exact-node.json"]:
    life = read(filename)
    assert life["sourceCommit"] == native_source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
    assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
    assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
    assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
preserved = read("preservation-after-diagnostics.json")
assert preserved["nativeSourceCommit"] == native_source and preserved["applicationPid"] == 37276
assert preserved["bytesPreserved"] and preserved["remainingOwnedFixtureNodeProcesses"] == 0
assert preserved["monitorExecSession"] == 46484 and preserved["monitorPid"] == 3920
assert preserved["managerDeadlineEpoch"] == 1791903339 and preserved["panelDeadlineEpoch"] == 1791903340
assert preserved["originalDataHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert preserved["originalDataHashes"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert not any(preserved[key] for key in ["productOrCycleRestarted", "frontWindowActivated",
                                        "rawNetworkCountsAccepted", "fullNativeGateAccepted", "releaseAccepted"])
disposition = read("failure-disposition.json")
assert disposition["sourceCommit"] == source and disposition["ciConclusion"] == "failure"
assert disposition["ciLibraryResult"] == {"passed": 502, "failed": 3, "ignored": 4}
assert not disposition["localReadback"]["failureCauseIsolated"] and not disposition["localReadback"]["wholeCiWorkspaceReproducedLocally"]
assert not disposition["actions"]["ciRerunRequested"] and not disposition["actions"]["fixtureSerializedOrThresholdLoosened"]
assert not disposition["fullCiAccepted"] and not disposition["actions"]["mainIntegrated"]
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "exact failed feature Windows CI logs/artifact/source plus four unchanged local9-test diagnostics, including exact CI Node; failure remains",
    "featureSourceCommit": source, "nativeSourceCommit": native_source,
    "ciAccepted": False, "ciFailureCleared": False, "failureCauseIsolated": False,
    "localMatchedSamples": 4, "localTestsPerSample": 9, "originalProbeDeadlineSeconds": 2,
    "mainIntegrated": False, "fixedNativeUiAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "failedCiPreserved": True,
                  "localDiagnostics": "four9-test passes", "failureCauseIsolated": False}))
