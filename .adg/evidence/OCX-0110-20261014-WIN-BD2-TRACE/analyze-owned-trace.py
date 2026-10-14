"""Summarize immutable raw snapshots without treating success as a failed trace."""
import collections
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
source = "bd2f6eb10d58115235e35d575507bb69f44666da"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
result = read("single-run-result.json")
assert result["source"] == source and result["testExitCode"] == 101
assert not result["forcedOuterCleanup"] and not result["desktopStarted"]
assert result["arguments"] == [] and result["defaultConcurrencyPreserved"] and result["testCapturePreserved"]
assert result["protectedSandboxBefore"] == result["protectedSandboxAfter"]
assert result["dailyExeBefore"] == result["dailyExeAfter"]
assert result["finalNativeCount"] == 0
raw = (root / "owned-process-thread-io.raw.jsonl").read_bytes()
rows = [json.loads(line) for line in raw.decode().splitlines()]
assert rows[0]["kind"] == "observer-start" and rows[-1]["kind"] == "observer-finished"
assert not rows[-1]["fileBufferOverflow"] and not rows[-1]["schedulingEtwRecorded"]
assert not rows[-1]["processTerminatedByObserver"] and not rows[-1]["unrelatedProcessDataPersisted"]
stdout = (root / "test-stdout.original.log").read_text(encoding="utf-8-sig")
expected_tests = [
    "infrastructure::official_version_source::windows_tests::version_nonzero_exit_reaps_owned_helper",
    "infrastructure::official_version_source::windows_tests::version_success_reaps_owned_helper_and_preserves_version_output",
    "infrastructure::official_version_source::windows_tests::version_timeout_cleans_owned_tree_and_preserves_unrelated_process",
    "modules::about::remote::windows_tests::metadata_output_overflow_cleans_descendant",
    "modules::about::remote::windows_tests::metadata_success_also_reaps_its_owned_helper_tree",
    "modules::about::remote::windows_tests::metadata_nonzero_exit_cleans_descendant",
    "modules::about::remote::windows_tests::metadata_timeout_cleans_descendant_and_preserves_unrelated_process",
]
assert all(f"test {name} ... ok" in stdout for name in expected_tests)
assert "529 passed; 1 failed; 4 ignored; 0 measured; 0 filtered out" in stdout
assert stdout.count("---- commands::workspace::tests::local_documents_are_frozen stdout ----") == 1
samples = [row for row in rows if row["kind"] == "owned-process-sample"]
all_threads = [thread for row in samples if isinstance(row.get("threads"), list) for thread in row["threads"]]
chains = [thread["waitChain"] for thread in all_threads if thread.get("waitChain")]
nodes = [row for row in rows if row["kind"] == "owned-node-discovered"]
assert len(nodes) == 6 and all(row["imageAndCreationVerified"] for row in nodes)
ids = [rows[0]["rootPid"]] + [row["pid"] for row in nodes]
assert all(row["pid"] in ids for row in samples)
groups = []
for pid in ids:
    observed = [row for row in samples if row["pid"] == pid]
    assert observed
    creations = {row["creationFileTime100ns"] for row in observed}
    assert len(creations) == 1
    valid_io = [row["io"] for row in observed if row["ioOk"]]
    assert valid_io
    deltas = {key: valid_io[-1][key] - valid_io[0][key] for key in valid_io[0]}
    assert all(value >= 0 for value in deltas.values())
    groups.append({
        "pid": pid, "creationFileTime100ns": observed[0]["creationFileTime100ns"],
        "sampleCount": len(observed), "firstObservedMs": observed[0]["elapsedMs"],
        "lastObservedMs": observed[-1]["elapsedMs"],
        "firstWaitStatus": observed[0]["waitStatus"], "lastWaitStatus": observed[-1]["waitStatus"],
        "firstKernelCpu100ns": observed[0]["kernelCpu100ns"], "lastKernelCpu100ns": observed[-1]["kernelCpu100ns"],
        "firstUserCpu100ns": observed[0]["userCpu100ns"], "lastUserCpu100ns": observed[-1]["userCpu100ns"],
        "sampleIntervalIoDeltas": deltas,
        "deltaStartsAtFirstObservationNotProcessCreation": True,
        "caseRoleMappedFromCapturedOriginalTestLog": False,
    })
file_notifications = [row for row in rows if row["kind"] == "fixture-file-notification"]
assert len(file_notifications) == 274
chain_outcomes = collections.Counter(f"{chain['ok']}:{chain['error']}" for chain in chains)
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
    "scope": "one Windows local success comparison with sampled own-process thread/WCT/IO and fixture file notifications",
    "rawTraceSha256": hashlib.sha256(raw).hexdigest(), "rawTraceBytes": len(raw),
    "rawRecordCounts": dict(collections.Counter(row["kind"] for row in rows)),
    "fullLibrary": {"passed": 529, "failed": 1, "ignored": 4, "filtered": 0, "exitCode": 101},
    "failedTest": "commands::workspace::tests::local_documents_are_frozen",
    "failedAssertion": "license.opened",
    "failedTestScope": "CI-compiled CARGO_MANIFEST_DIR points at runner repository absent on this physical host",
    "queryFixtureCasesPassed": expected_tests, "queryFixtureFailureReproduced": False,
    "originalFailedCiTraceObtained": False, "originalCiFailureCleared": False,
    "originalCiRootCauseIdentified": False, "etwSchedulingOrKernelFileTraceObtained": False,
    "observerInitializedAfterTestStartedMs": result["observerInitializedAtMs"],
    "sampleCount": result["samples"], "maximumSampleReadMs": result["maximumSampleMs"],
    "threadStateCounts": dict(collections.Counter(thread.get("state", "unavailable") for thread in all_threads)),
    "threadWaitReasons": dict(collections.Counter(thread["waitReason"] for thread in all_threads if thread.get("waitReason"))),
    "wctOutcomeCounts": dict(chain_outcomes), "waitChainNodesBeyondFirstStored": False,
    "processSummaries": groups,
    "limitations": [
        "One run only; no rerun-to-green, altered default test concurrency, prewarming or deadline/assertion changes.",
        "The CI failure did not reproduce; the snapshots describe locally successful query cases, not the historical failed runner.",
        "Default libtest capture hides successful-case eprintln; case-to-PID mapping and full process birth coverage are incomplete.",
        "Observer was initialized about 312ms after the test process began; short-lived processes may be missed.",
        "Six Node processes were sampled; this is not all seven controls plus every query worker/descendant.",
        "The observer's longest read was about 1459ms, including dynamic waits; diagnostics can perturb timing and are not performance acceptance.",
        "WCT is dynamic; thread states are sampled snapshots, not CPU scheduling/context-switch ETW events.",
        "Filesystem records are notification-delivery times, not operation-start times, latencies, filename-level I/O ETW, or all reads.",
        "Process I/O counters are cumulative; deltas begin at first observation and include the retained test process's full library activity.",
        "No causal attribution to Defender, loader, CPU starvation or product code is supported.",
    ],
    "nativeBeforeAfter": [0, 0], "sameProtectedSandboxAndDailyBytes": True,
    "nextEvidenceNeeded": "A actually failing reproduction/runner window with verified owned PID/creation and scheduler + file-I/O ETW; requires an appropriate privileged scoped capture.",
}
(root / "owned-trace-analysis.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
license_code = subprocess.run(["git", "show", source + ":apps/desktop/tauri/src/commands/workspace.rs"],
                             cwd=code, capture_output=True, check=True).stdout
assert b"CARGO_MANIFEST_DIR" in license_code and b"license.opened" in license_code
(root / "compiled-workspace-path-diagnosis.json").write_text(json.dumps({
    "source": source, "sourceBlobSha256": hashlib.sha256(license_code).hexdigest(),
    "originalCompiledWorkspace": r"D:\a\opencodex-desktop\opencodex-desktop\apps\desktop\tauri",
    "compiledWorkspaceExistsOnUserHost": Path(r"D:\a\opencodex-desktop\opencodex-desktop\apps\desktop\tauri").exists(),
    "artificialRunnerPathCreated": False, "testOrBinaryAltered": False,
    "licenseFailureCountedAsQueryFailure": False,
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"queryCasesPassed": 7, "failureReproduced": False, "rawRecords": len(rows),
                  "ownNodesSampled": len(nodes), "wctOutcomes": dict(chain_outcomes),
                  "originalFailureClosed": False}))
