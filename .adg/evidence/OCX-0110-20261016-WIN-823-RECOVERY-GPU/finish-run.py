"""Write the failed recovery result and independent snapshot; never close TASK."""
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
verification = json.loads(subprocess.check_output(
    [sys.executable, str(ROOT / "verify-run.py")], cwd=PROJECT))
write_json(ROOT / "verification.json", verification)
(ROOT / "README.md").write_bytes("""# 823单次恢复尝试与原生GPU快照

2026-10-16用户“允许继续前台测试”的本轮窗口为北京时间15:18:26–15:48:26。只启动一次精确823产品，未重新打包或重复019/823空闲配对，未使用sandbox/CDP。实际普通产品令牌false/type3，TokenUser摘要与原账户相同。

15:20:36从自有install-d启动后直接进入正常主界面，未观察到恢复选择器；设置实际数据根为Codex MSIX容器LocalCache/Roaming下的管理器旧测试根。生产823的`bootstrap::default_anchor`明确优先保留已有platform root；旧准备只清除HKCU并拒绝install-d创建，遗漏该已存在的平台根，未隔离本场景的前置条件。240秒控制器保留485次原始计数，最大原生数1、没有替换代次，原生恢复子集**未通过**。这轮未建立恢复交接产品缺陷，也不证明修复通过；无自动重试或强杀。

在这同一个真实普通产品代际上，工具768e2318读取一次保留句柄的进程族及GPU计数器：7个成员，识别1个GPU进程；两个GPU实例的共享使用量合计191668224字节，GPU进程RSS246169600字节。前者不能与RSS直接相加，也不能据此归因既往配对差值或单个GL缓冲。没有019同期GPU对照，未测600秒平均。工具首次CI在窗口前全success，完整job原日志仍未下载核验。

15:25:35经Sky向唯一实例发送Alt+F4，保留句柄证实exit0，随后native0。冻结恢复脚本先恢复ACL，随后因当前指针已指向平台根而拒绝覆盖；保留该拒绝。后台核验当前指针与捕获原字节一致、根与最后UI一致且native0后，恢复私有备份中的原指针，再核ACL/指针完全相同。私有SDDL和原始SID只留忽略运行目录，17份公开运行原件逐字节归档。

15:32:09保护读回确认日常EXE与TASK摘要未变，两份真实Roaming/Local旁存仍在；没有再启动产品或移动、删除数据。UI保存的是实际可访问性读回，截图仅在工具中观察，未把不存在的截图文件算进证据。`verify-run.py`独立核身份、原始计数、失败状态、GPU整数值、真实退出、精确还原及清单。

后续恢复夹具必须在启动前同时核对HKCU和**实际平台根**，包含本次已观察到的应用容器路径；平台根存在时不能仅靠清除指针声称全新安装。先完成仅限自有管理器目录的可回滚隔离与读回，再在新的人工窗口补一次严格交接。保留本次失败，不在本轮内重跑。

Windows平均RSS+29.221361MiB超过+10MiB、历史Windows/Mac资源失败、bd2实际失败窗口及全部原预算/周期/八门禁保持；DPI待验。TASK revision2/in_progress/pending不变，不合并、关闭、签名、晋升或发布。
""".encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
impact_file = PROJECT / impact_path
old = impact_file.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 823恢复前置遗漏，取得一次真实GPU快照"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

用户15:18:26新前台窗口内只启一次823精确EXE，实际普通false/type3、原同SID，未用sandbox/CDP或重复配对。因已有Codex应用容器LocalCache/Roaming管理器测试根，生产按default_anchor规则进入原根；准备只清除HKCU/拒绝install-d创建，遗漏实际平台根前置。240秒485次原计数最大native1、无替换代次，恢复子集未通过，不建立产品恢复缺陷，不自动重试或强杀。

同一实际产品代际由768e2318工具保留句柄取得7成员/1GPU/2计数器实例，共享GPU使用191668224字节、GPU进程RSS246169600字节；只是一时快照，不相加、不倒推历史性能PID或归因GL缓冲。工具首次全CI在窗口前成功，原job日志未完整核验。15:25:35正常Alt+F4/exit0/native0；冻结restore先恢复ACL再因自动登记的平台根指针拒绝，随后核当前原字节/实际UI/native0后精确恢复原指针，ACL及指针读回相同，原拒绝保留。

17份运行原件与独立验证见`.adg/evidence/OCX-0110-20261016-WIN-823-RECOVERY-GPU/`，私有SDDL/原SID不发布。15:32:09日常/TASK摘要和真实旁存存在性保持，前台已释放，无重启或数据移动/删除。下一恢复准备须核实际平台根并隔离自有夹具后另取人工窗口；本次失败不清除。+29.221361MiB RSS、Mac/Windows历史失败、bd2失败窗口与全部原预算/周期/八检查保持；DPI待验，TASK revision2/in_progress/pending、IMP未完成及禁止发布不变。
"""
impact_file.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
receipt_id = "RCP-OPENCODEX-DESKTOP-182-WIN-823-RECOVERY-GPU-20261016"
receipt_path = f".adg/work/objects/rcp/{receipt_id}.json"
evidence = ".adg/evidence/OCX-0110-20261016-WIN-823-RECOVERY-GPU/"
receipt = build_receipt(task, receipt_id=receipt_id, result="failed",
    changed_paths=[impact_path, receipt_path],
    checks=[
        {"name": "local:win-823-gpu-native-snapshot", "status": "passed",
         "evidence": evidence + "verification.json"},
        {"name": "local:win-823-recovery-preconditions", "status": "failed",
         "evidence": evidence + "execution-notes.json"},
        {"name": "local:win-823-normal-exit-and-restoration", "status": "passed",
         "evidence": evidence + "run/final-protection-readback.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": evidence + "run/recovery-process-order.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-823-IDLE-PAIR/comparison.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(impact_file.read_bytes()).hexdigest(),
        "summary": "记录恢复前置遗漏与单次真实GPU快照、正常退出和精确还原；原恢复/性能失败及全部门禁保持。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="823恢复尝试未通过：已有平台根使恢复前置不成立；真实GPU快照和exit0/native0/ACL指针还原已核验，不关闭TASK或发布。")
validate_object(receipt)
write_json(PROJECT / receipt_path, receipt)
files = [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
          "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
         for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"]
write_json(ROOT / "manifest.json", {"files": files, "frozen": True,
    "recoveryGatePassed": False, "nativeSnapshotVerified": True, "productAcceptanceClaimed": False})
subprocess.run([sys.executable, str(ROOT / "verify-run.py")], cwd=PROJECT, check=True)
print(json.dumps({"sealedFiles": len(files), "receipt": receipt_path, "recoveryResult": "failed"}))
