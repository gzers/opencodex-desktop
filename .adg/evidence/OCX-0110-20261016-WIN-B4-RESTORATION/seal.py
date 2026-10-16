"""Verify and record profile restoration; preserve the earlier failed lifecycle."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
PRIOR = ROOT.parent / "OCX-0110-20261016-WIN-B4-IDLE-ROLES"
PREP = ROOT.parent / "OCX-0110-20261016-WIN-B4-FRONT-PREP"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-B4-RESTORATION-20261016.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def ticks(value):
    digits = re.search(r"\.(\d+)(?:Z|[+-]\d\d:\d\d)$", value).group(1)
    instant = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    delta = instant.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * 10_000_000 + int(digits.ljust(7, "0"))


def verify():
    state = read(ROOT / "restoration-readback.original.json")
    ready = read(PRIOR / "original/fixture-ready.json")
    prep = read(PREP / "preparation-readback.json")
    marker_bytes = (ROOT / "fixture-restored.original.json").read_bytes()
    marker = json.loads(marker_bytes.decode("utf-8-sig"))
    assert sha(marker_bytes) == state["markerSha256"]
    assert marker_bytes == Path(state["markerRuntimePath"]).read_bytes()
    assert state["passed"] and state["nativeCountBefore"] == state["nativeCountAfter"] == 0
    assert not state["baselinePreviousExists"]
    assert state["humanReportedExit"] and state["humanMessage"] == "已退出"
    assert state["candidateExitCode"] is None and not state["exactExitTimeProven"]
    assert state["originalExitTimeoutsPreserved"] and not state["formalPairLifecycleAdmitted"]
    for key in ["fileId", "creationFileTime", "finalPath"]:
        assert state["originalDirectory"][key] == ready["originalDirectory"][key]
    for key in ["fileId", "creationFileTime"]:
        assert state["measuredCloneDirectory"][key] == ready["cloneDirectory"][key]
    assert marker["measuredClonePreservedAt"] == state["measuredCloneDirectory"]["path"]
    assert marker["originalDirectoryRestored"]
    for key in ["dataDeleted", "productStarted", "uiInputInjected", "registryOrAclChanged"]:
        assert not marker[key]
    for key in ["productStarted", "uiInputInjected", "forceTerminated", "dataDeleted", "releaseAccepted"]:
        assert not state[key]
    original, = [r for r in prep["fixtureReadback"] if r["path"] == ready["baselineRoot"]]
    assert state["restoredPreferencesSha256"] == original["preferencesSha256"]
    assert state["startupPointerSha256"] == prep["startupPointerSha256"]
    assert state["taskFileSha256"] == sha(TASK.read_bytes()) == (
        "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93")
    expected = set()
    for version in ["baseline019", "candidate0110"]:
        roles = read(PRIOR / f"original/{version}-settled-roles.original.json")
        expected |= {(version, r["pid"], ticks(r["creationUtc"]), r["role"]) for r in roles["members"]}
    observed = {(r["version"], r["capturedPid"], ticks(r["capturedCreationUtc"]), r["role"])
                for r in state["ownedGenerations"]}
    assert expected == observed and len(observed) == len(state["ownedGenerations"]) == 14
    assert all(not r["sameGenerationStillRunning"] and not r["unrelatedProcessDetailsSaved"]
               for r in state["ownedGenerations"])
    prior_scope = read(PRIOR / "scope.json")
    assert not prior_scope["fixtureRestoredAtArchive"] and not prior_scope["formalPairLifecycleAdmitted"]
    assert not prior_scope["fullPerformanceGatePassed"] and prior_scope["meanRssIncreaseMiB"] > 10
    if (ROOT / "bindings.json").exists():
        for row in read(ROOT / "bindings.json")["files"]:
            assert sha((PROJECT / row["path"]).read_bytes()) == row["sha256"]
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    for path in [ROOT / "fixture-restored.original.json", ROOT / "restoration-readback.original.json"]:
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", path.read_text(encoding="utf-8-sig"))
    return {
        "status": "verified-profile-restoration-followup",
        "restoredAtUtc": marker["atUtc"], "nativeCountAtReadback": 0,
        "capturedGenerationsConfirmedGone": 14, "originalDirectoryIdentityPreserved": True,
        "originalPreferencesDigestPreserved": True, "startupPointerDigestPreserved": True,
        "measurementClonePreserved": True, "candidateExitCode": None,
        "originalExitTimeoutsPreserved": True, "formalPairLifecycleAdmitted": False,
        "rssFailureCleared": False, "fullProductGatePassed": False, "releaseAccepted": False,
    }


result = verify()
if "--verify" in sys.argv:
    print(json.dumps(result))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "manifest.json").exists()
bindings = []
for path in [PREP / "manifest.json", PRIOR / "manifest.json", PRIOR / "scope.json",
             PREP / "restore-fixture.ps1"]:
    bindings.append({"path": path.relative_to(PROJECT).as_posix(), "sha256": sha(path.read_bytes())})
write(ROOT / "bindings.json", {"files": bindings})
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object

task = read(TASK)
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
assert len(task["required_checks"]) == 8
relative = ROOT.relative_to(PROJECT).as_posix()
prior_relative = PRIOR.relative_to(PROJECT).as_posix()
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-test-profile-restoration", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-captured-generations-ended", "status": "passed",
         "evidence": relative + "/restoration-readback.original.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": prior_relative + "/original/candidate0110-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": prior_relative + "/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP.read_bytes()),
        "summary": "用户退出后确认native0并恢复原目录，原身份/偏好/指针与14代际结束核验通过；保留测量克隆，原退出超时及RSS失败不解除。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "配置恢复子集通过；原目录及测量克隆均保留，原性能/生命周期和其它门禁未通过，TASK不关闭。",
    "revision": 1,
})
write(RECEIPT, receipt)
files = []
for path in sorted(ROOT.iterdir()):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        files.append({"path": path.name, "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": files})
verify()
print(json.dumps({"receipt": receipt["id"], "receiptResult": receipt["result"], **result}))
