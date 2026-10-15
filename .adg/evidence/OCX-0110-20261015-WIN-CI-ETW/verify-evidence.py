"""Verify the complete diagnostic receipt; historical query failure and version gates remain open."""
import gzip
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
for script in ("verify-preparation.py", "analyze-diagnostic.py"):
    subprocess.run([sys.executable, str(root / script)], check=True, capture_output=True)
actual_checkouts = {}
for number, event in ((38660621496, "push"), (38660625647, "pull_request")):
    terminal = read(f"ci-terminal-{number}.json")
    run = terminal["run"]
    assert run["head_sha"] == sha and run["id"] == number and run["event"] == event
    assert run["status"] == "completed" and run["conclusion"] == "success"
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        matches = [row for row in terminal["jobs"]["jobs"] if row["name"] == name]
        assert len(matches) == 1 and matches[0]["conclusion"] == "success"
    summary = read(f"ci-log-summary-{number}.json")
    archive = root / f"ci-original-logs-{number}.zip"
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == summary["archiveSha256"]
    assert archive.stat().st_size == summary["archiveBytes"] and summary["zipCrcPassed"]
    assert summary["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    assert summary["failedTestLines"] == [] and not summary["originalBd2FailedRunCleared"]
    for row in summary["selectedOriginalLogs"]:
        data = gzip.decompress((root / row["savedAs"]).read_bytes())
        assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
    for platform in ("backend", "backend-windows"):
        log = gzip.decompress((root / f"ci-{number}-{platform}-checkout.log.gz").read_bytes()).decode("utf-8-sig")
        match = re.search(r"log -1 --format=%H\s*\n[^\n]*?\b([0-9a-f]{40})\b", log)
        assert match
        actual_checkouts[f"{number}-{platform}"] = match[1]
        if event == "push":
            assert match[1] == sha
assert read("ci-watch-followup-result.json")["allRemoteTerminalObserved"]
assert read("diagnostic-dispatch-result.json")["httpStatus"] == "204"
diagnostic = read("diagnostic-analysis.json")
assert diagnostic["originalUnfilteredLibraryRuns"] == 1 and diagnostic["originalTestExitCode"] == 0
assert not diagnostic["targetQueryFailureReproduced"] and not diagnostic["actualFailureTimeWindowEvidenceObtained"]
assert not diagnostic["historicalBd2AndE6FailuresCleared"]
assert diagnostic["processStarts"] == diagnostic["processStops"] == 31
assert diagnostic["roleCounts"] == {"test": 1, "node": 22, "command-basename": 8}
assert diagnostic["serializationAndKernelDrops"] == 0
assert not diagnostic["fullRootHotEventCoverage"]
log_verification = read("diagnostic-job-log-verification.json")
assert log_verification["actualCheckout"] == sha and log_verification["zipCrcPassed"]
archive = root / "diagnostic-original-job-logs.zip"
assert archive.stat().st_size == log_verification["originalArchiveBytes"]
assert hashlib.sha256(archive.read_bytes()).hexdigest() == log_verification["originalArchiveSha256"]
for row in log_verification["selectedOriginalJobLogs"]:
    data = gzip.decompress((root / row["savedAs"]).read_bytes())
    assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
signature_step = next(
    row for row in log_verification["selectedOriginalJobLogs"]
    if "Verify collector guards" in row["member"])
signature_log = gzip.decompress((root / signature_step["savedAs"]).read_bytes()).decode("utf-8-sig")
assert "Get-AuthenticodeSignature" in signature_log
assert "Pinned official Node signature is not valid" in signature_log
assert "##[error]" not in signature_log
final = read("final-protection-check.json")
assert final["codeSource"] == final["codeRemote"] == sha and final["codeClean"]
assert final["nativeCount"] == 0 and final["nativeProcesses"] == [] and final["localNewDesktopStarts"] == 0
assert final["dailyExeUnchanged"] and final["protectedSandboxUnchanged"]
assert final["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert final["diagnosticDispatchCount"] == final["diagnosticOriginalLibraryRuns"] == 1
assert not final["automaticDiagnosticRetryRequested"] and not final["macNativeHandoffPerformed"]
assert not final["releaseRequested"] and not final["fullVersionGatePassed"]
manifest_path = root / "manifest.json"
report_path = root / "archive-verification.json"
paths = sorted((path for path in root.rglob("*") if path.is_file()
                and path.name not in {"manifest.json", "archive-verification.json"}),
               key=lambda path: path.relative_to(root).as_posix())
manifest = [
    {"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
     "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    for path in paths
]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert {row["path"]: row for row in read(manifest_path.name)} == {row["path"]: row for row in manifest}
report = {
    "archiveIntegrity": "passed", "files": len(manifest), "collectorSource": sha,
    "sameHeadPushAndPrCiSuccess": True, "actualCheckouts": actual_checkouts,
    "diagnosticRun": 38662786466, "artifact": 11939861138,
    "diagnosticDispatchCount": 1, "originalLibraryRunCount": 1,
    "originalTestPassed": 530, "originalTestFailed": 0, "originalTestIgnored": 4,
    "queryFixturesPassed": 7, "events": 17085, "bytes": 5811506,
    "processStartsAndStops": 31, "serializationAndKernelDrops": 0,
    "targetQueryFailureReproduced": False, "actualFailureWindowEtwObtained": False,
    "historicalFailureCleared": False, "fullRootHotEventCoverage": False,
    "localNativeInstances": 0, "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_path.name)
print(json.dumps(report))
