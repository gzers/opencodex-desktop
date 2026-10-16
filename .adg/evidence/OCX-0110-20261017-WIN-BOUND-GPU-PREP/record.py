"""Freeze complete tool CI and preparation; do not claim a real GPU capture."""
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
RUNTIME = CODE / "test/runtime/bound-gpu-memory-prep-20261017T0020"
SOURCE = "391e70979ffbb32a548c0d05bdeeb62160d942d3"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-BOUND-GPU-PREP-20261017.json"


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
    original = ROOT / "original"
    admission = read(original / "ci-complete-admission.json")
    state = read(original / "terminal.original.json")
    run, jobs = state["run"], state["jobs"]
    assert run["id"] == admission["runId"] == 38934816693
    assert run["head_sha"] == admission["source"] == SOURCE and run["event"] == "push"
    assert run["run_attempt"] == 1 and run["status"] == "completed" and run["conclusion"] == "success"
    assert admission["wholeToolCiAdmitted"] and not admission["fullProductGateAccepted"]
    assert jobs["total_count"] == len(jobs["jobs"]) == 6
    required = {"windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    assert {j["name"] for j in jobs["jobs"] if j["conclusion"] == "success"} == required
    assert [j["name"] for j in jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
    counts = {}
    for row in admission["requiredJobLogs"]:
        data = (original / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == row["actualCheckout"] == SOURCE
        job, = [j for j in jobs["jobs"] if j["id"] == row["jobId"]]
        assert job["name"] == row["job"] and job["head_sha"] == SOURCE
        tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if tests:
            assert all(int(r[1]) == 0 for r in tests)
            counts[job["name"]] = {"groups": len(tests), "passed": sum(int(r[0]) for r in tests),
                                  "ignored": sum(int(r[2]) for r in tests)}
        if job["name"] == "frontend":
            assert re.search(r"Test Files\s+99 passed \(99\)", text)
            assert re.search(r"\bTests\s+535 passed \(535\)", text)
    assert counts == {"backend": {"groups": 35, "passed": 923, "ignored": 6},
                      "backend-windows": {"groups": 35, "passed": 648, "ignored": 7}}
    method = read(original / "method-admission.json")
    assert method["source"] == SOURCE and method["runId"] == run["id"]
    assert method["localOrdinaryOuterChecks"] == 10 and method["ciAdministratorOuterChecks"] == 15
    assert method["compiledGuardChecks"] == 20 and method["bindingChecksEach"] == 18
    assert method["entryRefusalChecks"] == 8
    assert not method["wholeCiAdmitted"] and method["wholeCiStatusAtFrozenRead"] == "in_progress"
    assert not method["actualProductGpuBindingExecuted"] and not method["hostTracingStarted"]
    for row in method["sourceFiles"]:
        blob = subprocess.check_output(["git", "show", SOURCE + ":" + row["path"]], cwd=CODE)
        assert b"\r\n" not in blob
        assert sha(blob) == row["git-lf"] and sha(blob.replace(b"\n", b"\r\n")) == row["git-checkout-crlf"]
    artifact = read(original / "ci-method-download.json")
    assert artifact["artifactId"] == 12059879326 and artifact["artifactBytes"] == 76710
    assert artifact["artifactSha256"] == "62f039d713e53d586e1b5ddf01c286cb88ba5dc735f12d183c9e51bc2c7254d1"
    private = read(ROOT / "private-method-artifact.json")
    archive = Path(private["runtimePath"])
    assert archive.stat().st_size == artifact["artifactBytes"] and sha(archive.read_bytes()) == artifact["artifactSha256"]
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        assert len(artifact["extracted"]) == 11
        for row in artifact["extracted"]:
            data = (original / row["path"]).read_bytes()
            assert data == zipped.read(row["member"]) and sha(data) == row["sha256"]
    ci = original / "ci-method-extracted"
    controlled = read(ci / "controlled-memory/result.json")
    local = read(original / "final-controlled-tests/result.json")
    assert controlled["passed"] and controlled["checks"] == 15 and controlled["administratorToken"]
    assert controlled["controlledRecordingPassed"] and not controlled["ordinaryTokenRefusalTested"]
    assert local["passed"] and local["checks"] == 10 and local["ordinaryTokenRefusalTested"]
    assert not local["administratorToken"] and local["compiledGuardChecks"] == 20
    for path in (ci / "memory-target/result.json", original / "final-binding-tests/result.json"):
        binding = read(path)
        assert binding["passed"] and binding["checks"] == 18
        assert binding["actualCurrentTokenUserMatched"] and binding["independentRetainedLeasesTested"]
        assert binding["generationTickMismatchRefused"]
        assert not binding["productGpuBindingExecuted"] and not binding["tracingStarted"]
    entry = read(original / "entry-refusal-result.json")
    assert entry["passed"] and entry["checks"] == 8
    assert all(entry[k] for k in ("ordinaryControllerRefused", "native0PrepareRefused", "oldFullCiRefused", "partialCiRefused"))
    capture = read(ci / "controlled-memory/capture/result.json")
    events = (ci / "controlled-memory/capture/owned-memory.events.jsonl").read_bytes()
    rows = [json.loads(line) for line in events.splitlines()]
    assert capture["passed"] and capture["controlledSelf"] and not capture["boundGpuObservation"]
    assert len(rows) == capture["records"] == 1225 and len(events) == capture["serializedBytes"] == 746512
    assert capture["probeBytes"] == 8388608 and capture["probeFreed"]
    assert capture["matchingProbeAllocationEvents"] == capture["matchingAllocationStacks"] == 1
    assert all(row["pid"] == capture["selfPid"] == capture["targetPid"] for row in rows)
    probe, = [r for r in rows if r["kind"] == "virtual-allocation" and r["length"] == 8388608]
    stack, = [r for r in rows if r["kind"] == "allocation-stack" and r["tid"] == probe["tid"]
              and r["eventRelativeMilliseconds"] == probe["relativeMilliseconds"]]
    assert stack["frameCount"] > 0
    assert capture["omittedRecords"] == capture["callbackErrors"] == 0
    assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
    assert capture["ownedSessionAbsent"] and capture["readerJoined"] and not capture["stopProcessingUsed"]
    assert not capture["cleanupErrors"] and capture["captureError"] is None and capture["consumerError"] is None
    preflight = read(original / "diagnostic-preflight.original.json")
    assert preflight["passed"] and preflight["preparationOnly"]
    assert preflight["nativeCountBefore"] == preflight["nativeCountAfter"] == 0
    assert preflight["toolSource"] == SOURCE
    assert preflight["productExeSha256"] == "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"
    assert preflight["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert preflight["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert not any(preflight[k] for k in ("actualControllerElevated", "freshHumanForegroundSignalPresent",
                                        "productStarted", "uiInputInjected", "tracingStarted",
                                        "selfElevationPerformed", "originalProfileMoved", "configurationWritten"))
    plan = read(original / "next-diagnostic-plan.json")
    assert plan["preparationOnly"] and plan["freshDirectHumanSignalRequired"]
    assert not plan["foregroundAuthorityGranted"] and not plan["actualControllerReady"]
    assert plan["plannedForegroundMinutes"] == 15 and plan["maximumHumanWindowMinutes"] == 30
    assert plan["captureSeconds"] == 30 and plan["maximumCaptureSeconds"] == 45
    assert plan["allocationEvidenceObservedRequiredBeforeAllocationClaim"]
    assert plan["zeroEventsDoNotProveZeroRssOrRootCause"]
    assert not plan["sameVersionFormalPerformancePairPlanned"] and not plan["dpiAcceptancePlanned"]
    bindings = read(ROOT / "historical-bindings.json")
    for row in bindings:
        assert sha((PROJECT / row["path"]).read_bytes()) == row["sha256"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-bound-gpu-preparation-and-first-complete-tool-ci",
            "source": SOURCE, "runId": run["id"], "attempt": 1, "requiredJobsPassed": 5,
            "frontendTests": 535, "rust": counts, "wholeToolCiAdmitted": True,
            "localOrdinaryChecks": 10, "ciAdministratorChecks": 15, "compiledGuards": 20,
            "bindingChecksEach": 18, "entryRefusalChecks": 8, "controlledSelfAllocationStackPairs": 1,
            "actualProductGpuBindingExecuted": False, "actualProductGpuRecordingExecuted": False,
            "newForegroundAuthorityGranted": False, "hostProductStarted": False,
            "hostTracingStarted": False, "oldMethodSnapshotRewritten": False,
            "rssFailureCleared": False, "originalBd2FailureCleared": False,
            "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []


def copy(path, relative):
    data = path.read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), path.name
    target = ROOT / "original" / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": target.relative_to(ROOT).as_posix(), "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})


admission = read(RUNTIME / "ci-complete-admission.json")
selected = [
    "ci-handle.json", "ci-method-job-and-artifacts.original.json", "ci-method-download.json",
    "method-admission.json", "ci-complete-admission.json", "entry-refusal-result.json",
    "ordinary-entry-refusal.decoded.log", "native0-entry-refusal.decoded.log",
    "entry-refusal-checks.ps1", "final-binding-tests.original.log", "final-controlled-tests.original.log",
    "diagnostic-preflight.ps1", "diagnostic-preflight.original.json", "next-diagnostic-plan.json",
    "verify-method.py", "collect-existing-ci.py", "remote-readback.json",
]
selected += [row["path"] for row in admission["requiredJobLogs"]]
for name in dict.fromkeys(selected):
    copy(RUNTIME / name, name)
copy(RUNTIME / admission["snapshot"], "terminal.original.json")
for folder in ("ci-method-extracted", "final-binding-tests", "final-controlled-tests"):
    for path in sorted((RUNTIME / folder).rglob("*")):
        if path.is_file() and path.suffix in {".json", ".jsonl", ".log", ".txt"}:
            copy(path, path.relative_to(RUNTIME).as_posix())
write(ROOT / "copies.json", copies)
archive = RUNTIME / "ci-method.original.zip"
write(ROOT / "private-method-artifact.json", {"runtimePath": str(archive),
                                            "sha256": sha(archive.read_bytes()),
                                            "bytes": archive.stat().st_size, "rawZipPublished": False})
historical = [
    ".adg/evidence/OCX-0110-20261017-WIN-BINDING-IPC-FAILURE/manifest.json",
    ".adg/evidence/OCX-0110-20261016-WIN-B4-RESTORATION/manifest.json",
    ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/manifest.json",
    ".adg/evidence/OCX-0110-20261016-WIN-B4-CANDIDATE/manifest.json",
]
write(ROOT / "historical-bindings.json", [
    {"path": p, "sha256": sha((PROJECT / p).read_bytes())} for p in historical])
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
        {"name": "local:windows-bound-gpu-tool-full-ci", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-bound-gpu-diagnostic-preparation", "status": "passed",
         "evidence": relative + "/original/diagnostic-preflight.original.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "391首次完整工具CI、受控自进程链路及只读前置通过；真实GPU未录制，下一15分钟计划需新人工信号和实际管理员采集器。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "GPU入口准备与完整工具CI通过，真实产品分配证据、RSS及原门禁未解除。",
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
