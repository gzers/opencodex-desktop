"""Seal the first complete IPC revision CI; preserve the first failed run."""
import hashlib
import json
import re
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/"
               "test/runtime/ipc-idle-signal-20261017T0035")
SOURCE = "ca89fbac763d2376c6bfafcdfab4c256359d2a6b"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-IPC-IDLE-FULL-CI-20261017.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    admission = read(ROOT / "original/ci-complete-admission.json")
    state = read(ROOT / "original/terminal.original.json")
    run, jobs = state["run"], state["jobs"]
    assert run["id"] == admission["runId"] == 38933650482
    assert run["head_sha"] == admission["source"] == SOURCE and run["event"] == "push"
    assert run["run_attempt"] == 1 and run["status"] == "completed" and run["conclusion"] == "success"
    assert admission["wholeToolCiAdmitted"] and not admission["fullProductGateAccepted"]
    assert jobs["total_count"] == len(jobs["jobs"]) == 6
    required = {"windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    assert {j["name"] for j in jobs["jobs"] if j["conclusion"] == "success"} == required
    assert [j["name"] for j in jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
    counts = {}
    named = ["modules::ipc::service::tests::cancelled_sync_observer_keeps_admissions_until_terminal_and_audit_for_each_result",
             "modules::ipc::service::tests::cancelled_backup_observer_keeps_admission_through_real_terminal_and_audit",
             "infrastructure::storage_writers::tests::idle_observer_wakes_after_final_admission_without_releasing_it"]
    for row in admission["requiredJobLogs"]:
        data = (ROOT / "original" / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == row["actualCheckout"] == SOURCE
        job, = [j for j in jobs["jobs"] if j["id"] == row["jobId"]]
        assert job["name"] == row["job"] and job["head_sha"] == SOURCE and job["run_attempt"] == 1
        tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if tests:
            assert all(int(r[1]) == 0 for r in tests)
            counts[job["name"]] = {"groups": len(tests), "passed": sum(int(r[0]) for r in tests),
                                  "ignored": sum(int(r[2]) for r in tests)}
            for name in named:
                assert re.search(r"\btest " + re.escape(name) + r" \.\.\. ok\b", text), name
        if job["name"] == "frontend":
            assert re.search(r"Test Files\s+99 passed \(99\)", text)
            assert re.search(r"\bTests\s+538 passed \(538\)", text)
        if job["name"] == "backend-windows":
            assert "test result: ok. 554 passed; 0 failed; 4 ignored;" in text
            smoke, = [s for s in job["steps"] if s["name"] == "Windows final EXE smoke"]
            assert smoke["conclusion"] == "success"
    assert counts == {"backend": {"groups": 35, "passed": 924, "ignored": 6},
                      "backend-windows": {"groups": 35, "passed": 651, "ignored": 7}}
    artifact = read(ROOT / "original/ci-complete-artifact-verification.json")
    assert artifact["source"] == SOURCE and artifact["runId"] == run["id"]
    assert artifact["artifactId"] == 12060483211 and artifact["bytes"] == 9583894
    assert artifact["artifactSha256"] == "30672f574edbb6997bf6e572913e786bcfaf8a0cd2be23182914ba2b7b5134c2"
    private = read(ROOT / "private-artifact.json")
    archive = Path(private["runtimePath"])
    assert sha(archive.read_bytes()) == artifact["artifactSha256"]
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        for row in artifact["members"]:
            data = zipped.read(row["path"])
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        library = artifact["library"]
        assert library["sha256"] == "a3c4afd1d64361c154236aec2972031e90116fd278af67b6ffa16b1d49fe0e62"
        assert sha(zipped.read(library["path"])) == library["sha256"]
    assert not artifact["binariesExecuted"] and not artifact["failedLibraryReplaced"]
    binding = read(ROOT / "first-failure-binding.json")
    prior = PROJECT / binding["directory"]
    assert sha((prior / "manifest.json").read_bytes()) == binding["manifestSha256"]
    previous = read(prior / "verification.json")
    assert previous["failedSource"] == "fdb7e3cb5363b0bf17ee6402035668703078eab0"
    assert previous["originalLibraryFailed"] == 2 and not previous["fixedWholeCiAdmitted"]
    analysis = read(prior / "binding/ipc-revision-source-analysis.json")
    assert analysis["twoSecondTimeoutCountBefore"] == analysis["twoSecondTimeoutCountAfter"] == 24
    assert analysis["auditAndTerminalAssertionsPreserved"] and not analysis["defaultTestConcurrencyOverridden"]
    assert not analysis["rootCauseEstablished"] and not analysis["localRustCompilationPerformed"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-first-complete-ipc-revision-ci", "source": SOURCE,
            "runId": run["id"], "attempt": 1, "requiredJobsPassed": 5, "frontendTests": 538,
            "rust": counts, "namedCancelledObserverTestsPassedBothPlatforms": True,
            "finalAdmissionWakeTestPassedBothPlatforms": True, "wholeCiAdmitted": True,
            "deadlinesAndAssertionsPreserved": True, "oldFailureCausalityProved": False,
            "oldFailureCleared": False, "binariesExecuted": False, "hostProductStarted": False,
            "originalBd2FailureCleared": False, "rssFailureCleared": False,
            "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []


def copy(path, relative):
    data = path.read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), path.name
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": relative, "runtimePath": str(path), "bytes": len(data), "sha256": sha(data)})


admission = read(RUNTIME / "ci-complete-admission.json")
copy(RUNTIME / admission["snapshot"], "original/terminal.original.json")
for path in sorted(RUNTIME.glob("ci-complete-*")):
    if path.is_file() and path.suffix in {".json", ".log"}:
        copy(path, "original/" + path.name)
write(ROOT / "copies.json", copies)
archive = RUNTIME / "ci-complete-windows-startup.original.zip"
write(ROOT / "private-artifact.json", {"runtimePath": str(archive), "sha256": sha(archive.read_bytes()),
                                      "bytes": archive.stat().st_size, "rawZipPublished": False,
                                      "binariesExecuted": False})
prior = ROOT.parent / "OCX-0110-20261017-WIN-BINDING-IPC-FAILURE"
write(ROOT / "first-failure-binding.json", {
    "directory": prior.relative_to(PROJECT).as_posix(),
    "manifestSha256": sha((prior / "manifest.json").read_bytes()),
})
result = verify()
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object
task = read(TASK)
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
relative = ROOT.relative_to(PROJECT).as_posix()
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-ipc-idle-revision-full-ci", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "ca89首次完整CI与五份同SHA原日志通过，指定两失败测试和最终许可通知用例通过；未证明旧失败因果或解除原门禁。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "IPC测试专用通知修订首次完整CI通过，首轮失败及原预算/周期保持。",
    "revision": 1,
})
write(RECEIPT, receipt)
files = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        files.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": files})
verify()
print(json.dumps({"originalCopies": len(copies), "receiptResult": receipt["result"], **result}))
