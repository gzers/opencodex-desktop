"""Seal one real product GPU capture, its limits, and the retained normal exit."""
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
RUNTIME = CODE / "test/runtime/foreground-b4-gpu-20261017T084609"
SOURCE = "fde1dbb5b1731c056bbfa72a420bf930133671ea"
PRODUCT = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
EXE = "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-GPU-ALLOCATION-RUN-20261017.json"


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


def filetime(value):
    match = re.fullmatch(r"(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)(?:\.(\d{1,7}))?(Z|[+-]\d\d:\d\d)", value)
    assert match
    seconds = datetime.fromisoformat(match[1] + ("+00:00" if match[3] == "Z" else match[3])) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return str((seconds.days * 86400 + seconds.seconds) * 10000000 + int((match[2] or "").ljust(7, "0")))


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    original = ROOT / "original"
    human = read(original / "human-window.json")
    assert human["explicitHumanStart"] and not human["automaticContinuation"]
    assert human["userMessage"] == "开始前台验证" and human["maximumNativeInstances"] == 1
    assert human["scope"] == "one-b4-gpu-allocation-diagnostic" and not human["dpiAcceptanceRequired"]
    assert (instant(human["expiresAtUtc"]) - instant(human["authorizedAtUtc"])).total_seconds() == 1800
    assert "not claimed exact message timestamp" in human["timestampSource"]
    pre = read(original / "preflight.json")
    assert pre["passed"] and pre["nativeCount"] == 0 and pre["frozenMethodsVerified"]
    assert pre["toolSource"] == SOURCE and pre["productSource"] == PRODUCT and pre["exeSha256"] == EXE
    launch = read(original / "root-launch.json")
    gpu = read(original / "gpu-launch.json")
    recorder = read(original / "elevated-recorder-ready.json")
    assert launch["source"] == PRODUCT and launch["toolSource"] == SOURCE and launch["exeSha256"] == EXE
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert sha(Path(launch["executable"]).read_bytes()) == EXE
    product_token, recorder_token = launch["applicationToken"], recorder["actualRecorderToken"]
    assert not product_token["elevated"] and product_token["elevationType"] == 3
    assert recorder_token["elevated"] and recorder_token["elevationType"] == 2
    assert product_token["sidSha256"] == recorder_token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert recorder["humanWindowSha256"] == sha((original / "human-window.json").read_bytes())
    child = read(original / "gpu-prepare-child-result.json")
    assert child["exitObserved"] and child["exitCode"] == 0 and not child["tracingRequested"]
    assert not (original / "gpu-prepare.stderr.decoded.log").read_bytes()
    stdout = (original / "gpu-prepare.stdout.decoded.log").read_text(encoding="utf-8-sig")
    prepared = read(original / "gpu-prepare-only.json")
    stdout_value = json.loads(stdout)
    # ConvertFrom-Json/ConvertTo-Json rewrites this ISO value into local time.
    # Compare all seven fractional digits, then allow only this representation change.
    assert filetime(stdout_value["binding"]["atUtc"]) == filetime(prepared["binding"]["atUtc"])
    stdout_value["binding"]["atUtc"] = prepared["binding"]["atUtc"]
    assert stdout_value == prepared
    assert prepared["status"] == "bound-gpu-prepared-only" and prepared["binding"]["gpuRoleVerified"]
    assert not prepared["tracingStarted"] and not prepared["recordingExecutionAdmitted"]
    request = read(original / "record-request.json")
    assert request["recordApproved"] and request["seconds"] == 30 and not request["automaticRetry"]
    assert request["rootLaunchSha256"] == sha((original / "root-launch.json").read_bytes())
    assert request["gpuLaunchSha256"] == sha((original / "gpu-launch.json").read_bytes())
    result = read(original / "gpu-capture/result.json")
    binding = read(original / "gpu-capture/binding-validation.json")
    assert result["passed"] and result["boundGpuObservation"] and not result["controlledSelf"]
    assert result["rootPid"] == launch["pid"] and result["targetPid"] == gpu["pid"]
    assert result["targetCreationFileTime"] == filetime(gpu["creationTimeUtc"])
    assert result["rootCreationFileTime"] == filetime(launch["creationTimeUtc"])
    # Parse the seventh fractional digit directly; datetime truncates it.
    assert prepared["binding"]["targetCreationUtc"] == gpu["creationTimeUtc"]
    assert binding["before"]["targetCreationUtc"] == binding["after"]["targetCreationUtc"] == gpu["creationTimeUtc"]
    assert result["rootCreationFileTime"] == read(original / "exit-observer-ready.json")["before"]["creationFileTime"]
    assert result["targetPid"] == prepared["binding"]["targetPid"]
    assert result["probeBytes"] is None and result["selfPid"] is None
    assert result["allocationEvidenceObserved"] and result["observedAllocationEvents"] == 231
    assert result["matchingAllocationStacks"] == 231 and result["configuredBufferMiB"] == 32
    assert result["maximumSerializedBytes"] == 4194304 and result["records"] <= 4096
    assert result["omittedRecords"] == result["callbackErrors"] == 0
    assert result["nativeStop"]["FinalEventsLost"] == result["nativeStop"]["FinalRealTimeBuffersLost"] == 0
    assert result["nativeStop"]["StoppedOrAlreadyAbsent"] and result["ownedSessionAbsent"]
    assert result["readerJoined"] and not result["stopProcessingUsed"]
    assert result["retainedTargetGenerationUnchanged"] and result["retainedRootGenerationUnchanged"]
    assert result["captureError"] is None and result["consumerError"] is None and not result["cleanupErrors"]
    assert not any(result[k] for k in ("globalSampleIntervalExplicitlyWritten", "genericSessionStopInvoked",
                                      "systemWideEtlWritten", "unrelatedEventPayloadsSerialized",
                                      "rawAllocationAddressSaved", "targetMemoryContentsSaved",
                                      "originalTestLibraryExecuted", "officialQueryNodeExecuted", "nativeDesktopStarted"))
    assert binding["toolSource"] == SOURCE and binding["ciRunId"] == 38946198178
    assert binding["bindingStable"] and binding["bindingError"] is None and binding["captureQualityPassed"]
    assert binding["allocationEvidenceObserved"] and not binding["rssFailureCleared"]
    assert binding["captureResultSha256"] == sha((original / "gpu-capture/result.json").read_bytes())
    for side in (binding["before"], binding["after"]):
        assert side["rootPid"] == launch["pid"] and side["targetPid"] == gpu["pid"]
        assert side["gpuRoleVerified"] and not side["controlledSelf"] and side["retainedProcessCount"] == 3
        assert side["targetExeSha256"] == gpu["exeSha256"]
        assert side["ownerSidSha256"] == product_token["sidSha256"]
        assert not side["rootToken"]["elevated"] and side["rootToken"]["elevationType"] == 3
    recorder_end = read(original / "elevated-recorder-result.json")
    assert recorder_end["completed"] and recorder_end["exitCode"] == 0 and not recorder_end["automaticRetry"]
    analysis_spec = importlib.util.spec_from_file_location("gpu_analysis", ROOT / "analyze.py")
    analysis_module = importlib.util.module_from_spec(analysis_spec)
    analysis_spec.loader.exec_module(analysis_module)
    summary, pairs = analysis_module.analyze(original)
    assert summary == read(original / "allocation-analysis.json")
    assert len(pairs) == 231 and summary["phases"]["return-action-through-first-overview-observation"]["allocationEvents"] == 209
    assert summary["exclusiveModuleGroups"]["contains-recorded-driver-module-range"]["stackPairs"] == 57
    assert summary["inclusiveModulePresence"]["uiautomationcore.dll"]["stackPairs"] == 5
    module_map = read(original / "gpu-stack-module-map.original.json")
    assert len(module_map["modules"]) == 14
    for module in module_map["modules"]:
        assert re.fullmatch("[0-9a-f]{64}", module["fileSha256"])
        # All selected module files remain installed; independently bind disk bytes.
        path = Path(module["path"])
        assert path.stat().st_size == module["fileBytes"] and sha(path.read_bytes()) == module["fileSha256"]
    assert not module_map["targetMemoryContentsRead"] and not module_map["rssDifferenceAttributed"]
    close = read(original / "normal-close-action.json")
    exited = read(original / "root-exit.json")
    assert close["inputReturned"] and not close["forcedTermination"]
    assert exited["exitObserved"] and exited["exitCode"] == 0 and exited["nativeCount"] == 0
    assert exited["retainedHandle"] and not exited["forcedTermination"] and exited["kernelTimes"]["exited"]
    assert int(exited["kernelTimes"]["exitFileTime"]) > int(exited["kernelTimes"]["creationFileTime"])
    assert instant(human["authorizedAtUtc"]) < instant(launch["atUtc"]) < instant(result["startedAtUtc"])
    assert instant(result["finishedAtUtc"]) < instant(close["startedAtUtc"]) < instant(human["normalCloseByUtc"])
    assert instant(exited["atUtc"]) < instant(human["expiresAtUtc"])
    census = read(original / "native-census.original.json")
    assert len(census) == 251 and max(row["count"] for row in census) == 1
    assert all(row["count"] == 0 or row["matchesBoundRoot"] for row in census)
    cleanup = read(original / "generation-cleanup-readback.json")
    assert cleanup["allCapturedGenerationsGone"] and len(cleanup["capturedGenerations"]) == 7
    assert all(not row["sameGenerationStillRunning"] for row in cleanup["capturedGenerations"])
    release = read(original / "foreground-released.json")
    assert release["foregroundReleased"] and release["nativeCount"] == 0 and release["recorderExited"]
    assert release["originalProfileId"] == "0ebd4e59-001000000005cf4b"
    assert release["originalProfileCreationFileTime"] == "134366003622018367"
    assert release["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert release["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert not release["productStartedAgain"] and not release["forcedTermination"] and not release["rssFailureCleared"]
    for row in read(ROOT / "prior-bindings.json"):
        assert sha((PROJECT / row["path"]).read_bytes()) == row["sha256"]
    prior = PROJECT / ".adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FULL-CI"
    prior_proof = read(prior / "verification.json")
    assert prior_proof["source"] == SOURCE and prior_proof["runId"] == 38946198178
    assert prior_proof["wholeToolCiAdmitted"] and not prior_proof["actualProductGpuRecordingExecuted"]
    syntax = read(prior / "preparation/validation.json")["syntax"]
    for row in syntax:
        assert sha((original / row["path"]).read_bytes()) == row["sha256"]
    assert read(prior / "preparation/plan.json") == read(original / "plan.json")
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-one-actual-product-gpu-allocation-capture",
            "toolSource": SOURCE, "toolCiRunId": 38946198178, "productSource": PRODUCT,
            "productExeSha256": EXE, "ordinaryProductTokenProven": True,
            "administratorRecorderTokenProven": True, "productDualTokenAcceptanceClaimed": False,
            "actualGpuPreparePassed": True, "actualGpuRecordingPassed": True,
            "declaredObservationSeconds": 30, "records": 581, "serializedBytes": 269538,
            "allocationStackPairs": 231, "freeEvents": 119, "lostOrOmittedEvents": 0,
            "ownedSessionAbsent": True, "readerNaturallyDrained": True,
            "moduleSnapshotMatchedDistinctIps": 705, "unresolvedKernelRangeIps": 39,
            "symbolsResolved": False, "moduleSnapshotIsEventTimeLoaderProof": False,
            "normalProductExitCode": 0, "kernelExitTimeProven": True, "maximumNativeInstances": 1,
            "nativeCensusRows": 251, "capturedGenerationsGone": 7,
            "originalProfileAndPointerPreserved": True, "priorFailureRewritten": False,
            "allocationLengthIsRssOrNetMemory": False, "rssCauseEstablished": False,
            "originalRssFailureCleared": False, "originalBd2FailureCleared": False,
            "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists()
if (ROOT / "copies.json").exists():
    assert "--seal-existing" in sys.argv and not (ROOT / "manifest.json").exists()
    copies = read(ROOT / "copies.json")
else:
    copies = []
    for path in sorted(RUNTIME.rglob("*")):
        if path.is_file() and path.suffix in {".json", ".jsonl", ".log", ".ps1"}:
            data = path.read_bytes()
            assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), path.name
            relative = Path("original") / path.relative_to(RUNTIME)
            target = ROOT / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as out:
                out.write(data)
            copies.append({"path": relative.as_posix(), "runtimePath": str(path),
                           "bytes": len(data), "sha256": sha(data)})
    write(ROOT / "copies.json", copies)
    prior = [".adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FULL-CI/manifest.json",
             ".adg/evidence/OCX-0110-20261016-WIN-B4-CANDIDATE/manifest.json",
             ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/manifest.json",
             ".adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FAILURE/manifest.json"]
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
        {"name": "local:windows-actual-bound-gpu-allocation-capture", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-gpu-diagnostic-normal-exit", "status": "passed",
         "evidence": relative + "/original/root-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "真实b4 GPU单次30秒录制231分配栈、零丢失遗漏、同代际、正常exit0；录后模块映射收窄图形路径，不等同原RSS根因。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "取得真实产品GPU分配栈与完整收尾证据，原RSS、bd2和全部剩余门禁保持。",
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
