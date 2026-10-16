"""Write back a bounded descriptive finding while preserving the RSS failure."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
assert not (ROOT / "manifest.json").exists()
subprocess.run([sys.executable, str(ROOT / "analyze.py")], cwd=PROJECT, check=True,
               stdout=subprocess.DEVNULL)
analysis = json.loads((ROOT / "analysis.json").read_text(encoding="utf-8"))
assert analysis["largestBySizeDifferenceFractionOfTotal"] > 0.89
assert not analysis["originalBudgetPassed"] and not analysis["sizeAccountingIsCausalAttribution"]
readme = """# 既有823配对RSS样本的时间线分解

仅离线读取已准入的019/823原始557/559样本，摘要与冻结comparison一致，无产品启动、采样、前台操作或第二轮配对。原样本平均RSS仍548.664841→577.886202MiB，**+29.221361MiB，超过+10MiB**，原口径和失败不变。

| 描述量 | 正式019 | 候选823 |
| --- | ---: | ---: |
| 原进程族均值 MiB | 548.664841 | 577.886202 |
| 各自最大WebView成员均值 MiB | 205.181329 | 231.411428 |
| 其它成员合计均值 MiB | 343.483512 | 346.474774 |
| 第一相对分钟进程族均值 MiB | 540.154537 | 566.731818 |
| 第十相对分钟进程族均值 MiB | 557.714205 | 583.250000 |
| 全跨度线性斜率 MiB/分钟 | 1.661734 | 2.057205 |

各版最大成员在自身全部样本中都是同一PID/创建代际；它们的均值差26.230100MiB，占进程族差值89.7634%，其余成员差2.991261MiB。两版之间仅比较“按大小最大的成员”这一描述量，未匹配角色、未将旧PID带入新进程，更不能直接叫GPU或renderer。这是算术分解，不是因果归因或可回收量。

第一相对分钟已差26.577281MiB，第十相对分钟差25.535795MiB；两版自身都增长，斜率差约0.395471MiB/分钟。本600秒观察既不能证明增长是固定占用，也不能确认/排除泄漏，足以把下一步重点收窄到真实WebView角色与分配资源，而非重复相同完整配对或先改Rust业务状态。

分钟窗是从各自第一计数样本起的[0,60)…[540,600)；两版各有1个相对600秒后的末样本不进入十个整分钟窗，但继续包含在完整均值、跨度和斜率。按真实间隔的梯形时间均值差29.197968MiB只作采样节奏敏感性描述，不替换原样本均值或预算。

下一次实际进程中先保留根/子代句柄，核当前创建代际、实际角色和版本，再读取资源；旧样本没有保存的角色不能补造。原ETW成功对照不解除bd2失败，DPI专项已取消；全部其它门禁、Mac依赖与禁止发布保持。
"""
(ROOT / "README.md").write_bytes(readme.encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
document = PROJECT / impact_path
old = document.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 复用有效样本收窄RSS取证目标"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

离线复算原019/823有效557/559样本，无新产品或配对。各版自身最大WebView成员全程代际稳定，均值205.181329→231.411428MiB，差26.230100MiB占总差89.7634%；其它成员差2.991261MiB。仅按大小分解，未匹配跨版本角色或把历史PID称为GPU/renderer，亦不当作可回收量或根因。

第一相对分钟已差26.577281MiB、第十相对分钟差25.535795MiB；两版全跨度斜率1.661734/2.057205MiB每分钟。十个整分钟各排除相对600秒后的1个末样本，但完整原均值和斜率仍含全部记录；时间加权差29.197968只作描述，不换验收指标。不能据600秒确认/排除泄漏。方法、摘要与完整分解见`.adg/evidence/OCX-0110-20261016-WIN-RSS-TIMELINE/`，下一步在真实新代际先取角色及分配证据，不重跑未改823的正式配对。

原+29.221361MiB相对+10MiB预算继续failed，bd2实际失败窗口仍缺，Mac依赖与其它原门禁不变；DPI专项已取消，TASK revision2/in_progress/pending，不关闭或发布。
"""
document.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import build_receipt, validate_object
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-RSS-TIMELINE-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
receipt = build_receipt(task, receipt_id=rid, result="failed", changed_paths=[impact_path, rpath],
    checks=[
        {"name": "local:windows-existing-rss-sample-decomposition", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RSS-TIMELINE/analysis.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-823-IDLE-PAIR/comparison.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(document.read_bytes()).hexdigest(),
        "summary": "原有效样本描述性分解将89.8%差额收窄到各自最大WebView成员，角色/根因未建立，原RSS失败不变。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="复用有效样本定位内存差额分布，无新配对；按大小最大WebView差26.23MiB，仍未证明角色或根因，原预算保持failed。")
validate_object(receipt)
for path, value in [(PROJECT / rpath, receipt), (ROOT / "manifest.json", {
    "files": [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
               "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
              for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"],
    "frozen": True, "newNativeSamplesTaken": False, "acceptanceMetricChanged": False,
})]:
    assert not path.exists()
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
print(json.dumps({"receipt": rpath, "originalRssBudgetPassed": False, "newNativeSamples": 0}))
