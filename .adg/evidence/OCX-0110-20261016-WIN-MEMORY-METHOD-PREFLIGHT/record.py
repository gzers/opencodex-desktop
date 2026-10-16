"""Archive the stopped controlled method and PR follow-up; do not rerun either."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop/test/runtime/memory-trace-method-20261016T2252")
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-MEMORY-METHOD-PREFLIGHT-20261016.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def verify():
    copies = read(ROOT / "copies.json")
    for row in copies:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    failure = read(ROOT / "original/controlled/failure.json")
    assert failure["name"] == "AssertionError" and "categories.categories.includes" in failure["message"]
    assert not failure["traceStarted"] and not failure["traceCompleted"] and not failure["automaticRetry"]
    binding = read(ROOT / "original/controlled/binding.json")
    assert not binding["desktopProductStarted"] and not binding["nativeIdlePairRepeated"]
    assert not binding["priorFailedComponentSnapshotRetried"] and not binding["priorRejectedPixelPrototypesUsed"]
    assert binding["methodSha256"] == sha((ROOT / "original/controlled-memory-infra.mjs").read_bytes())
    assert binding["observerSha256"] == sha((ROOT / "original/root-observer.ps1").read_bytes())
    assert binding["nodeVersion"] == "v24.19.0"
    ready = read(ROOT / "original/controlled/root-ready.original.json")
    stopped = read(ROOT / "original/controlled/root-exit.original.json")
    node_exit = read(ROOT / "original/controlled/node-root-exit.original.json")
    observer = read(ROOT / "original/controlled/observer-exit.original.json")
    assert ready["pid"] == stopped["pid"] == node_exit["pid"] == 64176
    assert ready["observerBoundBeforeTrace"] and ready["kernel"]["retainedHandle"]
    assert stopped["kernel"]["retainedHandle"] and stopped["kernel"]["exited"]
    assert ready["kernel"]["creationFileTime"] == stopped["kernel"]["creationFileTime"]
    assert int(stopped["kernel"]["exitFileTime"]) > int(ready["kernel"]["creationFileTime"])
    assert stopped["exitCode"] == node_exit["code"] == observer["code"] == 0
    assert node_exit["signal"] is None and stopped["nativeCount"] == 0
    assert not stopped["forcedTermination"]
    actual = read(ROOT / "original/final-readback.original.json")
    assert actual["nativeCount"] == 0 and not actual["controlledRootSameGenerationStillRunning"]
    assert not actual["controlledDescendantsFullyEnumerated"]
    assert not actual["productStarted"] and not actual["uiInputInjected"] and not actual["wprRecordRequested"]
    assert actual["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert actual["restoredPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert actual["taskFileSha256"] == sha(TASK.read_bytes()) == (
        "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93")
    before = read(ROOT / "original/pr-before.original.json")
    final = read(ROOT / "original/pr-final.original.json")
    request = read(ROOT / "original/pr-update-request.json")
    assert final["number"] == before["number"] == 6
    assert final["head"]["sha"] == before["head"]["sha"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
    assert final["state"] == "open" and final["draft"] and not final["merged"]
    assert final["body"] == request["body"] and final["body"] != before["body"]
    assert "+20.040155MiB" in final["body"] and "failed" in final["body"]
    assert "22:42:01" in final["body"] and "14个进程代际全部结束" in final["body"]
    assert "候选实际退出码和确切内核退出时刻仍未知" in final["body"]
    profile = (ROOT / "original/wpr-memory-profiledetails.decoded.log").read_text(encoding="utf-8")
    assert "Number of Buffers\t: 1259" in profile and "Number of Buffers\t: 513" in profile
    assert profile.count("Buffer Size (KB)\t: 1024") == 2
    assert "WPR is not recording" in (ROOT / "original/wpr-status.original.log").read_text(encoding="utf-8")
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {
        "status": "verified-stopped-before-trace", "controlledMethodAdmitted": False,
        "memoryDumpTaken": False, "browserRootNormalExitCode": 0,
        "controlledDescendantsFullyEnumerated": False, "nativeCountAtReadback": 0,
        "prNumber": 6, "prDraftOpen": True, "restorationFollowupInPr": True,
        "defaultCombinedWprConfiguredBuffersMiB": 1259 + 513,
        "configuredBuffersAreMeasuredProcessMemory": False, "wprRecordingStarted": False,
        "rssFailureCleared": False, "allocationCauseAttributed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []
for path in sorted(RUNTIME.rglob("*")):
    if not path.is_file() or path.name == "browser_protocol.original.json":
        continue
    data = path.read_bytes()
    assert path.suffix in {".json", ".py", ".mjs", ".ps1", ".log"}
    text = data.decode("utf-8-sig")
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", text)
    assert not re.search(r"\bgh[pousr]_[A-Za-z0-9]{20,}", text)
    target = ROOT / "original" / path.relative_to(RUNTIME)
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": target.relative_to(ROOT).as_posix(), "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
protocol = RUNTIME / "browser_protocol.original.json"
write(ROOT / "primary-source.json", {
    "url": "https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/master/json/browser_protocol.json",
    "runtimePath": str(protocol), "sha256": sha(protocol.read_bytes()), "bytes": protocol.stat().st_size,
    "entireExternalProtocolPublished": False, "actualBrowserCategoryArrayPersisted": False,
    "actualBrowserVersionResponsePersisted": False,
})
result = verify()
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object
task = read(TASK)
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
relative = ROOT.relative_to(PROJECT).as_posix()
prior = ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES"
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-component-memory-trace-admission", "status": "failed",
         "evidence": relative + "/original/controlled/failure.json"},
        {"name": "local:windows-controlled-trace-root-exit", "status": "passed",
         "evidence": relative + "/original/controlled/root-exit.original.json"},
        {"name": "local:windows-pr-restoration-followup", "status": "passed",
         "evidence": relative + "/original/pr-final.original.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": prior + "/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP.read_bytes()),
        "summary": "独立组件转储方法在录制前被类别前置拒绝、无自动重试；根代际exit0，草稿PR已回写配置恢复，RSS失败保持。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "组件转储前置未通过，未取得分配证据；只读核到WPR配置但未录制，PR6同步配置恢复，全部原失败/门禁保留。",
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
