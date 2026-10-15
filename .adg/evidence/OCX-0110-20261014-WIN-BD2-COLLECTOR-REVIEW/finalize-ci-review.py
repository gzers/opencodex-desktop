"""Bind complete CI evidence while leaving administrator recording unapproved."""
import gzip
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
def git(*args):
    return subprocess.run(["git", *args], cwd=code, check=True, capture_output=True).stdout.decode().strip()
required = ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]
query_names = [
    "infrastructure::official_version_source::windows_tests::version_nonzero_exit_reaps_owned_helper",
    "infrastructure::official_version_source::windows_tests::version_success_reaps_owned_helper_and_preserves_version_output",
    "infrastructure::official_version_source::windows_tests::version_timeout_cleans_owned_tree_and_preserves_unrelated_process",
    "modules::about::remote::windows_tests::metadata_success_also_reaps_its_owned_helper_tree",
    "modules::about::remote::windows_tests::metadata_nonzero_exit_cleans_descendant",
    "modules::about::remote::windows_tests::metadata_output_overflow_cleans_descendant",
    "modules::about::remote::windows_tests::metadata_timeout_cleans_descendant_and_preserves_unrelated_process",
]
results = []
for run_id in [38555890377, 38556292712]:
    terminal = read(root / f"ci-terminal-{run_id}.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == "success"
    job_map = {job["name"]: job for job in jobs}
    assert all(job_map[name]["conclusion"] == "success" for name in required)
    assert job_map["build"]["conclusion"] == "skipped"
    summary = read(root / f"ci-log-summary-{run_id}.json")
    assert summary["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    actual = {}
    for job_name in ["backend", "backend-windows"]:
        raw = gzip.decompress((root / f"ci-{run_id}-{job_name}-checkout.log.gz").read_bytes())
        text = raw.decode("utf-8-sig")
        tail = text[text.rfind("log -1 --format=%H"):]
        actual[job_name] = re.findall(r"\b[0-9a-f]{40}\b", tail)[-1]
    assert actual["backend"] == actual["backend-windows"]
    checkout = actual["backend"]
    if run["event"] == "push":
        assert checkout == source
    else:
        assert run["event"] == "pull_request" and checkout == "a813c1dc964e5f4cc441aa75d79a7d2277823dbd"
    assert git("rev-parse", checkout + "^{tree}") == git("rev-parse", source + "^{tree}")
    windows = gzip.decompress((root / f"ci-{run_id}-backend-windows-tests.log.gz").read_bytes()).decode("utf-8-sig")
    assert all("test " + name + " ... ok" in windows for name in query_names)
    results.append({
        "runId": run_id, "event": run["event"], "associatedHead": source,
        "actualCheckout": checkout, "actualTreeEqualsHelperHead": True,
        "status": "completed", "conclusion": "success",
        "jobs": {name: job_map[name]["conclusion"] for name in required + ["build"]},
        "rustCounts": summary["rustCounts"], "queryFixtureCasesPassed": query_names,
        "archiveBytes": summary["archiveBytes"], "archiveSha256": summary["archiveSha256"],
        "recorderOrPureScopeTestsExecutedByCi": False,
    })
assert git("diff", "--name-only", base, source, "--", "apps", ".github") == ""
wait = read(root / "ci-wait-status.json")
assert wait["status"] == "both-existing-runs-terminal-verified" and wait["elapsedSeconds"] < 600
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "base": base,
    "ciResults": results, "productAndOriginalWorkflowBytesUnchangedSinceBd2": True,
    "helperWaitExecSession": 67983, "helperWaitPid": wait["helperPid"],
    "helperWaitTerminal": True, "boundedWaitElapsedSeconds": wait["elapsedSeconds"],
    "ciManuallyDispatchedOrRerun": False, "rawEtwRecordedOnUserHost": False,
    "originalFailureReproducedOrClosed": False,
    "administratorApprovalStillPending": True,
    "goalAutomaticallyContinuingDoesNotApproveRecording": True,
    "earlierPreparationEvidenceModified": False,
    "localNativeInstanceCount": 0, "fullVersionGatePromoted": False,
    "pr5Merged": False, "releaseAccepted": False,
    "logParserCorrection": "Match the initial checkout step only, not Post checkout; reused already-downloaded ZIP and selected bytes after verifying their equality.",
    "firstInspectionEncodingCorrection": "Explicit UTF-8 after console-default GBK read failed; original bytes unchanged.",
}
(root / "ci-review-final-result.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"bothCiSuccessful": True, "actualTreesMatched": True,
                  "windowsPassedPerRun": 627, "macPassedPerRun": 900,
                  "recorderExecuted": False, "adminApprovalPending": True}))
