"""Read frozen native timing once; bind only unchanged scheduler behavior."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
PRIOR = PROJECT / ".adg/evidence/OCX-0110-20261015-WIN-REAL-STATE-FLOOR"
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-STATE-FLOOR-REUSE-20261017.json"
SOURCES = ["02a0d4a8cbae8ef1450908d86b2c75fcad65f04f",
           "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
           "fde1dbb5b1731c056bbfa72a420bf930133671ea"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def verify():
    manifest = read(PRIOR / "manifest.json")
    assert len(manifest["files"]) == 57
    for row in manifest["files"]:
        data = (PRIOR / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    old_equivalence = read(PRIOR / "schedule-source-equivalence.json")
    assert old_equivalence["actualTestExeSource"] == SOURCES[0]
    launch = read(PRIOR / "state-floor-retry-launch.json")
    assert launch["source"] == SOURCES[0]
    assert launch["exeSha256"] == "7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9"
    assert launch["sandboxUsed"] and launch["cdpUsed"] and launch["realMalformedState"]
    assert not launch["applicationUser"]["elevated"] and launch["applicationUser"]["elevationType"] == 3
    method = read(PRIOR / "logpoint-method.json")
    assert method["expectedDebuggerPauses"] == 0 and method["nowPerf"] < 20000
    assert not method["stubbedResult"] and not method["nativeBridgeReplaced"] and not method["clockOrTimerChanged"]
    observation = read(PRIOR / "logpoint-observation.json")
    result = read(PRIOR / "logpoint-result.json")
    assert observation["pauses"] == result["pauses"] == 0
    events = [e for e in observation["events"] if e["command"] == "update_schedule_plan"]
    assert len(events) == 2 and events[0]["status"] == "rejected" and events[1]["status"] == "fulfilled"
    assert 44000 <= events[0]["startPerf"] <= 90000
    gap = events[1]["startPerf"] - events[0]["endPerf"]
    assert gap == result["realMonotonicRetryGapMs"] == read(PRIOR / "verification.json")["realMonotonicRetryGapMs"]
    assert gap >= 1800000
    assert result["repaired"] and result["noEarlyPlannerCalls"] and result["realRetryFulfilled"]
    assert result["syntheticOnlineWakesUsed"] and not result["systemClockOrTimersChanged"]
    assert not result["queryEvents"] and not result["fullSchedulerGatePassed"]
    repair = read(PRIOR / "real-state-repair.json")
    assert 10000 <= repair["elapsedSinceRealFailureMs"] < 60000
    assert repair["nextDeadlinesAreFixtureOnly"] and not repair["frontendTimerChanged"]
    exited = read(PRIOR / "state-floor-retry-exit.json")
    assert exited["pid"] == launch["pid"] and exited["creationTimeUtc"] == launch["creationTimeUtc"]
    assert exited["exitObserved"] and exited["exitCode"] == 0 and not exited["forcedTermination"]
    finish = read(PRIOR / "finish-result.json")
    assert finish["nativeCount"] == 0 and finish["exitCode"] == 0 and finish["foregroundBeforeDuePerformed"]
    assert not read(PRIOR / "initial-failure.original.json")["repaired"]
    assert not read(PRIOR / "network-failure.original.json")["events"]
    files = []
    for old in old_equivalence["files"]:
        blobs = [subprocess.check_output(["git", "rev-parse", source + ":" + old["path"]],
                                         cwd=CODE).decode().strip() for source in SOURCES]
        assert len(set(blobs)) == 1 and blobs[0] == old["source02Blob"]
        data = subprocess.check_output(["git", "show", SOURCES[1] + ":" + old["path"]], cwd=CODE)
        files.append({"path": old["path"], "sameGitBlobForAllSources": blobs[0], "gitLfSha256": sha(data)})
    assert len(files) == 4
    binding = {"priorDirectory": PRIOR.relative_to(PROJECT).as_posix(),
               "priorManifestSha256": sha((PRIOR / "manifest.json").read_bytes()),
               "priorVerificationSha256": sha((PRIOR / "verification.json").read_bytes()),
               "actualTestExeSource": SOURCES[0], "actualTestExeSha256": launch["exeSha256"],
               "productCandidateSource": SOURCES[1], "diagnosticToolSource": SOURCES[2],
               "files": files}
    if (ROOT / "source-binding.json").exists():
        assert read(ROOT / "source-binding.json") == binding
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return binding, {
        "status": "verified-existing-real-state-error-floor-scope-reuse",
        "frozenPriorFilesVerified": 57, "unchangedSchedulerFiles": 4,
        "actualTestExeSource": SOURCES[0], "productCandidateSource": SOURCES[1],
        "realMonotonicRetryGapMs": gap, "minimumMs": 1800000, "debuggerPauses": 0,
        "oldActualErrorThenSuccessfulRetryVerified": True,
        "syntheticOnlineWakeDidNotAdvanceRetry": True,
        "laterCandidateExeActuallyExecutedByThisEvidence": False,
        "newRealTimerStarted": False, "hostProductStarted": False,
        "selectedQueryIpcCallsObserved": 0, "allNetworkRequestsOrShortLivedChildrenProvenZero": False,
        "real24hOr6hCycleAccepted": False, "sleepClockRollbackAndAllNativeWakeMatrixAccepted": False,
        "fullSchedulerGatePassed": False, "rssFailureCleared": False,
        "originalBd2FailureCleared": False, "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()[1]))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "source-binding.json").exists()
binding, result = verify()
write(ROOT / "source-binding.json", binding)
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
        {"name": "local:windows-real-state-error-floor-scope-reuse", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ], "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
                      "after_digest": sha(IMP.read_bytes()),
                      "summary": "57份既有真实30分钟下限原件核验，四调度blob在02/b4/fde相同；按已声明子集复用，不重计时或提升完整门禁。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "复用真实状态错误30分钟下限子集，保留实际旧EXE身份及所有其它调度/周期/性能缺口。",
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
print(json.dumps({"receiptResult": receipt["result"], **result}))
