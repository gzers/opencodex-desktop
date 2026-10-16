"""Archive controlled allocation proof without promoting any product gate."""
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
RUNTIME = CODE / "test/runtime/controlled-memory-validation-20261016T2330"
PREVIOUS = CODE / "test/runtime/memory-profile-prep-20261016T2300"
SOURCE = "a5a1aba290793b23c6d27c35edd0ab227d4f5e50"
PROFILE_SOURCE = "494fc830e30cc6b56cf62f34884d44e23bfb763d"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-CONTROLLED-MEMORY-PROOF-20261016.json"


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
    for row in read(ROOT / "source-identity.json")["files"]:
        blob = subprocess.check_output(["git", "show", SOURCE + ":" + row["path"]], cwd=CODE)
        assert sha(blob) == row["gitLfSha256"]
        assert sha(blob.replace(b"\n", b"\r\n")) == row["gitCheckoutCrLfSha256"]
    admission = read(ROOT / "original/method-admission.json")
    assert admission["source"] == admission["actualCheckout"] == SOURCE
    assert admission["runId"] == 38925613291 and admission["jobId"] == 117056254920
    assert admission["controlledSelfCaptureJobSucceeded"] and not admission["wholeCiAdmitted"]
    assert not admission["hostRecordingExecuted"] and not admission["productAllocationAttributed"]
    api = read(ROOT / "original/ci-method-job-and-artifacts.original.json")
    assert api["run"]["head_sha"] == api["job"]["head_sha"] == SOURCE
    assert api["run"]["event"] == "push" and api["run"]["run_attempt"] == api["job"]["run_attempt"] == 1
    assert api["job"]["status"] == "completed" and api["job"]["conclusion"] == "success"
    step, = [s for s in api["job"]["steps"] if s["name"] == "Verify controlled self-process memory allocation capture"]
    assert step["status"] == "completed" and step["conclusion"] == "success"
    local = read(ROOT / "original/lf-source-tests/result.json")
    result = read(ROOT / "original/ci-controlled/result.json")
    capture = read(ROOT / "original/ci-controlled/capture/result.json")
    assert local["passed"] and local["checks"] == 10 and local["compiledGuardChecks"] == 20
    assert local["ordinaryTokenRefusalTested"] and not local["administratorToken"]
    assert not local["controlledRecordingRequested"] and not local["controlledRecordingPassed"]
    assert result["passed"] and result["checks"] == 15 and result["compiledGuardChecks"] == 20
    assert result["administratorToken"] and not result["ordinaryTokenRefusalTested"]
    assert result["controlledRecordingRequested"] and result["controlledRecordingPassed"]
    assert capture["passed"] and capture["probeBytes"] == 8388608 and capture["probeFreed"]
    assert capture["nativeStop"]["Status"] == 0
    assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
    assert capture["readerJoined"] and not capture["stopProcessingUsed"] and capture["ownedSessionAbsent"]
    assert capture["retainedSelfGenerationUnchanged"] and capture["retainedSelfProcessHandle"]
    assert capture["omittedRecords"] == capture["callbackErrors"] == 0 and capture["cleanupErrors"] == []
    for key in ["genericSessionStopInvoked", "globalSampleIntervalExplicitlyWritten",
                "globalSampleIntervalBeforeAfterMeasured", "systemWideEtlWritten",
                "unrelatedEventPayloadsSerialized", "rawAllocationAddressSaved", "targetMemoryContentsSaved",
                "nativeDesktopStarted", "originalTestLibraryExecuted", "officialQueryNodeExecuted",
                "fullProductGatePassed"]:
        assert not capture[key]
    raw = (ROOT / "original/ci-controlled/capture/owned-memory.events.jsonl").read_bytes()
    assert b"\r" not in raw and len(raw) == capture["serializedBytes"] == 723913
    assert len(raw) <= capture["maximumSerializedBytes"]
    rows = [json.loads(line) for line in raw.splitlines()]
    assert len(rows) == capture["records"] == 1217
    assert all(r["pid"] == capture["selfPid"] for r in rows)
    allocations = [r for r in rows if r["kind"] == "virtual-allocation" and r["length"] == 8388608]
    stacks = [r for r in rows if r["kind"] == "allocation-stack"]
    matched = [a for a in allocations if any(
        s["tid"] == a["tid"] and s["eventRelativeMilliseconds"] == a["relativeMilliseconds"]
        and s["frameCount"] > 0 for s in stacks)]
    assert len(allocations) == len(matched) == capture["matchingProbeAllocationEvents"] == capture["matchingAllocationStacks"] == 1
    assert all(s["frameCount"] == len(s["instructionPointersHex"]) <= 128 for s in stacks)
    files = {r["path"]: r for r in read(ROOT / "source-identity.json")["files"]}
    for proof in [local, result]:
        for label, name in [("recorder", "windows-controlled-memory.cs"), ("wrapper", "windows-controlled-memory.ps1"),
                            ("sharedNativeStop", "windows-owned-etw.cs")]:
            source = files["test/smoke/" + name]
            assert proof["sourceSha256"][label] in {source["gitLfSha256"], source["gitCheckoutCrLfSha256"]}
    descriptor = read(ROOT / "private-artifact.json")
    archive = Path(descriptor["runtimePath"])
    assert sha(archive.read_bytes()) == descriptor["sha256"] == admission["artifactSha256"]
    assert archive.stat().st_size == descriptor["bytes"] == admission["artifactBytes"]
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        for name in ["result.json", "capture/result.json", "capture/owned-memory.events.jsonl"]:
            member, = [m for m in zipped.infolist() if m.filename.endswith("controlled-memory/" + name)]
            assert zipped.read(member) == (ROOT / "original/ci-controlled" / name).read_bytes()
    log = (ROOT / "original/ci-method-job.original.log").read_bytes()
    assert sha(log) == admission["jobLogSha256"]
    actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log.decode("utf-8-sig"))
    assert actual == SOURCE
    terminal = read(ROOT / "previous-ci/admission.json")
    assert terminal["source"] == PROFILE_SOURCE and terminal["runId"] == 38920950285
    assert terminal["status"] == "completed" and terminal["conclusion"] == "success" and terminal["wholeToolCiAdmitted"]
    previous_api = read(ROOT / "previous-ci/terminal.original.json")
    previous_run, previous_jobs = previous_api["run"], previous_api["jobs"]
    assert previous_run["id"] == 38920950285 and previous_run["head_sha"] == PROFILE_SOURCE
    assert previous_run["event"] == "push" and previous_run["run_attempt"] == 1
    assert previous_run["status"] == "completed" and previous_run["conclusion"] == "success"
    assert previous_jobs["total_count"] == len(previous_jobs["jobs"]) == 6
    assert all(j["status"] == "completed" and j["head_sha"] == PROFILE_SOURCE for j in previous_jobs["jobs"])
    assert {j["name"] for j in previous_jobs["jobs"] if j["conclusion"] == "success"} == {
        "windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    assert [j["name"] for j in previous_jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
    assert {r["job"] for r in terminal["requiredJobLogs"]} == {
        "windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    for item in terminal["requiredJobLogs"]:
        data = (ROOT / "previous-ci" / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and sha(data) == item["sha256"]
        actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", data.decode("utf-8-sig"))
        assert actual == item["actualCheckout"] == PROFILE_SOURCE
    state = read(ROOT / "original/final-readback.original.json")
    assert state["nativeCount"] == 0 and state["toolSource"] == SOURCE
    assert state["originalDirectory"]["fileId"] == "0ebd4e59-001000000005cf4b"
    assert state["originalDirectory"]["creationFileTime"] == "134366003622018367"
    assert state["preferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert state["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert state["taskFileSha256"] == sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    assert not state["hostEtwRecordingStarted"] and not state["nativeProductStarted"]
    if (ROOT / "manifest.json").exists():
        for item in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / item["path"]).read_bytes()
            assert len(data) == item["bytes"] and sha(data) == item["sha256"]
    return {"status": "verified-controlled-self-allocation-only", "source": SOURCE,
            "localOuterChecks": 10, "compiledGuardChecks": 20, "ciOuterChecks": 15,
            "probeBytes": 8388608, "allocationStackPairs": 1, "records": 1217,
            "eventBytes": 723913, "zeroLossOmissionOrDecodeErrors": True,
            "probeFreed": True, "ownedSessionAbsent": True, "readerDrained": True,
            "previousToolFullCiPassed": True, "currentToolWholeCiAdmitted": False,
            "nativeCountAtReadback": 0, "hostRecordingExecuted": False,
            "productAllocationCauseAttributed": False, "originalBd2FailureCleared": False,
            "fullProductGatePassed": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []


def copy(path, relative):
    data = path.read_bytes()
    text = data.decode("utf-8-sig")
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", text), relative
    assert not re.search(r"\bgh[pousr]_[A-Za-z0-9]{20,}", text), relative
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": relative, "runtimePath": str(path), "bytes": len(data), "sha256": sha(data)})


for path in sorted(RUNTIME.iterdir()):
    if path.is_file() and path.suffix in {".json", ".log", ".py"}:
        copy(path, "original/" + path.name)
for folder in ["ci-controlled", "lf-source-tests", "metadata-fix-tests", "wrapper-tests"]:
    for path in sorted((RUNTIME / folder).rglob("*")):
        if path.is_file() and path.suffix in {".json", ".log"}:
            copy(path, "original/" + path.relative_to(RUNTIME).as_posix())
        elif path.name == "owned-memory.events.jsonl":
            copy(path, "original/" + path.relative_to(RUNTIME).as_posix())
old_ci = read(PREVIOUS / "ci-terminal-admission.json")
copy(PREVIOUS / "ci-terminal-admission.json", "previous-ci/admission.json")
previous_api, = [
    p for p in PREVIOUS.glob("ci-*.original.json")
    if re.fullmatch(r"ci-\d{8}T\d+Z.original.json", p.name)
    and read(p)["run"]["id"] == 38920950285
    and read(p)["run"]["status"] == "completed"
]
copy(previous_api, "previous-ci/terminal.original.json")
for item in old_ci["requiredJobLogs"]:
    copy(PREVIOUS / item["path"], "previous-ci/" + item["path"])
write(ROOT / "copies.json", copies)
archive = RUNTIME / "ci-method.original.zip"
write(ROOT / "private-artifact.json", {"runtimePath": str(archive), "bytes": archive.stat().st_size,
                                      "sha256": sha(archive.read_bytes()), "rawZipPublished": False})
snapshots = []
for name in ["windows-controlled-memory-before-metadata-fix.cs", "windows-owned-etw-before-lf-normalization.cs"]:
    path = RUNTIME / name
    snapshots.append({"runtimePath": str(path), "bytes": path.stat().st_size, "sha256": sha(path.read_bytes()),
                      "implementationCopiedIntoDocs": False})
write(ROOT / "private-source-snapshots.json", snapshots)
source_files = []
for name in ["test/smoke/windows-controlled-memory.cs", "test/smoke/windows-controlled-memory.ps1",
             "test/smoke/windows-controlled-memory.test.ps1", "test/smoke/windows-memory-decoder.py",
             "test/smoke/windows-owned-etw.cs", ".github/workflows/ci.yml"]:
    data = subprocess.check_output(["git", "show", SOURCE + ":" + name], cwd=CODE)
    assert b"\r\n" not in data
    source_files.append({"path": name, "gitLfSha256": sha(data),
                         "gitCheckoutCrLfSha256": sha(data.replace(b"\n", b"\r\n"))})
write(ROOT / "source-identity.json", {"source": SOURCE, "files": source_files,
                                     "branch": "feature/0.1.10-windows-working-set-pages"})
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
        {"name": "local:windows-controlled-memory-allocation-stack", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-ordinary-token-memory-recording-refusal", "status": "passed",
         "evidence": relative + "/original/lf-source-tests/result.json"},
        {"name": "local:windows-wpr-preparation-full-ci", "status": "passed",
         "evidence": relative + "/previous-ci/admission.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/candidate0110-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP.read_bytes()),
        "summary": "a5受控CI捕获8MiB分配/栈1对且无丢失，普通令牌拒绝有效；494完整CI通过，但产品RSS、bd2失败窗口及原门禁未解除。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "仅受控自进程取证方法通过；真实GPU入口与产品分配根因未验证，RSS/原生等完整门禁继续失败。",
    "revision": 1,
})
write(RECEIPT, receipt)
items = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        items.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": items})
verify()
print(json.dumps({"originalCopies": len(copies), "receiptResult": receipt["result"], **result}))
