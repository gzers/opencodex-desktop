"""Verify source fix and bounded evidence; a verified archive is not acceptance."""
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


preview = sys.argv[1:] == ["--preview"]
assert not sys.argv[1:] or preview
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
if not preview:
    manifest = read("manifest.json")
    assert set(files) == {row["path"] for row in manifest["files"]}
    for row in manifest["files"]:
        data = files[row["path"]].read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
audit = read("source-audit.json")
assert audit["source"] == SOURCE and len(audit["changedPaths"]) == 6
assert audit["bothCodeWorktreesClean"] and all(audit["sharedFilesIdentical"].values())
assert audit["oldStartupBlobUnchangedThroughD94"] and audit["oldObservedMaximumExeCount"] == 2
assert not audit["exactOverlapDurationEstablished"] and not audit["oldFailureMarkedPassed"]
assert audit["uiTreeUnchangedFromD94"] and not audit["nativeRecoveryExecutedOnNewSource"]
for path, row in audit["sourceBlobs"].items():
    data = subprocess.run(["git", "-c", "core.longpaths=true", "-C", str(CODE),
                           "show", SOURCE + ":" + path], check=True, capture_output=True).stdout
    assert sha(data) == row["gitBlobSha256"]
old = ROOT.parent / "OCX-0110-20261015-WIN-FG-02-ROOT"
for name, expected in audit["oldEvidenceReferences"].items():
    assert sha((old / name).read_bytes()) == expected
expected_cases = {"commit-after-exit": 0, "no-commit": 42, "wrong-generation": 41,
                  "wrong-target": 46, "exit-deadline": 43}
for name in ("controlled-final-result.original.json", "ci-result.json"):
    result = read(name)
    assert result["status"] == "passed" and result["checks"] == 40
    assert result["controlledParentAndWaiterPairs"] == 5 and not result["cleanup"]
    assert {c["case"]: c["helperExitCode"] for c in result["cases"]} == expected_cases
    assert all(c["status"] == "passed" and not c["productStarted"] for c in result["cases"])
    assert not result["productStarted"] and not result["foregroundActivated"]
    assert not result["dataRootOrRegistryModified"] and not result["nativeRecoveryGatePassed"]
    assert not result["fullPerformanceGatePassed"] and not result["originalQueryDeadlinesChanged"]
for name in ("controlled-final-target.original.json", "ci-commit-after-exit-target.json"):
    assert not read(name)["sameParentGenerationAlive"]
local = read("local-checks.json")
assert local["status"] == "passed" and local["controlledProcessChecks"] == 40
assert local["rustProductionMetadataPassed"] and local["rustTestMetadataPassed"]
assert local["changedRustFormatPassed"] and not local["fullRustLinkCompletedLocally"]
assert local["localLinkMissingWindowsSdk"] and not local["nativeRecoveryGatePassed"]
assert all(p["errors"] == 0 for p in local["powershellParsing"])
for name in ("rust-metadata-final.original.log", "rust-final-test-metadata.original.log"):
    assert "error[" not in (ROOT / name).read_text(encoding="utf-8-sig")
assert "can't find crate for `std`" in (ROOT / "rust-metadata-first.original.log").read_text(encoding="utf-8-sig")
assert "could not open 'kernel32.lib'" in (ROOT / "rust-link-first.original.log").read_text(encoding="utf-8-sig")
smoke, job = read("smoke-verification.json"), read("smoke-job.original.json")
assert smoke["source"] == job["head_sha"] == SOURCE and smoke["actualCheckout"] == SOURCE
assert smoke["runId"] == 38838964154 and smoke["jobId"] == 116774382636
assert smoke["checks"] == 40 and job["conclusion"] == "success"
for prefix in ("smoke-job", "push-windows-backend"):
    data = (ROOT / (prefix + ".original.log")).read_bytes()
    receipt = read(prefix + "-download.json")
    assert len(data) == receipt["bytes"] and sha(data) == receipt["sha256"]
artifact = read("smoke-artifact.original.json")
archive = ROOT / "smoke-artifact.original.zip"
assert sha(archive.read_bytes()) == artifact["digest"][7:] == smoke["artifactSha256"]
assert len(archive.read_bytes()) == artifact["size_in_bytes"]
with zipfile.ZipFile(archive) as bundle:
    assert bundle.testzip() is None
    for row in smoke["entries"]:
        data = bundle.read(row["archivePath"])
        assert data == (ROOT / row["savedAs"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
eol = read("ci-source-eol-binding.json")
assert eol["source"] == SOURCE and eol["ciHashMatchesCrLfCheckout"]
source = subprocess.run(["git", "-C", str(CODE), "show",
                         SOURCE + ":apps/desktop/tauri/src/infrastructure/windows_restart.ps1"],
                        check=True, capture_output=True).stdout
assert sha(source) == eol["gitBlobSha256"] == read("controlled-final-result.original.json")["waiterSha256"]
assert sha(source.replace(b"\n", b"\r\n")) == eol["ciWorkingFileSha256"] == read("ci-result.json")["waiterSha256"]
failure = read("push-failure-classification.json")
assert failure["source"] == SOURCE and failure["jobConclusionRetained"] == "failure"
assert failure["failureKind"] == "dependency-download-transport" and failure["package"] == "phf_generator"
assert not failure["rustClippyCompleted"] and not failure["backendTestsExecuted"]
assert not failure["finalExecutableProduced"] and not failure["workflowRerunRequested"]
failed_job = read("push-windows-backend-job.original.json")
assert failed_job["id"] == failure["jobId"] == 116774382572 and failed_job["conclusion"] == "failure"
failed_log = (ROOT / "push-windows-backend.original.log").read_text(encoding="utf-8-sig")
assert "phf_generator/0.13.1/download" in failed_log and "Failed receiving HTTP2 data" in failed_log
failed_artifact = read("push-failure-artifact.original.json")
failed_archive = ROOT / "push-failure-artifact.original.zip"
assert sha(failed_archive.read_bytes()) == failed_artifact["digest"][7:] == failure["artifactSha256"]
assert len(failed_archive.read_bytes()) == failed_artifact["size_in_bytes"]
with zipfile.ZipFile(failed_archive) as bundle:
    assert bundle.testzip() is None
    name, = [n for n in bundle.namelist() if n.endswith("/test-executable-unavailable.txt")]
    assert bundle.read(name) == (ROOT / "push-test-executable-unavailable.original.txt").read_bytes()
pr = read("pr-after.original.json")
assert pr["head"]["sha"] == SOURCE and pr["draft"] and not pr["merged"] and pr["auto_merge"] is None
assert pr["body"] == (ROOT / "pr-description.md").read_text(encoding="utf-8").strip()
final_pr = read("pr-final-after.original.json")
assert final_pr["head"]["sha"] == SOURCE and final_pr["draft"] and not final_pr["merged"]
assert final_pr["auto_merge"] is None
assert final_pr["body"] == (ROOT / "pr-description-final.md").read_text(encoding="utf-8").strip()
ci = read("pr-ci-collection.json")
assert ci["source"] == SOURCE and ci["runId"] == 38838969102
assert not ci["companionPushFailureCleared"] and not ci["workflowRerunRequested"]
for item in ci["archivedJobs"]:
    if not item["logArchived"]:
        continue
    name = item["name"]
    data = (ROOT / ("pr-" + name + ".original.log")).read_bytes()
    receipt = read("pr-" + name + "-download.json")
    assert sha(data) == item["sha256"] == receipt["sha256"] and len(data) == receipt["bytes"]
    commit = read("checkout-" + item["actualCheckout"] + ".original.json")
    assert commit["sha"] == item["actualCheckout"] and commit["tree"]["sha"] == ci["headFullTree"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == item["actualCheckout"]
    if name == "frontend":
        assert re.findall(r"Tests\s+(\d+) passed", text) == ["535"]
    elif item["conclusion"] == "success":
        suites = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        assert len(suites) == 35 and all(int(row[1]) == 0 for row in suites)
        assert sum(int(row[0]) for row in suites) == (923 if name == "backend" else 650)
if ci["wholeCiSucceeded"]:
    assert ci["status"] == "completed" and ci["conclusion"] == "success"
    assert all(j["status"] == "completed" and j["conclusion"] == "success"
               for j in ci["jobs"] if j["name"] != "build")
    build, = [j for j in ci["jobs"] if j["name"] == "build"]
    assert build["status"] == "completed" and build["conclusion"] == "skipped"
    windows_job = read("pr-backend-windows-job.original.json")
    for step_name in ("Run cargo clippy --locked --workspace --all-targets -- -D warnings",
                      "Windows backend regression",
                      "Run cargo build --locked --release --features custom-protocol --bins",
                      "Windows final EXE smoke"):
        step, = [s for s in windows_job["steps"] if s["name"] == step_name]
        assert step["status"] == "completed" and step["conclusion"] == "success"
push_terminal = read("push-ci-terminal.original.json")
assert push_terminal["run"]["head_sha"] == SOURCE and push_terminal["run"]["conclusion"] == "failure"
assert push_terminal["run"]["status"] == "completed"
assert [j["name"] for j in push_terminal["jobs"]["jobs"] if j["conclusion"] == "failure"] == ["backend-windows"]
protect = read("final-protection.json")
assert protect["nativeCount"] == 0 and protect["dailyExeUnchanged"] and protect["taskUnchanged"]
assert protect["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protect["fixturePreferencesAndRuntimeUnchanged"] and not protect["productStarted"]
assert not protect["etwReplayExecuted"] and not protect["candidateBuildRequested"]
summary = {
    "status": "preview-cross-checks-passed" if preview else "verified-recovery-source-fix-evidence",
    "files": len(files), "source": SOURCE,
    "localControlledChecks": 40, "ciControlledChecks": 40,
    "prCiRunId": ci["runId"], "prCiStatus": ci["status"],
    "prCiConclusion": ci["conclusion"], "wholeCiSucceeded": ci["wholeCiSucceeded"],
    "companionPushFailureRetained": True, "fullRustBuildCompletedLocally": False,
    "nativeProductStarted": False, "nativeRecoveryGatePassed": False,
    "newCandidateBuilt": False, "fullPerformanceGatePassed": False,
    "historicalFailuresCleared": False, "releaseAccepted": False,
}
if not preview:
    (ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
