"""Freeze failed full-library diagnostics without clearing the original failed CI."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "d975b2351920bc027c931c8135b5605fde7a46c2"
diagnostic = "be7395acfce0ec1e36b83300b576db408f8c35bb"
native = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
local_failures = [
    "commands::workspace::tests::local_documents_are_frozen",
    "modules::sync::config::tests::config_accepts_legacy_encryption_field_and_rejects_tls_override",
    "modules::sync::config::tests::legacy_encryption_field_is_read_and_not_written",
]
version_cases = [
    "version_nonzero_exit_reaps_owned_helper",
    "version_success_reaps_owned_helper_and_preserves_version_output",
    "version_timeout_cleans_owned_tree_and_preserves_unrelated_process",
]
for mode, argv in [("default", ["--nocapture"]), ("four", ["--test-threads=4", "--nocapture"])]:
    value = read(mode + "-result.json")
    assert value["sourceCommit"] == source and value["ciRunId"] == 38349316806
    assert value["testExeSha256"] == "e5aaed4c66d4bc362c1a63f9485e3fdf806f620e7be6a39d503c5e7c0018a4fc"
    assert value["actualNodeVersion"] == "v24.21.0" and value["hostLogicalProcessors"] == 20
    assert value["argv"] == argv and value["status"] == "actual-local-full-library-failed"
    assert value["exitCode"] == 101 and value["sameOriginalDesktopLifecycle"] and value["originalDataBytesPreserved"]
    assert value["originalDataHashes"] == value["afterDataHashes"]
    assert value["productionProbeDeadlineSeconds"] == 2 and not value.get("outerCleanupUsed", False)
    for key in ["inheritedCredentials", "desktopApplicationStarted", "foregroundInputIssued", "testFilterApplied",
                "ignoredTestsEnabled", "testAssertionsChanged", "fixtureSourceChanged",
                "fullCiWorkspaceMatched", "localPassClearsCiFailure", "ciRerunRequested"]:
        assert value[key] is False
    for key in ["helperToken", "testToken"]:
        assert value[key]["elevated"] is False and value[key]["elevationType"] == 3
    stdout = (root / (mode + "-stdout.log")).read_text(encoding="utf-8")
    stderr = (root / (mode + "-stderr.log")).read_text(encoding="utf-8")
    assert "running 509 tests" in stdout
    assert "502 passed; 3 failed; 4 ignored; 0 measured; 0 filtered out" in stdout
    assert stdout.count(" ... ok") == 502 and stdout.count(" ... FAILED") == 3
    for name in local_failures:
        assert "test " + name + " ... FAILED" in stdout
    for name in version_cases:
        assert "test infrastructure::official_version_source::windows_tests::" + name + " ... ok" in stdout
    assert stderr.count("query fixture ") == 7
    for mode_name, outcome in [("success", "Success"), ("nonzero", "Failure"), ("timeout", "Timeout")]:
        line = next(line for line in stderr.splitlines() if line.startswith("query fixture version/" + mode_name + ":"))
        assert "result=" + outcome in line
    assert "fixture descendant did not start" not in stderr
    assert "assertion failed: license.opened" in stderr
    assert "assertion failed: validate_config(&config).is_ok()" in stderr
    assert "assertion failed: validate_config(&parsed).is_ok()" in stderr
disposition = read("diagnostic-disposition.json")
assert disposition["localFailures"] == local_failures
assert not disposition["localFullLibraryPassed"] and not disposition["originalCiCauseIsolated"]
assert disposition["diagnosticSha"] == diagnostic and disposition["diagnosticCiRunId"] == 38352525092
assert not disposition["localDifferences"]["compiledRepositoryLicenseExists"]
assert all(item["unchanged"] for item in disposition["sourceWitnesses"])
assert not disposition["localDifferences"]["ignoredTestsExecuted"]
for key in ["mainIntegrated", "nativeFocusFixAccepted", "fullNativeAccepted", "pairedPerformanceAccepted",
            "cyclesAccepted", "formalTaskReceipt", "releaseAccepted"]:
    assert disposition[key] is False
source_check = read("source-check-result.json")
assert source_check["result"] == "passed" and source_check["diagnosticSourceCommit"] == diagnostic
assert source_check["officialReleaseSourceEqualsParentAfterCfgRemoval"] and source_check["testOnlyTraceBlocks"] == 4
assert source_check["ciTestCommandAndConcurrencyUnchanged"] and not source_check["localRustCompilationAccepted"]
run = read("diagnostic-ci-initial.json")
assert run["id"] == 38352525092 and run["head_sha"] == diagnostic and run["event"] == "push"
assert run["status"] == "in_progress" and run["conclusion"] is None
jobs = read("diagnostic-ci-initial-jobs.json")
assert next(job for job in jobs["jobs"] if job["name"] == "backend-windows")["status"] == "in_progress"
for filename in ["retained-lifecycle.json", "retained-lifecycle-after-four.json"]:
    value = read(filename)
    assert value["sourceCommit"] == native and value["desktopApplicationCount"] == 1
    assert {item["processId"] for item in value["processReadbacks"]} == {37276, 24300, 3920}
    assert all(item["liveHandleVerified"] and not item["hasExited"] for item in value["processReadbacks"])
    assert value["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
for filename in ["fixture-residual-readback.json", "fixture-residual-after-four.json"]:
    assert read(filename)["remainingOwnedFixtureNodeProcesses"] == 0
cycle = read("monitor-state-after-full-library.json")
assert cycle["status"] == "observing-real-deadlines" and cycle["sourceCommit"] == native
assert cycle["failedObservations"] == 0 and cycle["managerDeadlineEpoch"] == 1791903339
assert cycle["panelDeadlineEpoch"] == 1791903340 and not cycle["fullCycleAccepted"]
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
    "result": "passed", "filesVerified": len(entries), "localWholeLibrarySamples": 2,
    "eachLocalWholeLibraryResult": {"passed": 502, "failed": 3, "ignored": 4},
    "scope": "unchanged failed-CI EXE/Node full-library diagnostics and test-only diagnostic source guard; historical new CI still in progress",
    "originalCiAccepted": False, "originalCiFailureCleared": False, "originalCiCauseIsolated": False,
    "diagnosticCiAccepted": False, "localFullLibraryAccepted": False, "mainIntegrated": False,
    "nativeFocusFixAccepted": False, "pairedPerformanceAccepted": False, "cyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "localFullLibrarySamplesFailed": 2,
                  "originalCiFailureCleared": False, "diagnosticCiAccepted": False}))
