"""Record the governed result writeback only; never advance or close TASK."""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
SKILL = Path("C:/Users/15119/.codex/skills/project-governance")
sys.path.insert(0, str(SKILL))
from scripts.lib.runtime.kernel import build_receipt, validate_object

assert not (ROOT / "manifest.json").exists(), "Frozen archive"
TASK_PATH = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
task = json.loads(TASK_PATH.read_text(encoding="utf-8-sig"))
IMP = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
REPORT = "docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md"
RECEIPT = ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-823-IDLE-20261016.json"
EVIDENCE = ".adg/evidence/OCX-0110-20261016-WIN-823-IDLE-PAIR/"
target = PROJECT / RECEIPT
assert not target.exists(), "Immutable receipt; never overwrite"
comparison = json.loads((ROOT / "comparison.json").read_text(encoding="utf-8-sig"))
verified = json.loads((ROOT / "verification.json").read_text(encoding="utf-8-sig"))
assert verified["verification"] == "passed" and comparison["validPairForRecordedSubset"]
assert not comparison["meanRssBudgetPassed"] and comparison["cpuBudgetPassed"]
task_before = TASK_PATH.read_bytes()
receipt = build_receipt(
    task,
    receipt_id=target.stem,
    result="failed",
    # This receipt covers fact/result writeback. Raw execution and copied
    # evidence retain their own immutable identities and copy receipt.
    changed_paths=[RECEIPT, IMP, REPORT],
    checks=[
        {"name": "local:win-823-idle-pair-integrity", "status": "passed",
         "evidence": EVIDENCE + "verification.json"},
        {"name": "local:win-823-idle-cpu-budget", "status": "passed",
         "evidence": EVIDENCE + "comparison.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": EVIDENCE + "comparison.json"},
    ],
    acceptance_results=[],
    fact_impacts=[{
        "target": "imp-20", "path": IMP,
        "after_digest": hashlib.sha256((PROJECT / IMP).read_bytes()).hexdigest(),
        "summary": "记录正式019/823各600秒配对及RSS超预算；严格恢复交接和原全部未完门禁保持。",
    }],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="结果回写回执：019/823完整空闲配对CPU差−5.311683pp，平均RSS差+29.221361MiB超过+10MiB；两版正常exit0/native0，不关闭TASK或发布。",
)
validate_object(receipt)
target.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
assert TASK_PATH.read_bytes() == task_before
print(json.dumps({"receipt": RECEIPT, "result": receipt["result"],
                  "taskUnchanged": True, "contentDigest": receipt["content_digest"]}))
