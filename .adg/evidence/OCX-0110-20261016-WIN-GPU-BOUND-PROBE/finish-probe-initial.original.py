"""Seal the completed controlled probe without changing historical originals."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import build_receipt, validate_object


def write_json(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


assert not (ROOT / "manifest.json").exists(), "Frozen evidence cannot be extended"
verified = json.loads(subprocess.check_output(
    [sys.executable, str(ROOT / "verify-probe.py")], cwd=PROJECT))
write_json(ROOT / "verification.json", verified)
write_json(ROOT / "ci-current-readback.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "source": verified["source"], "runId": verified["ciRunId"],
    "firstPushWholeCiStatus": verified["ciStatus"],
    "firstPushWholeCiConclusion": verified["ciConclusion"],
    "rawRunAndJobsApiVerified": True,
    "rawJobLogsFullyVerified": False,
    "initialProvenancePreserved": True,
    "nativeSnapshotAtProbeTime": False,
    "foregroundFollowupArchive": ".adg/evidence/OCX-0110-20261016-WIN-823-RECOVERY-GPU/",
})
(ROOT / "README.md").write_bytes("""# GPU绑定读取器的受控执行检查

工具分支`feature/0.1.10-windows-gpu-resources@768e2318`已推送读回，ETW待录制分支仍固定54927d2c。本机37项检查通过，包括真实PowerShell根/子代绑定、保留句柄、角色与计数器筛选、正常退出和代际消失读回；未启动桌面产品。受控进程未取得GPU实例，缺失实例不解释为零显存，亦不证明原生产品入口已执行。

首次push CI38864269467于15:17:42读回完整success，五必需作业成功、main-only build正常skip；原run/jobs API与摘要逐项核对。三作业原始日志尚未完整下载核验，不能声称原日志准入。`code-provenance.json`保留收集时的`sourceCiPassed:false`，终态由新增读回说明，不重写历史。

初版离线校验把同一UTC瞬间的七位和六位小数作为不同字符串，失败及原校验器保留；修订按100ns整数核代际，原快照/退出/清理字节不改。`verify-probe.py`核对原件、Git方法字节、实际绑定、全CI状态和清单。后续人工窗口的真实产品快照及恢复尝试另存`OCX-0110-20261016-WIN-823-RECOVERY-GPU`，不倒改本准备结论。

Windows平均RSS+29.221361MiB、历史Windows/Mac资源失败、bd2真实失败窗口及全部原门禁保持；本方法检查不代替600秒性能、单个GL缓冲归因或发布验收。TASK revision2/in_progress/pending不变，不合并、关闭或发布。
""".encode("utf-8"))

impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
impact_file = PROJECT / impact_path
old = impact_file.read_bytes().replace(b"\r\n", b"\n")
addition = """

### 2026-10-16 GPU读取器完成受控执行与首次CI

独立工具分支768e2318已推送读回，保持ETW待录制分支549不变。本机37项检查含真实PowerShell根/子代保留句柄与正常退出，控制代际均已消失、native0；GPU实例为空不视为零显存，不当作桌面产品快照。首次push38864269467于15:17:42全CI成功，五必需作业通过、main-only build正常skip，原run/jobs API核对；完整job原日志仍未取得。初版UTC字符串尾零比较失败与修订校验器保留，原件未改。

证据`.adg/evidence/OCX-0110-20261016-WIN-GPU-BOUND-PROBE/`独立校验通过，只证明受控方法及CI状态。后续真实人工窗口的823恢复/GPU执行另档，不回改准备时事实。Windows RSS+29.221361MiB、原Mac/Windows失败、bd2实际失败窗口缺失及全部原预算/周期/八门禁保持；TASK revision2/in_progress/pending、DPI待验与禁止发布状态不变。
"""
assert "### 2026-10-16 GPU读取器完成受控执行与首次CI" not in old.decode("utf-8")
impact_file.write_bytes(old.rstrip() + addition.encode("utf-8") + b"\n")
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
receipt_id = "RCP-OPENCODEX-DESKTOP-182-WIN-GPU-PROBE-20261016"
receipt_path = f".adg/work/objects/rcp/{receipt_id}.json"
receipt = build_receipt(task, receipt_id=receipt_id, result="passed",
    changed_paths=[impact_path, receipt_path],
    checks=[
        {"name": "local:win-gpu-controlled-binding", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-GPU-BOUND-PROBE/verification.json"},
        {"name": "local:win-gpu-first-push-ci-status", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-GPU-BOUND-PROBE/ci-current-readback.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(impact_file.read_bytes()).hexdigest(),
        "summary": "记录GPU读取器受控执行37项及首次全CI状态；完整原日志、原生执行和原全部门禁不提升。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="受控GPU绑定读取方法与首次CI状态通过；不代表产品性能或TASK关闭，原失败保持。")
validate_object(receipt)
write_json(PROJECT / receipt_path, receipt)
files = [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
          "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
         for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"]
write_json(ROOT / "manifest.json", {"files": files, "frozen": True,
    "rawLogsFullyVerified": False, "productAcceptanceClaimed": False})
subprocess.run([sys.executable, str(ROOT / "verify-probe.py")], cwd=PROJECT, check=True)
print(json.dumps({"sealedFiles": len(files), "receipt": receipt_path, "result": "controlled-method-only"}))
