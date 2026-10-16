"""Preserve one actual failed preparation, normal exit, and test-tool repair."""
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
RUN = CODE / "test/runtime/foreground-b4-gpu-20261017T020829"
FIX = CODE / "test/runtime/gpu-helper-scope-fix-20261017T0216"
OLD = "391e70979ffbb32a548c0d05bdeeb62160d942d3"
SOURCE = "fde1dbb5b1731c056bbfa72a420bf930133671ea"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-GPU-SCOPE-FAILURE-20261017.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def instant(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    front = ROOT / "foreground"
    human = read(front / "human-window.json")
    launch = read(front / "root-launch.json")
    recorder = read(front / "elevated-recorder-ready.json")
    failure = read(front / "prepare-failure-observation.json")
    roles = read(front / "settings-roles.original.json")
    gpu = read(front / "gpu-launch.json")
    request = read(front / "record-request.json")
    recorder_end = read(front / "elevated-recorder-result.json")
    close = read(front / "normal-close-action.json")
    exit_result = read(front / "root-exit.json")
    released = read(front / "foreground-released.json")
    cleanup = read(front / "generation-cleanup-readback.json")
    assert human["explicitHumanStart"] and not human["automaticContinuation"]
    assert human["userMessage"] == "开始前台验证"
    assert human["scope"] == "one-b4-gpu-allocation-diagnostic"
    assert (instant(human["expiresAtUtc"]) - instant(human["authorizedAtUtc"])).total_seconds() == 1800
    assert launch["source"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19" and launch["toolSource"] == OLD
    assert launch["exeSha256"] == "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"
    product_token, recorder_token = launch["applicationToken"], recorder["actualRecorderToken"]
    assert not product_token["elevated"] and product_token["elevationType"] == 3
    assert recorder_token["elevated"] and recorder_token["elevationType"] == 2
    assert product_token["sidSha256"] == recorder_token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert recorder["humanWindowSha256"] == sha((front / "human-window.json").read_bytes())
    assert roles["rootPid"] == launch["pid"] and roles["rootCreationUtc"] == launch["creationTimeUtc"]
    gpu_role, = [r for r in roles["members"] if r["role"] == "gpu"]
    assert gpu_role["pid"] == gpu["pid"] and gpu_role["creationUtc"] == gpu["creationTimeUtc"]
    assert roles["desktopRootVerified"] and not roles["formalIdleSpanMeasured"]
    assert failure["recordKind"] == "derived-observation-of-tool-output"
    assert not failure["rawStderrFileSaved"] and not failure["initialOpenBindingWitnessPersisted"]
    assert not failure["prepareOutputFileCreated"] and not failure["etwRecordingStarted"]
    assert failure["toolSource"] == OLD and not failure["recordApprovedPassedToEntry"]
    assert request["cancelled"] and request["recordApproved"] is False and request["seconds"] == 0
    assert not recorder_end["completed"] and recorder_end["exitCode"] == 1
    assert recorder_end["message"] == "One approved 30-second recording required"
    assert not (RUN / "gpu-capture").exists() and not (RUN / "gpu-prepare-only.original.json").exists()
    assert not close["forcedTermination"] and close["inputReturned"]
    assert exit_result["exitObserved"] and exit_result["exitCode"] == 0 and exit_result["retainedHandle"]
    assert not exit_result["forcedTermination"] and exit_result["nativeCount"] == 0
    kernel = exit_result["kernelTimes"]
    assert kernel["exited"] and kernel["retainedHandle"]
    assert int(kernel["exitFileTime"]) > int(kernel["creationFileTime"])
    assert instant(human["authorizedAtUtc"]) < instant(launch["atUtc"]) < instant(close["startedAtUtc"])
    assert instant(close["returnedAtUtc"]) < instant(human["normalCloseByUtc"])
    assert instant(exit_result["atUtc"]) < instant(human["expiresAtUtc"])
    census = read(front / "native-census.original.json")
    assert len(census) == 148 and max(r["count"] for r in census) == 1
    assert all(r["count"] == 0 or r["matchesBoundRoot"] for r in census)
    assert cleanup["allCapturedGenerationsGone"] and len(cleanup["capturedGenerations"]) == 7
    assert all(not row["sameGenerationStillRunning"] for row in cleanup["capturedGenerations"])
    assert released["foregroundReleased"] and released["nativeCount"] == 0 and released["recorderExited"]
    assert not released["gpuCaptureDirectoryExists"] and not released["tracingStarted"]
    assert released["originalProfile"]["fileId"] == "0ebd4e59-001000000005cf4b"
    assert released["originalProfile"]["creationFileTime"] == "134366003622018367"
    assert released["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert released["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    overview = read(front / "overview-before.original.json")
    settings = read(front / "settings-settled.original.json")
    assert "#overview" in overview["accessibility"]["tree"] and "#settings" in settings["accessibility"]["tree"]
    assert overview["screenshotDisplayed"] and not overview["screenshotPayloadSaved"]
    fix = ROOT / "fix"
    analysis = read(fix / "source-analysis.json")
    assert analysis["failedToolSource"] == OLD and analysis["fixedToolSource"] == SOURCE
    assert analysis["productApplicationTreeBefore"] == analysis["productApplicationTreeAfter"]
    assert not analysis["productApplicationChanged"] and not analysis["actualProductGpuReplayAfterFix"]
    assert analysis["originalNewRegressionFailedBeforeFix"] and analysis["fixedLocalChecks"] == 22
    assert "Token reader disappeared after opening the lease" in (fix / "before.original.log").read_text(encoding="utf-8-sig")
    assert read(fix / "before-result.json")["exitCode"] != 0
    for row in analysis["changedFiles"]:
        before = subprocess.check_output(["git", "show", OLD + ":" + row["path"]], cwd=CODE)
        after = subprocess.check_output(["git", "show", SOURCE + ":" + row["path"]], cwd=CODE)
        assert sha(before) == row["beforeSha256"] and sha(after) == row["afterSha256"]
    local = read(fix / "after/result.json")
    assert local["passed"] and local["checks"] == 22 and local["helpersAvailableAfterOpenReturns"]
    assert local["actualHarnessTokenRereadAfterOpen"] and local["roleParserCheckedAfterOpen"]
    controlled = read(fix / "controlled-after/result.json")
    assert controlled["passed"] and controlled["checks"] == 10 and controlled["compiledGuardChecks"] == 20
    assert controlled["ordinaryTokenRefusalTested"] and not controlled["administratorToken"]
    method = read(fix / "method-admission.json")
    handle = read(fix / "ci-handle.json")
    checkpoint = read(fix / "method-checkpoint.original.json")
    assert checkpoint["run"]["id"] == 38946198178 and checkpoint["run"]["head_sha"] == SOURCE
    assert checkpoint["run"]["status"] == "in_progress" and checkpoint["run"]["conclusion"] is None
    assert checkpoint["run"]["run_attempt"] == 1
    checkpoint_job, = [j for j in checkpoint["jobs"]["jobs"] if j["id"] == 117125807346]
    assert checkpoint_job["conclusion"] == "success" and checkpoint_job["head_sha"] == SOURCE
    assert method["source"] == handle["source"] == SOURCE
    assert method["runId"] == handle["runId"] == 38946198178
    assert method["jobId"] == 117125807346 and method["actualCheckout"] == SOURCE
    assert method["bindingChecks"] == 22 and not method["wholeCiAdmitted"]
    assert method["artifactId"] == 12064933397 and method["artifactBytes"] == 77413
    assert method["artifactSha256"] == "c2acc976a058fc286aad36d98ec81921882cfd8fa080af05ff2a210044f70ede"
    private = read(ROOT / "private-method-artifact.json")
    archive_path = Path(private["runtimePath"])
    assert sha(archive_path.read_bytes()) == method["artifactSha256"]
    with zipfile.ZipFile(archive_path) as zipped:
        assert zipped.testzip() is None
        for row in method["extracted"]:
            data = (fix / row["path"]).read_bytes()
            assert data == zipped.read(row["member"]) and sha(data) == row["sha256"]
    log = (fix / "ci-method-job.original.log").read_bytes()
    assert sha(log) == method["jobLogSha256"]
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    ci_binding = read(fix / "ci-method/memory-target/result.json")
    assert ci_binding["passed"] and ci_binding["checks"] == 22
    capture = read(fix / "ci-method/controlled-memory/capture/result.json")
    assert capture["passed"] and capture["controlledSelf"] and not capture["boundGpuObservation"]
    assert capture["matchingProbeAllocationEvents"] == capture["matchingAllocationStacks"] == 1
    assert capture["probeFreed"] and capture["ownedSessionAbsent"] and capture["readerJoined"]
    events = (fix / "ci-method/controlled-memory/capture/owned-memory.events.jsonl").read_bytes()
    assert len(events) == capture["serializedBytes"] == 765616 and len(events.splitlines()) == capture["records"] == 1257
    assert capture["omittedRecords"] == capture["callbackErrors"] == 0
    assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
    for row in read(ROOT / "prior-bindings.json"):
        assert sha((PROJECT / row["path"]).read_bytes()) == row["sha256"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-first-gpu-prepare-failure-and-tool-scope-repair",
            "failedToolSource": OLD, "fixedToolSource": SOURCE, "ciRunId": 38946198178,
            "productOrdinaryTokenProven": True, "recorderAdministratorTokenProven": True,
            "productDualTokenAcceptanceClaimed": False, "actualGpuPreparationFailed": True,
            "actualProductGpuRecordingExecuted": False, "nativeCensusRows": 148, "maximumNativeInstances": 1,
            "normalExitCode": 0, "kernelExitTimeProven": True, "capturedGenerationsGone": 7,
            "originalProfileAndPointerPreserved": True, "newRegressionFailedBeforeRepair": True,
            "localAndCiBindingChecksEach": 22, "methodCiAdmitted": True,
            "wholeCiAdmittedAtFrozenCheckpoint": False, "productApplicationChanged": False,
            "rssFailureCleared": False, "originalBd2FailureCleared": False,
            "historicalExitFailuresRewritten": False, "fullProductGatePassed": False, "releaseAccepted": False}


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


for path in sorted(RUN.iterdir()):
    if path.is_file() and path.suffix in {".json", ".ps1"}:
        copy(path, "foreground/" + path.name)
for name in ("before.original.log", "before-result.json", "remote-readback.json", "source-analysis.json",
             "ci-handle.json", "method-job-and-artifacts.original.json", "method-admission.json",
             "ci-method-job.original.log", "follow-ci.py"):
    copy(FIX / name, "fix/" + name)
for folder in ("after", "controlled-after", "ci-method"):
    for path in sorted((FIX / folder).rglob("*")):
        if path.is_file() and path.suffix in {".json", ".jsonl", ".log", ".txt"}:
            copy(path, "fix/" + path.relative_to(FIX).as_posix())
snapshots = sorted(p for p in FIX.glob("ci-*.original.json")
                   if re.fullmatch(r"ci-\d{8}T\d+Z.original.json", p.name))
copy(snapshots[-1], "fix/method-checkpoint.original.json")
write(ROOT / "copies.json", copies)
archive_path = FIX / "method.original.zip"
write(ROOT / "private-method-artifact.json", {"runtimePath": str(archive_path),
                                            "sha256": sha(archive_path.read_bytes()),
                                            "bytes": archive_path.stat().st_size, "rawZipPublished": False})
prior = [".adg/evidence/OCX-0110-20261017-WIN-BOUND-GPU-PREP/manifest.json",
         ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/manifest.json",
         ".adg/evidence/OCX-0110-20261016-WIN-B4-RESTORATION/manifest.json"]
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
        {"name": "local:windows-actual-gpu-prepare", "status": "failed",
         "evidence": relative + "/foreground/prepare-failure-observation.json"},
        {"name": "local:windows-gpu-tool-scope-repair-method", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-gpu-diagnostic-normal-exit", "status": "passed",
         "evidence": relative + "/foreground/root-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "真实GPU准备作用域失败，未录制；唯一普通b4正常exit0及原配置保持。fde修复前失败/后22通过，首次CI方法通过、整轮待核。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "保存实际准备失败和正常退出，修复测试工具作用域；原产品RSS及全部门禁未解除。",
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
