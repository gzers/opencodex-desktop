"""Preserve fdb's first regression failure and the unchanged-budget test revision."""
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
BINDING = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop/test/runtime/memory-target-binding-20261017T0010")
IPC = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/ipc-idle-signal-20261017T0035")
FAILED = "fdb7e3cb5363b0bf17ee6402035668703078eab0"
FIX = "ca89fbac763d2376c6bfafcdfab4c256359d2a6b"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-BINDING-IPC-FAILURE-20261017.json"


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
    admission = read(ROOT / "binding/method-admission.json")
    assert admission["source"] == FAILED and admission["runId"] == 38930174628
    assert admission["readOnlyBindingJobSucceeded"] and admission["checks"] == 16
    assert not admission["wholeCiAdmitted"] and not admission["actualProductGpuBindingExecuted"]
    for folder in ["validation", "ci-binding"]:
        result = read(ROOT / "binding" / folder / "result.json")
        assert result["passed"] and result["checks"] == 16 and result["actualCurrentTokenUserMatched"]
        assert result["independentRetainedLeasesTested"] and result["generationTickMismatchRefused"]
        assert not result["productGpuBindingExecuted"] and not result["tracingStarted"] and not result["productStarted"]
    state = read(ROOT / "binding/terminal.original.json")
    assert state["run"]["head_sha"] == FAILED and state["run"]["id"] == 38930174628
    assert state["run"]["run_attempt"] == 1 and state["run"]["status"] == "completed"
    assert state["run"]["conclusion"] == "failure"
    job = read(ROOT / "binding/failed-backend-job.original.json")
    assert job["head_sha"] == FAILED and job["id"] == 117071843002 and job["conclusion"] == "failure"
    step, = [s for s in job["steps"] if s["conclusion"] == "failure"]
    assert step["name"] == "Windows backend regression"
    log = (ROOT / "binding/failed-backend-windows.original.log").read_bytes()
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == FAILED
    tests = ["cancelled_sync_observer_keeps_admissions_until_terminal_and_audit_for_each_result",
             "cancelled_backup_observer_keeps_admission_through_real_terminal_and_audit"]
    assert all(name in text for name in tests)
    assert "called `Result::unwrap()` on an `Err` value: Elapsed(())" in text
    assert "549 passed; 2 failed; 4 ignored" in text
    artifact = read(ROOT / "binding/failed-artifact-verification.json")
    assert artifact["source"] == FAILED and artifact["artifactId"] == 12059460774
    assert artifact["artifactSha256"] == "7d536e172199b17bbfbcc5706bc4eabd71bd6550faee201df9ba1422f8b5308b"
    assert artifact["bytes"] == 8875175 and artifact["crcVerified"] and not artifact["binariesExecuted"]
    private = read(ROOT / "private-failure-artifact.json")
    archive = Path(private["runtimePath"])
    assert sha(archive.read_bytes()) == artifact["artifactSha256"]
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        library, = [r for r in artifact["members"] if r["path"].endswith(".exe")]
        assert library["sha256"] == "0490fb14c01cd88b4384dfd304e063239e40f6171e5523c65e1059f236babb93"
        assert sha(zipped.read(library["path"])) == library["sha256"]
        assert library["sha256"] != "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
    fmt = read(ROOT / "ipc/failed-backend-job.original.json")
    assert fmt["head_sha"] == "ea8d359543f32d326d457b521d1a0d963ac9e39e" and fmt["conclusion"] == "failure"
    fmt_step, = [s for s in fmt["steps"] if s["conclusion"] == "failure"]
    assert fmt_step["name"] == "Run cargo fmt --all -- --check"
    analysis = read(ROOT / "binding/ipc-revision-source-analysis.json")
    assert analysis["failedSource"] == FAILED and analysis["revision"] == FIX
    assert analysis["failedAndB4ServiceBytesEqual"] and analysis["auditAndTerminalAssertionsPreserved"]
    assert analysis["twoSecondTimeoutCountBefore"] == analysis["twoSecondTimeoutCountAfter"] == 24
    assert len(analysis["onlyNamedWaitLoopsChanged"]) == 2
    assert not analysis["defaultTestConcurrencyOverridden"] and not analysis["localRustCompilationPerformed"]
    assert not analysis["rootCauseEstablished"] and not analysis["oldCiRerunRequested"]
    current = read(ROOT / "ipc/windows-regression-step.original.json")
    assert current["head_sha"] == FIX and current["run_id"] == 38933650482 and current["run_attempt"] == 1
    regression, = [s for s in current["steps"] if s["name"] == "Windows backend regression"]
    assert regression["status"] == "completed" and regression["conclusion"] == "success"
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-first-failure-and-pending-revision", "failedSource": FAILED,
            "failedCiRunId": 38930174628, "bindingChecksPassed": 16,
            "originalLibraryPassed": 549, "originalLibraryFailed": 2, "ignored": 4,
            "failureRootCauseEstablished": False, "fixedSource": FIX,
            "fixedWindowsRegressionStepSucceeded": True, "fixedWholeCiAdmitted": False,
            "deadlinesAndAssertionsPreserved": True, "defaultConcurrencyPreserved": True,
            "failedBinariesExecuted": False, "originalBd2FailureCleared": False,
            "rssFailureCleared": False, "fullProductGatePassed": False}


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


for path in sorted(BINDING.iterdir()):
    if path.is_file() and path.suffix in {".json", ".log", ".py"}:
        copy(path, "binding/" + path.name)
for folder in ["validation", "ci-binding"]:
    for path in sorted((BINDING / folder).iterdir()):
        if path.is_file():
            copy(path, "binding/" + folder + "/" + path.name)
terminal = sorted(p for p in BINDING.glob("ci-status-*.original.json")
                  if read(p)["run"]["status"] == "completed" and read(p)["run"]["conclusion"] == "failure")[-1]
copy(terminal, "binding/terminal.original.json")
for path in sorted(IPC.iterdir()):
    if path.is_file() and path.suffix in {".json", ".log"}:
        copy(path, "ipc/" + path.name)
windows = sorted(IPC.glob("ci-ca89fbac-windows-*.original.json"))[-1]
copy(windows, "ipc/windows-regression-step.original.json")
write(ROOT / "copies.json", copies)
archive = BINDING / "failed-windows-backend.original.zip"
write(ROOT / "private-failure-artifact.json", {"runtimePath": str(archive), "sha256": sha(archive.read_bytes()),
                                               "bytes": archive.stat().st_size, "rawZipPublished": False,
                                               "binariesExecuted": False})
result = verify()
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object
task = read(TASK)
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
relative = ROOT.relative_to(PROJECT).as_posix()
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"], "task_digest": task["content_digest"],
    "task_summary_digest": task["summary_digest"], "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-memory-target-read-only-method", "status": "passed",
         "evidence": relative + "/binding/method-admission.json"},
        {"name": "local:windows-fdb-full-ci", "status": "failed",
         "evidence": relative + "/binding/terminal.original.json"},
        {"name": "local:windows-ipc-revision-regression-step", "status": "passed",
         "evidence": relative + "/ipc/windows-regression-step.original.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "fdb只读方法16通过但整轮IPC两项超时失败原件保留；ea8格式失败后ca89保持2秒/断言的测试通知修订已过Windows回归步骤，整轮待核。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "记录首轮失败与未降预算的夹具修订进展，不清除原bd2、RSS或其它门禁。",
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
