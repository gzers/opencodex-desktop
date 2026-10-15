"""Verify this source-fix stage without accepting the native/version gates."""
import gzip
import hashlib
import json
import re
from pathlib import Path
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
digest = lambda data: hashlib.sha256(data).hexdigest()

identity = read(root / "a2e4ea17-source-identity.json")
assert identity["source"] == source
assert identity["applicationTree"] == "8c98d9197c484f1b6172f041f9c167018ed86067"
assert len(identity["files"]) == 11 and len(identity["sharedBlobs"]) == 5
for row in identity["files"]:
    raw = subprocess.check_output(["git", "show", source + ":" + row["path"]], cwd=code)
    assert digest(raw) == row["sha256"] and len(raw) == row["bytes"]
    assert not row["path"].startswith(("docs/", ".adg/"))
for path in identity["unchangedQueryFiles"]:
    before = subprocess.check_output(["git", "rev-parse", identity["before"] + ":" + path], cwd=code)
    after = subprocess.check_output(["git", "rev-parse", source + ":" + path], cwd=code)
    assert before == after
assert read(root / "admission-tests.json")["exitCode"] == 0
assert "Ran 13 tests" in (root / "admission-tests.original.log").read_text(encoding="utf-8-sig")
assert read(root / "local-format.json")["exitCode"] == 0
audit_path = "apps/desktop/tauri/src/modules/ipc/audit.rs"
audit_before = subprocess.check_output(
    ["git", "show", "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9:" + audit_path], cwd=code).decode()
audit_after = subprocess.check_output(["git", "show", source + ":" + audit_path], cwd=code).decode()
assert audit_before.split("#[cfg(test)]")[0] == audit_after.split("#[cfg(test)]")[0]
original_assertions = re.findall(r"assert(?:_eq|_ne)?!\([\s\S]*?\);", audit_before)
assert len(original_assertions) == 12
for assertion in original_assertions:
    assert re.sub(r"\s+", "", assertion) in re.sub(r"\s+", "", audit_after)

for run_id, expected_source in (
    (38722473197, "6ab09772855b964f1f7e4f6faf170337109b3f7c"),
    (38722479010, "6ab09772855b964f1f7e4f6faf170337109b3f7c"),
    (38723079993, "bc14165ceb91cb084361e8dce23b10d381330e1a"),
    (38723086934, "bc14165ceb91cb084361e8dce23b10d381330e1a"),
    (38724440086, "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9"),
    (38724446374, "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9"),
    (38725991590, "020e668c4b9de5859e1137c08769af8c6be75281"),
    (38726001178, "020e668c4b9de5859e1137c08769af8c6be75281"),
    (38726727330, source),
    (38726732452, source),
):
    terminal = read(root / f"run-{run_id}-terminal.original.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    final = expected_source == source
    assert run["head_sha"] == expected_source and run["status"] == "completed" and run["run_attempt"] == 1
    assert run["conclusion"] == ("success" if final or run_id == 38724440086 else "failure")
    logs = read(root / f"run-{run_id}-log-verification.json")
    raw = (root / f"run-{run_id}-logs.original.zip").read_bytes()
    assert digest(raw) == logs["logs"]["sha256"] and len(raw) == logs["logs"]["bytes"]
    with zipfile.ZipFile(root / f"run-{run_id}-logs.original.zip") as archive:
        assert archive.testzip() is None
        for row in logs["selectedLogs"]:
            content = gzip.decompress((root / row["file"]).read_bytes())
            assert content == archive.read(row["member"])
            assert digest(content) == row["sha256"]
        text = "\n".join(archive.read(name).decode("utf-8-sig") for name in archive.namelist()
                         if name.endswith(".txt") and "/" in name)
        if expected_source.startswith("6ab"):
            assert "non-exhaustive patterns" in text and "DataRootWriteDenied" in text
        elif expected_source.startswith("bc141"):
            assert "drive_root_parent_traversal_and_source_nesting_are_rejected" in text
            assert "545 passed; 1 failed; 4 ignored" in text
        elif expected_source.startswith("020e"):
            assert "manual implementation of" in text and "is_multiple_of" in text
        elif run_id == 38724446374:
            assert "Original result preserved; first-regression capture incomplete" in text
            assert "retention_keeps_ninety_days_and_max_records has been running for over 60 seconds" in text
    if final:
        for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
            found, = [job for job in jobs if job["name"] == name]
            assert found["conclusion"] == "success"
        for job_name in ("backend", "backend-windows"):
            job, = [job for job in jobs if job["name"] == job_name]
            for name in ("Run cargo fmt --all -- --check",
                         "Run cargo clippy --locked --workspace --all-targets -- -D warnings"):
                step, = [step for step in job["steps"] if step["name"] == name]
                assert step["conclusion"] == "success"
        assert logs["rustResults"]["backend"] == {"suites": 35, "passed": 918, "failed": 0, "ignored": 6}
        assert logs["rustResults"]["backend-windows"] == {"suites": 35, "passed": 643, "failed": 0, "ignored": 7}

analyses = []
for run_id in (38723079993, 38723086934, 38724440086, 38724446374, 38726727330, 38726732452):
    saved = root / f"run-{run_id}"
    download = read(saved / "download-verification.json")
    metadata = read(saved / "artifact.original.json")
    artifact = saved / "artifact.original.zip"
    assert digest(artifact.read_bytes()) == download["artifactSha256"] == metadata["digest"][7:]
    assert artifact.stat().st_size == download["artifactBytes"] == metadata["size_in_bytes"]
    with zipfile.ZipFile(artifact) as archive:
        assert archive.testzip() is None
        for row in download["selectedEntries"]:
            stored = (saved / "selected-original" / row["savedAs"]).read_bytes()
            assert digest(stored) == row["savedSha256"] and len(stored) == row["savedBytes"]
            raw = gzip.decompress(stored) if row["savedAs"].endswith(".jsonl.gz") else stored
            assert raw == archive.read(row["path"]) and digest(raw) == row["sha256"]
    process = subprocess.run([sys.executable, "-B", str(root / "analyze-first-ci.py"), str(run_id)],
                             capture_output=True, text=True, check=True)
    analysis = read(saved / "analysis.json")
    assert json.loads(process.stdout) == analysis
    if run_id == 38724446374:
        assert analysis["libraryExitCode"] == -1 and analysis["libraryResult"] is None
        assert analysis["forcedOuterCleanup"] and not analysis["originalLibraryCompleted"]
    else:
        successful = run_id in (38724440086, 38726727330, 38726732452)
        assert analysis["libraryExitCode"] == (0 if successful else 101)
        assert analysis["libraryResult"] == (
            {"passed": 546, "failed": 0, "ignored": 4} if successful
            else {"passed": 545, "failed": 1, "ignored": 4})
    assert not analysis["queryFailureObservedInCurrentBuild"]
    assert not analysis["historicalFixedArtifactFailureWindowEtwObtained"]
    assert not analysis["rootCauseDetermined"] and not analysis["historicalFailuresCleared"]
    analyses.append(analysis)

tree = read(root / "pr-actual-tree-verification.json")
assert tree["source"] == source and tree["treesMatch"] and not tree["binaryEqualityInferred"]
assert tree["headTree"] == tree["prActualTree"] == "5279aa9b0f655954a47e758680eb2e5790672eef"
assert read(root / "remote-code-readback.json")["object"]["sha"] == source
protection = read(root / "after-ci-protection.original.json")
assert protection["source"] == source and protection["codeWorktreeClean"]
assert protection["nativeCount"] == 0 and protection["nativeProcesses"] == []
assert protection["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protection["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protection["originalTaskBytes"] == 3878 and len(protection["protectedSandboxHashes"]) == 4
for name in ("localProductOrOriginalLibraryOrFixedNodeStarted", "localEtwSessionStarted",
             "localUiInputInjected", "hostInstallerExecuted", "newAccountOrProfileCreated",
             "localStartupChoiceRegistryWritten", "permissionsOrSystemConfigurationChanged",
             "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert protection[name] is False
pr = read(root / "pr-terminal-after.original.json")
assert pr["head"]["sha"] == source and pr["state"] == "open" and pr["draft"]
assert not pr["merged"] and pr["auto_merge"] is None
handoff = read(root / "candidate-handoff.json")
assert handoff["source"] == source and handoff["runId"] == 38729632145
assert handoff["statusAtHandoff"] == "in_progress" and not handoff["candidateAcceptanceClaimed"]

manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": digest(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in row["path"] for row in entries)
report = {
    "sourceFixStageEvidence": "passed", "manifestFiles": len(entries), "source": source,
    "priorFailedRunsRetained": [38722473197, 38722479010, 38723079993, 38723086934,
                                38724446374, 38725991590, 38726001178],
    "currentCi": [38726727330, 38726732452], "bothCurrentCiRunsSuccessful": True,
    "macRustPerRun": {"passed": 918, "ignored": 6},
    "windowsRustPerRun": {"passed": 643, "ignored": 7},
    "captures": [{"run": row["runId"], "exitCode": row["libraryExitCode"],
                  "queryFailure": row["queryFailureObservedInCurrentBuild"],
                  "librarySha256": row["actualLibrarySha256"]} for row in analyses],
    "nativeRecoveryAcceptance": "pending", "newMatchingInstallerAcceptance": "pending",
    "matchingCandidateRun": 38729632145, "candidateAcceptanceClaimed": False,
    "historicalFixedArtifactFailureWindowEtwObtained": False, "rootCauseDetermined": False,
    "historicalFailuresCleared": False, "nativeCountAtCheckpoint": 0,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries and read(report_path) == report
print(json.dumps(report))
