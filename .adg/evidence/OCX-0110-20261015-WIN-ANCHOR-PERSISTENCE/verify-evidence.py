"""Accept only this source-fix stage, not native or release readiness."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
identity = read("source-identity.json")
assert identity["source"] == source
assert identity["applicationTree"] == "2b38f5218f26a0682e2eec520886c14107bf2bed"
row = identity["changedFile"]
data = subprocess.check_output(["git", "show", source + ":" + row["path"]], cwd=code)
assert sha(data) == row["sha256"] and len(data) == row["bytes"]
assert subprocess.check_output(
    ["git", "diff", "--name-only", identity["before"], source], cwd=code).decode().splitlines() == [row["path"]]
for path in identity["unchangedFiles"]:
    assert subprocess.check_output(["git", "rev-parse", identity["before"] + ":" + path], cwd=code) == (
        subprocess.check_output(["git", "rev-parse", source + ":" + path], cwd=code))
assert len(identity["sharedFiles"]) == 5
assert all(len(set(blobs.values())) == 1 for blobs in identity["sharedFiles"].values())
assert read("local-format.json")["exitCode"] == 0
sid = read("sid-method-test.original.json")
assert sid["result"] == "passed" and sid["checksPassed"] == len(sid["checks"]) == 4
assert sid["childExitedNormally"] and sid["nativeBeforeCount"] == sid["nativeAfterCount"] == 0
assert sid["harness"]["processId"] != sid["ownedChild"]["processId"]
assert sid["harness"]["sidSha256"] == sid["ownedChild"]["sidSha256"]
for item in (sid["harness"], sid["ownedChild"]):
    assert item["sourceMethodSha256"] == sha((root / "methods/process-user-sid.ps1").read_bytes())
    assert re.fullmatch("[0-9a-f]{64}", item["sidSha256"])
    assert not item["rawSidPublished"] and not item["nativeRootAcceptanceClaimed"]
assert not sid["targetApplicationObserved"] and not sid["dualPermissionOrSameRootAcceptanceProved"]
assert not sid["registryOrAccountOrAclChanged"] and not sid["productStarted"]
initial_sid = read("sid-method-initial-failure-analysis.json")
assert initial_sid["result"] == "initial-method-comparison-failed"
assert initial_sid["processIdsMatch"] and initial_sid["utcTicksMatch"] and initial_sid["sidDigestFormatValid"]
assert not initial_sid["productStarted"] and not initial_sid["registryOrAccountOrAclChanged"]
tests = {
    "installed_location_change_reuses_registered_anchor_and_its_reference_binding",
    "first_anchor_registration_failure_releases_locks_and_refuses_business_startup",
    "unreadable_or_future_pointer_stops_before_creating_installation_data",
    "uncertain_pointer_commit_is_reconciled_without_resetting_a_moved_install",
    "legacy_anchor_is_registered_without_changing_its_binding_or_external_home",
}
analyses = []
for identifier, event in ((38734133095, "push"), (38734138996, "pull_request")):
    terminal = read(f"run-{identifier}-terminal.original.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    assert run["id"] == identifier and run["event"] == event and run["head_sha"] == source
    assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
    assert terminal["jobs"]["total_count"] == len(jobs)
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        job, = [item for item in jobs if item["name"] == name]
        assert job["conclusion"] == "success"
    logs = read(f"run-{identifier}-log-verification.json")
    assert logs["source"] == source and logs["logFormat"] == "individual-job-originals"
    assert not logs["runWideZipObtained"] and not logs["archiveFabricated"]
    assert logs["rustResults"] == {
        "backend": {"suites": 35, "passed": 923, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 648, "failed": 0, "ignored": 7},
    }
    for job_name in ("backend", "backend-windows"):
        job, = [item for item in jobs if item["name"] == job_name]
        for step_name in ("Run cargo fmt --all -- --check",
                          "Run cargo clippy --locked --workspace --all-targets -- -D warnings"):
            assert next(item for item in job["steps"] if item["name"] == step_name)["conclusion"] == "success"
    assert len(logs["originalJobs"]) == 2
    assert {entry["job"] for entry in logs["originalJobs"]} == {"backend", "backend-windows"}
    for entry in logs["originalJobs"]:
        job, = [item for item in jobs if item["name"] == entry["job"]]
        metadata = read(entry["metadata"])
        assert metadata["source"] == source and metadata["runId"] == identifier
        assert metadata["job"]["id"] == entry["jobId"] == job["id"]
        assert metadata["job"]["name"] == entry["job"] and metadata["job"]["conclusion"] == "success"
        data = (root / entry["file"]).read_bytes()
        assert len(data) == entry["bytes"] == metadata["log"]["bytes"]
        assert sha(data) == entry["sha256"] == metadata["log"]["sha256"]
        text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
        actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert logs["actualCheckout"][entry["job"]] == actual
        counts = [tuple(map(int, row)) for row in re.findall(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
        assert logs["rustResults"][entry["job"]] == {
            "suites": len(counts), "passed": sum(row[0] for row in counts),
            "failed": sum(row[1] for row in counts), "ignored": sum(row[2] for row in counts)}
        passed = set(re.findall(r"test infrastructure::windows_startup_root::tests::(\w+) \.\.\. ok", text))
        assert tests <= passed
    saved = root / f"run-{identifier}"
    download = json.loads((saved / "download-verification.json").read_text())
    metadata = json.loads((saved / "artifact.original.json").read_text())
    archive_path = saved / "artifact.original.zip"
    assert sha(archive_path.read_bytes()) == download["artifactSha256"] == metadata["digest"][7:]
    assert archive_path.stat().st_size == metadata["size_in_bytes"]
    assert metadata["workflow_run"]["id"] == identifier and metadata["workflow_run"]["head_sha"] == source
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for entry in download["selectedEntries"]:
            stored = (saved / "selected-original" / entry["savedAs"]).read_bytes()
            assert sha(stored) == entry["savedSha256"] and len(stored) == entry["savedBytes"]
            data = gzip.decompress(stored) if entry["savedAs"].endswith(".jsonl.gz") else stored
            assert data == archive.read(entry["path"]) and sha(data) == entry["sha256"]
    process = subprocess.run([sys.executable, "-B", str(root / "analyze-first-ci.py"), str(identifier)],
                             capture_output=True, text=True, check=True)
    analysis = json.loads((saved / "analysis.json").read_text())
    assert json.loads(process.stdout) == analysis
    assert analysis["libraryExitCode"] == 0
    assert analysis["libraryResult"] == {"passed": 551, "failed": 0, "ignored": 4}
    assert not analysis["queryFailureObservedInCurrentBuild"]
    assert not analysis["historicalFailuresCleared"] and not analysis["rootCauseDetermined"]
    analyses.append(analysis)
tree = read("pr-actual-tree-verification.json")
assert tree["source"] == source and tree["headTree"] == tree["prActualTree"] and tree["treesMatch"]
assert not tree["binaryEqualityInferred"]
assert read("remote-code-readback.json")["object"]["sha"] == source
protection = read("final-protection.original.json")
assert protection["source"] == source and protection["codeWorktreeClean"] and protection["nativeCount"] == 0
assert protection["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protection["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protection["originalTaskBytes"] == 3878 and len(protection["protectedSandboxHashes"]) == 4
for field in ("localProductOrOriginalLibraryOrFixedNodeStarted", "localEtwSessionStarted",
              "hostInstallerExecuted", "newAccountOrProfileCreated", "localStartupChoiceRegistryWritten",
              "permissionsOrSystemConfigurationChanged", "fullVersionGatePassed", "releaseRequested"):
    assert protection[field] is False
pr = read("pr-terminal.original.json")
assert pr["head"]["sha"] == source and pr["state"] == "open" and pr["draft"]
assert not pr["merged"] and pr["auto_merge"] is None
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
assert not any("__pycache__" in item["path"] for item in entries)
report = {
    "anchorPersistenceSourceStage": "passed", "source": source, "manifestFiles": len(entries),
    "currentCi": [38734133095, 38734138996], "bothCiRunsSuccessful": True,
    "newRootRegistrationTestsPerPlatform": len(tests),
    "macRustPerRun": {"passed": 923, "ignored": 6},
    "windowsRustPerRun": {"passed": 648, "ignored": 7},
    "readonlySidMethodChecks": 4, "actualApplicationSidOrDualPermissionObserved": False,
    "captures": [{"run": item["runId"], "events": item["recordedEvents"],
                  "bytes": item["eventBytes"], "librarySha256": item["actualLibrarySha256"]} for item in analyses],
    "realHkcuOrInstallRelocationAcceptance": "pending", "ordinaryAdministratorAcceptance": "pending",
    "oldA2CandidateRelabelled": False, "nativeCountAtCheckpoint": 0,
    "historicalFixedArtifactFailureWindowEtwObtained": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries and read(report_path) == report
print(json.dumps(report))
