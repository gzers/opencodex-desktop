"""Seal this preparation checkpoint; the same-CI network RUN stays open."""
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


def write(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


assert not (ROOT / "manifest.json").exists(), "Do not modify a frozen preparation"
verification = json.loads(subprocess.check_output(
    [sys.executable, str(ROOT / "verify-preparation.py")], cwd=PROJECT))
write(ROOT / "verification.json", verification)
(ROOT / "README.md").write_bytes("""# 恢复前置修正：四条逻辑路径对应两份原生目录

按用户10月16日本机配置清理授权执行，仅处理已核实的OpenCodex桌面管理器目录。当前普通Codex宿主上下文中，标准AppData和应用容器LocalCache的Roaming/Local四条路径，分别映射到两个相同原生目录ID；GetFinalPathNameByHandle给出容器内实际路径。此为实际句柄证据，不按路径字符串或创建时间猜测，也不推广到管理员、Explorer或另一宿主上下文。

首次方法按四条路径移动，完成两次后第三条已不存在，停止并保留失败。先把实际完成的两次移动还原，核原生目录ID/创建时刻完全相同，再按真实目录ID去重隔离：两份原目录对象各移动一次，全部内容保存在忽略的运行目录，四条原逻辑路径均不存在。未删除、发布配置内容，HKCU指针和ACL未改。

独立检查核对12份运行原件、原生ID的分组/还原/再次保存、空平台根拒绝、两个新入口缺人工窗口拒绝、最终native0及原指针保持。空目录拒绝测试只创建并非递归删除了一份空探针；其初版辅助函数作用域错误、清理读回和修订方法均保留。完整新还原脚本仅语法核验，先前两次部分还原确已执行；不把准备、空探针或普通观察器令牌算作产品恢复或管理员通过。

后续新人工窗口使用本目录`prepare-recovery-v2.ps1`和`launch-recovery-v2.ps1`，同时传入新的RunDirectory与`preparation-binding.json`中的runtimePrepared。两个入口在原冻结人工窗口/候选身份守卫之后再核实际平台根不存在及两份保存目录原生ID，然后调用原恢复控制器；不得绕过原控制器或直接改冻结文件。新窗口授权以前不启动产品。

结束时先正常退出实际替换代际，确认native0，再恢复原ACL/指针。若应用新建了管理器profile，先将新profile按实际原生ID保存到本轮忽略目录，待原位置不存在后才用`restore-platform-roots.ps1`还原旧profile；该脚本拒绝覆盖新状态。已有保存目录继续保留，不能当清理垃圾删除。

16:23读回为本普通调用上下文四路径不存在、两原目录ID保持、native0、指针原字节摘要保持、日常EXE/TASK摘要未变，823/GPU工具分支均无tracked改动。此次没有产品、安装器、ETW或正式性能执行。专项DPI按用户裁决取消；其余原门禁与发布限制保持。
""".encode("utf-8"))
raw = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN"
(raw / "README.md").write_bytes("""# 同一549首次CI的原日志继续入口

本RUN继续CI38858859494/source54927d2c。旧冻结目录保持原件，复制的准入/派发方法与输入由continuation-binding记录来源。一次同job日志读回在blob域名解析处返回curl6，空传输及失败回执保留；一次不修改系统设置且保留TLS验证的独立HTTPS解析路径连接失败，curl7/HTTP000，结果注明来自工具命令观察。未取得完整日志，未生成ci-admission，也未派发ETW或执行原库/官方Node。

checkpoint-manifest固定本次文件字节，RUN目录保持打开；仅在网络状态改变后继续同一CI原日志，不重复不变失败路径，不新建CI或重发录制。准入仍需实际checkout、五必需job及三份原日志完整核验，原并发/2/4/5/8秒/120秒/64MiB/32768均不变，最多派发一次。下载方法只增加禁止写Python字节码缓存；准入方法只把新终态另存ci-admission-terminal，保留旧原件，判定条件不改。

本次没有修改外部进程或系统网络。bd2实际失败窗口/根因仍缺，成功回放不能解除历史失败。本机恢复准备另见WIN-RECOVERY-PLATFORM-PREP，Windows/Mac资源失败和全部其余门禁保持。
""".encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
impact_file = PROJECT / impact_path
old = impact_file.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 按原生目录ID修正恢复夹具，ETW原日志仍受阻"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

按用户本机OpenCodex配置清理授权，实际句柄核到当前普通Codex宿主的四条逻辑AppData/LocalCache路径只对应两份原生目录；首次逐路径移动在第三条不存在时停止，已完成两次移动先按原生ID还原，再按真实ID分组各保存一次。四条逻辑路径现均不存在，两原目录对象及创建时刻保持，内容留忽略运行目录，未删除或发布配置。HKCU指针/ACL未改；不把该命名空间推广为管理员或另一启动上下文。

12份原件、独立校验、空平台根拒绝与两个缺人工窗口拒绝见`.adg/evidence/OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/`；原恢复控制器不改，新v2入口补实际根缺失与原生ID守卫。16:23读回native0、日常/TASK摘要保持，823/768两代码工作区无tracked改动，未启动产品或安装器。新完整profile还原脚本仅语法检查，先前部分还原已实际核ID；严格产品恢复仍待新的人工窗口，不能把本准备算通过。

同549首次CI38858859494的原日志继续档`.adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/`保留一次curl6/DNS失败与一次独立只读HTTPS路径curl7连接失败；网络状态未改变时停止相同读回。未准入、派发或录制，原bd2库与官方Node未执行，未动外部进程或系统网络。后续沿同CI及同RUN，最多一次修订录制，原预算/断言不改。

源码检查否定“所有路由同时挂载”的假设：App.vue只挂当前动态组件；光层几何有变化，但没有原生分配归因，不做臆测优化或重跑未改823配对。+29.221361MiB RSS、bd2实际失败窗口、Mac失败和其余原门禁保持，DPI专项按用户裁决取消；TASK revision2/in_progress/pending、禁止合并/关闭/签名/晋升/发布不变。
"""
impact_file.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-PLATFORM-PREP-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
receipt = build_receipt(task, receipt_id=rid, result="failed", changed_paths=[impact_path, rpath],
    checks=[
        {"name": "local:windows-native-directory-isolation", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/verification.json"},
        {"name": "local:windows-platform-and-human-window-guards", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/prepared/guard-validation.original.json"},
        {"name": "local:windows-raw-ci-log-admission", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/network-continuation-checkpoint.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(impact_file.read_bytes()).hexdigest(),
        "summary": "实际保存两份原生配置目录并核四逻辑根缺失，保持人工窗口守卫；ETW日志仍受阻、RSS根因与原门禁未完成。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="恢复夹具原生目录映射已修正、缺根/人工窗口守卫通过；ETW日志下载阻塞，未录制或启动产品，原失败与TASK未完成状态保持。")
validate_object(receipt)
write(PROJECT / rpath, receipt)
write(ROOT / "manifest.json", {"files": [
    {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
     "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
    for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"
], "frozen": True, "fullRecoveryGatePassed": False, "profileContentsArchived": False})
subprocess.run([sys.executable, str(ROOT / "verify-preparation.py")], cwd=PROJECT, check=True)
print(json.dumps({"receipt": rpath, "result": "failed-network-admission; platform-preconditions-verified"}))
