"""Compare retained historical failure and successful replay without executing either workload."""
import gzip
import hashlib
import json
import re
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent
workspace = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
old_dir = workspace / (
    "docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/"
    "05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据")
old_path = old_dir / "control-pr-terminal-backend-windows.log.gz"
old_raw = gzip.decompress(old_path.read_bytes())
old_text = old_raw.decode("utf-8-sig")
strip_prefix = lambda text: re.sub(r"(?m)^\d{4}-\d\d-\d\dT[0-9:.]+Z ?", "", text)
old_clean = strip_prefix(old_text)
match = re.search(r'\{\s*"logicalProcessors":.*?"rustTestTimeUnit":\s*(?:null|"[^"]*")\s*\}', old_clean, re.S)
assert match, "Historical environment observation must be read from original logs."
old_environment = json.loads(match[0])
assert old_environment["logicalProcessors"] == 4
assert old_environment["rustTestThreads"] is None and old_environment["rustTestTimeUnit"] is None
assert "cargo test --locked --workspace --features integration-test -- --show-output" in old_clean
old_tests = re.search(r"test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored;", old_clean)
assert old_tests and tuple(map(int, old_tests.groups())) == (523, 7, 4)
prior = root.parent / "OCX-0110-20261015-WIN-CI-ETW"
admission = json.loads((prior / "diagnostic-original/out/windows-query-etw/admission.json").read_text())
analysis = json.loads((prior / "diagnostic-analysis.json").read_text())
receipt = json.loads((prior / "diagnostic-original/out/windows-query-etw/original-test-executable-receipt.original.json").read_text(encoding="utf-8-sig"))
assert receipt["Hash"].lower() == admission["testSha256"]
assert receipt["Path"] != admission["testExecutable"]
assert old_environment["nodeExecutable"] != admission["node"]
assert admission["originalLibraryCommandArguments"] == []
collector = subprocess.check_output(
    ["git", "show", "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff:test/smoke/windows-owned-etw.cs"], cwd=code)
collector_text = collector.decode("utf-8")
for name in ("TEMP", "TMP", "HOME", "USERPROFILE"):
    assert f'info.Environment["{name}"] =' in collector_text
assert 'info.Environment["TEMP"] = temp;' in collector_text
assert 'info.Environment.Remove(name)' in collector_text
stdout_path = prior / "diagnostic-original/runtime/ci-bd2-query-etw/capture/test-stdout.original.log"
replay_stdout = stdout_path.read_text(encoding="utf-8-sig")
assert "530 passed; 0 failed; 4 ignored;" in replay_stdout
assert "control-snapshot=" not in replay_stdout and "control-marker-at=" not in replay_stdout
report = {
    "historicalRun": 38537166560, "replayRun": 38662786466,
    "testExecutableBytesSame": True, "testExecutableSha256": admission["testSha256"],
    "historicalEnvironmentFromOriginalLog": old_environment,
    "historicalTestExecutablePath": receipt["Path"],
    "replayTestExecutablePath": admission["testExecutable"],
    "replayNodePath": admission["node"],
    "historicalCommand": "cargo test --locked --workspace --features integration-test -- --show-output",
    "replayLibraryArguments": [],
    "historicalPostCompileWorkloadContextPreserved": False,
    "sameRuntimeEnvironmentProven": False,
    "testAndNodeLocationsChanged": True,
    "replayTempAndHomeEnvironmentOverridden": ["TEMP", "TMP", "HOME", "USERPROFILE"],
    "replaySuccessfulFixturePhaseOutputPresent": False,
    "perCaseEtwPidToQueryLabelMappingProven": False,
    "allRetainedNodeCmdLifecyclesHaveStop": analysis["allRetainedProcessStartsHaveStops"],
    "targetFailureReproduced": False,
    "causalClaimMade": False,
    "newOriginalLibraryOrNodeRuns": 0, "etwRecordingsOrCiReruns": 0,
    "nextEvidence": [
        "Actual failing query time window, with retained PID/creation and case/phase correlation.",
        "Preserve and record real run paths, CLI flags and environment; audit all intentional differences.",
        "Do not use another successful replay, a longer deadline, forced contention or warm-up to close the failure.",
    ],
    "sources": [
        {"path": old_path.relative_to(workspace).as_posix(), "sha256": hashlib.sha256(old_path.read_bytes()).hexdigest(),
         "uncompressedSha256": hashlib.sha256(old_raw).hexdigest()},
        {"path": "ca26e771:test/smoke/windows-owned-etw.cs", "gitBlob": subprocess.check_output(
            ["git", "rev-parse", "ca26e771:test/smoke/windows-owned-etw.cs"], cwd=code).decode().strip(),
         "sha256": hashlib.sha256(collector).hexdigest()},
        {"path": stdout_path.relative_to(workspace).as_posix(), "sha256": hashlib.sha256(stdout_path.read_bytes()).hexdigest()},
    ],
}
target = root / "replay-context-audit.json"
if target.exists():
    assert json.loads(target.read_text()) == report
else:
    target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
