"""Preserve the rejected component attempt; produce no resource comparison."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop/test/runtime")
ATTEMPT = RUNTIME / "gpu-alpha-resource-20261016T1636"
assert not (ROOT / "manifest.json").exists()


def write(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


copied = []
names = ["binding.json", "cleanup-readback.original.json", "failure.json",
         "reference-a/launch.original.json", "reference-a/observer-failure.json",
         "reference-a/scene-before.original.json"]
for name in names:
    data = (ATTEMPT / name).read_bytes()
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    assert not target.exists()
    target.write_bytes(data)
    copied.append({"path": target.relative_to(ROOT).as_posix(), "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
for name in ["gpu-alpha-resource-method-20261016T1636.mjs", "gpu-alpha-resource-observer-20261016T1636.ps1"]:
    data = (RUNTIME / name).read_bytes()
    target = ROOT / "methods" / name
    target.parent.mkdir(exist_ok=True)
    assert not target.exists()
    target.write_bytes(data)
    copied.append({"path": target.relative_to(ROOT).as_posix(), "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
binding = json.loads((ATTEMPT / "binding.json").read_text(encoding="utf-8-sig"))
failure = json.loads((ATTEMPT / "failure.json").read_text(encoding="utf-8-sig"))
cleanup = json.loads((ATTEMPT / "cleanup-readback.original.json").read_text(encoding="utf-8-sig"))
scene = json.loads((ATTEMPT / "reference-a/scene-before.original.json").read_text(encoding="utf-8-sig"))
observer = json.loads((ATTEMPT / "reference-a/observer-failure.json").read_text(encoding="utf-8-sig"))
assert binding["source"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert not binding["productionCodeChanged"] and binding["priorPixelFailureRetained"]
assert failure["completedStages"] == [] and not failure["automaticRetry"]
assert observer["message"] == "Owned process exited during the snapshot"
assert scene["frames"] > 0 and not scene["contextLost"] and scene["glError"] == 0
assert cleanup["nativeCount"] == 0 and not cleanup["sameRootGenerationStillRunning"]
assert cleanup["headlessRootExitCode"] is None and not cleanup["descendantCleanupFullyProven"]
assert not (ATTEMPT / "result.json").exists()
assert not (ATTEMPT / "premultiplied-b").exists() and not (ATTEMPT / "reference-a2").exists()
write(ROOT / "verification.json", {
    "status": "verified-rejected-method-attempt-only", "originalCopies": copied,
    "completedResourceStages": 0, "reason": observer["message"],
    "referenceSceneDrewBeforeObserver": True, "resourceSnapshotAccepted": False,
    "resourceComparisonProduced": False, "prototypeExecuted": False, "prototypeAdopted": False,
    "headlessRootLaterAbsent": True, "rootExitCodeKnown": False,
    "allDescendantCleanupProven": False, "nativeCountAtCleanup": 0,
    "productStarted": False, "pinnedOfficialNodeExecuted": False,
    "rssRootCauseEstablished": False, "formalPairRepeated": False,
    "historicalFailuresCleared": False, "releaseAccepted": False,
})
(ROOT / "README.md").write_bytes("""# 隔离浏览器资源对照未准入

本次只尝试在headless Edge组件装置对照823原光场与之前已否决的预乘透明原型，未修改生产源码或重复原生019/823配对。使用随宿主提供的Node24.19.0，不执行固定查询用官方Node24.21.0；原像素差异仍保留，不预先恢复该原型。

第一组原823参考光场已正常绘制，随后保留句柄的资源观察器发现自有子进程在快照期间退出，按原守卫拒绝，未产出资源快照。后两组没有启动，未执行原型、没有对照结果或内存收益结论，不重跑或放宽守卫。该方法失败不能算产品缺陷，也不能证明29.22MiB增量来源。

finally请求关闭该独立browser-server，后续读回其根代际已不存在、native0；退出观察器在失败前未就绪，实际exitCode和内核退出时刻未知，后代清理未完整证明。不写成正常exit0或完整回收通过。六份原件和两个实际方法逐字节保留，未复制生产源码或原型成为第二事实源。

生产仍823且原RSS失败保留。后续资源定位需在实际原生配对所用WebView代际取得可绑定的角色和分配证据；本装置不替代该范围。DPI专项按用户裁决取消，其余原门禁与发布限制保持。
""".encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
document = PROJECT / impact_path
old = document.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 隔离组件资源尝试未准入，未采用原型"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

一次headless组件资源对照在首组823参考场景绘制后，因快照期间自有子进程退出被原保留句柄守卫拒绝；后两组未执行，未运行已否决的预乘原型、未生成资源比较、不重跑或放宽守卫。方法与六份原件见`.adg/evidence/OCX-0110-20261016-WIN-GPU-COMPONENT-ATTEMPT/`。独立浏览器finally关闭请求后根代际已不存在/native0；exitCode、内核退出时刻未知，后代清理未完整证明，不补写正常退出。

本次未启动产品或固定官方查询Node，生产823无改动，不能据此解释或解除+29.221361MiB RSS失败。bd2单次549已录制但目标失败未复现，原Mac/Windows失败与其余门禁保持；DPI专项已取消，不关闭或发布。
"""
document.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import build_receipt, validate_object
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-GPU-COMPONENT-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
from datetime import datetime, timezone
receipt = build_receipt(task, receipt_id=rid, result="failed", changed_paths=[impact_path, rpath],
    checks=[{"name": "local:windows-component-resource-comparison", "status": "failed",
        "evidence": ".adg/evidence/OCX-0110-20261016-WIN-GPU-COMPONENT-ATTEMPT/verification.json"}],
    acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(document.read_bytes()).hexdigest(),
        "summary": "保留组件资源观察未准入，不生成比较、采用原型或解除原生RSS失败。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="组件资源快照被自有子进程退出守卫拒绝，0组完整结果、无重跑、无生产修改；原门禁保持。")
validate_object(receipt)
write(PROJECT / rpath, receipt)
write(ROOT / "manifest.json", {"files": [
    {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
     "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
    for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"
], "frozen": True, "resourceComparisonProduced": False})
print(json.dumps({"archivedRuntimeFiles": len(copied), "resourceStages": 0, "receipt": rpath}))
