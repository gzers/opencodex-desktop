"""Freeze only preparation facts; later terminal logs and recording need separate receipts."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
result = read("preparation-result.json")
assert result["source"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
assert result["parent"] == "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
assert set(result["changedFiles"]) == {
    ".github/workflows/ci.yml", ".github/workflows/windows-candidate.yml",
    "test/smoke/windows-etw-ci-admit.py", "test/test_windows_etw_ci.py",
}
assert result["productAndOriginalCollectorUnchanged"] and result["codeWorktreeClean"]
assert result["nativeCount"] == 0 and result["nativeProcesses"] == []
for key in ("localOriginalLibraryRuns", "localOfficialNodeRuns", "localEtwRecordings", "localDesktopStarts"):
    assert result[key] == 0
for key in ("diagnosticDispatched", "newInstallerBuilt", "foregroundInputIssued", "fullVersionGatePassed",
            "failureTimeWindowEvidenceObtained", "originalFailuresCleared", "globalSettingsChanged", "releaseRequested"):
    assert not result[key]
assert result["dailyExeUnchanged"] and result["protectedSandboxUnchanged"]
assert result["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert result["pureAdmissionTestsPassed"] == 10 and result["originalRedRetained"]
assert result["actualPinnedArchivesVerified"] == 3
assert result["originalCollectorBudgetsUnchanged"] and result["originalQueryConcurrencyAndDeadlinesUnchanged"]
assert result["baselinePerformanceProfileIsolationStillUnproven"]
archives = read("offline-originals/pinned-archives-verification.json")
assert archives["status"] == "passed" and len(archives["archives"]) == 3
assert all(row["crcAndSafeNames"] and row["payloadHashMatches"] for row in archives["archives"])
assert archives["binariesExecuted"] == archives["networkRequests"] == archives["etwSessionsStarted"] == 0
assert read("offline-originals/offline-red-result.json")["exitCode"] != 0
final_log = (root / "offline-originals/offline-final.stderr.original.txt").read_text(encoding="utf-8-sig")
assert "Ran 10 tests" in final_log and final_log.rstrip().endswith("OK")
syntax = read("offline-originals/workflow-powershell-syntax.json")
assert syntax["powershellWorkflowBlocksParsed"] == 10 and syntax["syntaxErrors"] == syntax["blocksExecuted"] == 0
refused = read("offline-originals/local-refusal.stdout.original.txt")
assert refused["status"] == "admission-stopped-before-execution" and not refused["testOrNodeStarted"]
pr = read("pr5-recovery-after.original.json")
payload = read("pr5-update-payload.json")
assert pr["title"] == payload["title"] and pr["body"] == payload["body"]
assert pr["draft"] and not pr["merged"] and pr["auto_merge"] is None
assert pr["head"]["sha"] == result["source"]
snapshot = read("preparation-ci-checkpoint.original.json")
for number in (38660621496, 38660625647):
    run = snapshot["reads"]["run-" + str(number)]
    assert run["id"] == number and run["head_sha"] == result["source"]
    assert run["status"] == "in_progress" and run["conclusion"] is None
manifest_path = root / "preparation-manifest.json"
report_path = root / "preparation-verification.json"
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    names = {
        ".gitattributes", "preparation-result.json", "preparation-ci-checkpoint.original.json",
        "pr5-description.md", "pr5-update-payload.json", "pr5-update-attempt.json",
        "pr5-before-update.original.json", "pr5-after-update.original.json",
        "pr5-recovery-before.original.json", "pr5-recovery-response.original.json",
        "pr5-recovery-after.original.json", "pr5-recovery-attempt.json",
        "ci-watch-result.json", "live-job-log-attempt.json", "log-method-provenance.json",
    }
    names.update(path.name for path in root.glob("*.py"))
    names.update(path.relative_to(root).as_posix() for path in (root / "offline-originals").glob("*") if path.is_file())
    manifest = [
        {"path": name, "bytes": (root / name).stat().st_size,
         "sha256": hashlib.sha256((root / name).read_bytes()).hexdigest()}
        for name in sorted(names)
    ]
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
else:
    manifest = read(manifest_path.name)
for row in manifest:
    path = root / row["path"]
    assert path.stat().st_size == row["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(manifest), "source": result["source"],
    "offlineAdmissionRegressionTests": 10, "pinnedRealArchivesVerified": 3,
    "originalProductAndCollectorUnchanged": True, "localNativeInstances": 0,
    "preparedDiagnosticWasNotDispatchedAtCheckpoint": True,
    "ciNotTerminalAtCheckpoint": True, "prDraftAndUnmerged": True,
    "fullVersionGatePassed": False, "historicalFailureCleared": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_path.name)
print(json.dumps(report))
