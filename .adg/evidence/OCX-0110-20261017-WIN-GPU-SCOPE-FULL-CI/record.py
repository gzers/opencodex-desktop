"""Seal the same first-push CI terminal state and fresh-window preparation."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
RUNTIME = CODE / "test/runtime/gpu-helper-scope-fix-20261017T0216"
PREP = CODE / "test/runtime/gpu-next-window-prep-20261017T0232"
SOURCE = "fde1dbb5b1731c056bbfa72a420bf930133671ea"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-GPU-SCOPE-FULL-CI-20261017.json"


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
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    admission = read(ROOT / "original/ci-complete-admission.json")
    state = read(ROOT / "original/terminal.original.json")
    run, jobs = state["run"], state["jobs"]
    assert run["id"] == admission["runId"] == 38946198178
    assert run["head_sha"] == admission["source"] == SOURCE and run["event"] == "push"
    assert run["run_attempt"] == admission["attempt"] == 1
    assert run["status"] == admission["status"] == "completed"
    assert run["conclusion"] == admission["conclusion"] == "success"
    assert admission["wholeToolCiAdmitted"] and not admission["fullProductGateAccepted"]
    assert jobs["total_count"] == len(jobs["jobs"]) == 6
    required = {"windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    assert {j["name"] for j in jobs["jobs"] if j["conclusion"] == "success"} == required
    assert [j["name"] for j in jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
    counts = {}
    for row in admission["requiredJobLogs"]:
        data = (ROOT / "original" / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == row["actualCheckout"] == SOURCE
        job, = [j for j in jobs["jobs"] if j["id"] == row["jobId"]]
        assert job["name"] == row["job"] and job["head_sha"] == SOURCE and job["run_attempt"] == 1
        assert job["conclusion"] == "success"
        tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if tests:
            assert all(int(r[1]) == 0 for r in tests)
            counts[job["name"]] = {"groups": len(tests), "passed": sum(int(r[0]) for r in tests),
                                  "ignored": sum(int(r[2]) for r in tests)}
        if job["name"] == "frontend":
            assert re.search(r"Test Files\s+99 passed \(99\)", text)
            assert re.search(r"\bTests\s+535 passed \(535\)", text)
        if job["name"] == "backend-windows":
            smoke, = [s for s in job["steps"] if s["name"] == "Windows final EXE smoke"]
            assert smoke["conclusion"] == "success"
    assert counts == {"backend": {"groups": 35, "passed": 923, "ignored": 6},
                      "backend-windows": {"groups": 35, "passed": 648, "ignored": 7}}
    bindings = read(ROOT / "prior-bindings.json")
    for row in bindings:
        assert sha((PROJECT / row["path"]).read_bytes()) == row["sha256"]
    failure = read(PROJECT / ".adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FAILURE/verification.json")
    assert failure["actualGpuPreparationFailed"] and failure["methodCiAdmitted"]
    assert not failure["wholeCiAdmittedAtFrozenCheckpoint"]
    assert not failure["actualProductGpuRecordingExecuted"]
    assert failure["normalExitCode"] == 0 and failure["kernelExitTimeProven"]
    plan = read(ROOT / "preparation/plan.json")
    validation = read(ROOT / "preparation/validation.json")
    assert plan["toolSource"] == SOURCE and plan["toolCiRunId"] == run["id"]
    assert plan["toolAdmissionPath"] == str(RUNTIME / "ci-complete-admission.json")
    assert plan["freshDirectHumanSignalRequired"] and not plan["foregroundAuthorityGranted"]
    assert plan["foregroundPreviouslyReleased"] and not plan["actualControllerReady"]
    assert not plan["preparationDirectoryIsExecutionDirectory"]
    assert plan["freshCliProcessForPrepare"] and plan["prepareStdoutAndStderrSavedBeforeJudging"]
    assert not plan["retryPriorProductRun"] and not plan["stalePacketsOrHumanReceiptCopied"]
    assert plan["captureSeconds"] == 30 and plan["maximumCaptureSeconds"] == 45
    assert plan["allocationEvidenceObservedRequiredBeforeAllocationClaim"]
    assert plan["zeroEventsDoNotProveZeroRssOrRootCause"]
    assert not plan["sameVersionFormalPerformancePairPlanned"]
    assert validation["passed"] and validation["nativeCount"] == 0
    assert validation["missingHumanReceiptRefused"] and validation["methodOnlyCiRefused"]
    assert validation["previousSourceFullCiRefused"]
    assert not validation["foregroundAuthorityCreated"] and not validation["recordRequestCreated"]
    assert not validation["actualGpuExecutionValidated"]
    for row in validation["syntax"]:
        data = (ROOT / "preparation" / row["path"]).read_bytes()
        assert row["syntaxPassed"] and sha(data) == row["sha256"]
    common = (ROOT / "preparation/common.ps1").read_text(encoding="utf-8-sig")
    assert "foreground-released.json" in common and "This foreground window has already been released" in common
    binding_script = (ROOT / "preparation/bind-and-prepare.ps1").read_text(encoding="utf-8-sig")
    assert "RedirectStandardOutput=$true" in binding_script and "RedirectStandardError=$true" in binding_script
    assert "gpu-prepare.stdout.decoded.log" in binding_script and "gpu-prepare.stderr.decoded.log" in binding_script
    assert not (PREP / "human-window.json").exists() and not (PREP / "root-launch.json").exists()
    assert not (PREP / "record-request.json").exists() and not (PREP / "gpu-capture").exists()
    protection = read(ROOT / "preparation/protection-readback.json")
    assert protection["nativeCount"] == 0 and protection["passed"]
    assert protection["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert protection["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert protection["originalProfileId"] == "0ebd4e59-001000000005cf4b"
    assert protection["originalProfileCreationFileTime"] == "134366003622018367"
    assert not protection["hostProductStarted"] and not protection["hostTracingStarted"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-first-complete-scope-repair-ci-and-next-window-preparation",
            "source": SOURCE, "runId": run["id"], "attempt": 1, "requiredJobsPassed": 5,
            "mainOnlyBuildSkipped": True, "frontendTests": 535, "rust": counts,
            "wholeToolCiAdmitted": True, "previousFailureSnapshotRewritten": False,
            "nextWindowPreparationVerified": True, "freshHumanSignalStillRequired": True,
            "actualProductGpuPreparationAfterFixExecuted": False,
            "actualProductGpuRecordingExecuted": False, "hostProductStarted": False,
            "hostTracingStarted": False, "rssFailureCleared": False,
            "originalBd2FailureCleared": False, "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
admission = read(RUNTIME / "ci-complete-admission.json")
assert admission["wholeToolCiAdmitted"]
copies = []


def copy(path, relative):
    data = path.read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), path.name
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": relative, "runtimePath": str(path), "bytes": len(data), "sha256": sha(data)})


copy(RUNTIME / "ci-complete-admission.json", "original/ci-complete-admission.json")
copy(RUNTIME / admission["snapshot"], "original/terminal.original.json")
for row in admission["requiredJobLogs"]:
    copy(RUNTIME / row["path"], "original/" + row["path"])
for path in sorted(PREP.iterdir()):
    if path.is_file() and path.suffix in {".json", ".ps1"}:
        copy(path, "preparation/" + path.name)
write(ROOT / "copies.json", copies)
prior = [".adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FAILURE/manifest.json",
         ".adg/evidence/OCX-0110-20261017-WIN-STATE-FLOOR-REUSE/manifest.json"]
write(ROOT / "prior-bindings.json", [{"path": p, "sha256": sha((PROJECT / p).read_bytes())} for p in prior])
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
    "task_summary_digest": task["summary_digest"], "baseline_digest": task["baseline_digest"],
    "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-gpu-scope-repair-full-ci", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-gpu-next-window-preparation", "status": "passed",
         "evidence": relative + "/preparation/validation.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "fde首次完整CI五必需job及原日志同SHA核验；下一前台入口绑定新源/完整CI，独立Prepare保留双流，实际GPU未重试。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "采集工具作用域修复完整CI通过，下一窗口准备完成，原产品分配/RSS及全部门禁保持。",
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
