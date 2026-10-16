"""Seal one authorized trace without converting its success into failure evidence."""
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


assert not (ROOT / "manifest.json").exists(), "Never extend frozen evidence"
verification = json.loads(subprocess.check_output(
    [sys.executable, str(ROOT / "verify-run.py")], cwd=PROJECT))
write(ROOT / "verification.json", verification)
network = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/raw-scheduling-network-20261016T1645")
for name in ["anonymous-rate-limit.original.json", "proxy-probe-result.json",
             "azure-tls-negotiation.original.headers", "azure-tls-negotiation.json"]:
    destination = ROOT / "network-recovery" / name
    destination.parent.mkdir(exist_ok=True)
    assert not destination.exists()
    destination.write_bytes((network / name).read_bytes())
write(ROOT / "final-method-binding.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "source": "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56", "runId": 38876971001,
    "methodSha256": {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest() for name in [
        "github-reader-via-proxy.py", "download-ci-logs-via-proxy.py", "admit-ci-via-proxy.py",
        "dispatch-once-via-proxy.py", "retrieve-recording.py", "analyze-recording.py", "verify-run.py",
    ]},
    "transportChangesOnlyBeforeDispatch": True,
    "githubApiTlsMinimumAndMaximum": "1.2 (original protocol bound retained)",
    "anonymousBlobTlsMinimum": "1.2; allow negotiated newer versions",
    "proxy": "socks5h://[::1]:3060", "certificateVerificationEnabled": True,
    "globalNetworkSettingsChanged": False, "crossHostCredentialsForwarded": False,
    "analysisCountCorrectionOnly": "source callbacks versus direction rows; preserve collision multiplicity",
    "firstDerivedAnalysisRetained": True, "rawEvidenceChanged": False,
    "postCaptureLibraryExecution": False, "anotherOriginalRunRequested": False,
})
readme = """# 单次原始调度字节录制：有效成功对照，目标失败未复现

同一54927d2c首次CI38858859494完整准入：五必需作业成功，三份原job日志分别核实际checkout549、前端99文件535项、Mac923/6ignored及Windows648/7ignored（后端各35组）。本机DNS/curl6、代理初版TLS/curl35及原始空传输保留。最终使用既有mihomo的IPv6回环SOCKS，命令级代理与分域TLS参数完成下载；证书校验始终开启，未更改系统网络、停止外部进程或跨主机转发GitHub凭据。

按既有管理员后台授权仅派发一次query-etw/isolated-replay，run38876971001/attempt1。固定原bd2失败库497814ca…、官方Node24.21.0/ba4e6d11…、TraceEvent3.1.21；原库无参数、默认并发、2/4/5/8秒、120秒外层、64MiB/32768队列及IRP预算不变，无预热。录制实际08:56:21–08:56:42 UTC，原库530passed/0failed/4ignored、七query均ok，目标失败未复现。runner实际提升type1，仅证明该诊断进程，不能作为本机产品双令牌或旧环境相同的证据。

制品12035074475，ZIP423839字节/SHA256 649ce8c01482b2cf964c4b3d16f54d51c230649c03ba9089134fce447599bda4；API摘要、CRC、13包内原件及实际job checkout核对。事件17296条/8153824字节/SHA256 54ee07da52b36fd51904d13a90cb5e11e02a781da50dae053ed3dd461171be4e，31对进程和939对线程，七个真实ThreadSetName。丢弃/内核丢失/解码错误/已知后代与命名缺口0，自有会话停止排空成功，未强清理。

调度字节只保存准入事件payload偏移8–15的8字节，排除线程ID前缀和未知尾部，v5语义仍未确认。采集器按5319次内核回调计数，生成6131行进/出记录：812组双方向共享一次回调；另有一组相同QPC/字节下的两条不同线程切出，不能把简化键当唯一事件ID。初版离线把回调数等同行数而拒绝，原方法/失败保留；按源码定义和方向多重性复算下界5319，与原计数一致。getter读回与偏移12/13/14字节相符只证明解码来源，不赋予等待状态语义。

根命名查询线程热点从命名事件后开始；其它根调度156797次和文件发起287105次按原范围省略，不补写或声称完整根热点。成功对照不能解除bd2历史失败，也未获得实际失败窗口或根因。本轮结束后不追加原库回放，下一次自然失败应复用既有首轮采集能力留证，而非反复跑成功轮。

事件JSONL采用确定性gzip逐字节往返核验，未另复制源码或凭据。制品取回后补job日志曾遇TLS错误，续取只复核已有ZIP与原件，没有重下载或重启录制。旧checkpoint-manifest原字节保留，当前manifest封存整个本轮；冻结后不得再执行派发、下载或状态写入方法。

17:33本机native0、日常EXE/TASK摘要保持，四条逻辑管理器根仍不存在、两份保存目录原生ID保持、指针未改，823/768代码工作区无tracked改动。RSS+29.221361MiB、Mac失败、严格恢复和其余全部原门禁继续开放；DPI专项按用户裁决取消，不合并、关闭、签名、晋升或发布。
"""
(ROOT / "README.md").write_bytes(readme.encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
impact_file = PROJECT / impact_path
old = impact_file.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 单次549原始调度字节录制完成，仍未复现bd2失败"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

通过既有IPv6 SOCKS命令级代理及分域TLS参数取回同549首次CI38858859494三份原日志，实际checkout/前端535/Mac923/Windows648及五必需job核验完整。原DNS与TLS失败保留，证书校验未关闭、系统网络未改。沿持续管理员后台授权仅一次派发38876971001/attempt1，固定原bd2库497814ca/官方Node24.21.0-ba4e6d11，原并发/截止/外层/容量预算均不变、无预热；原库530passed/0failed/4ignored、七queryok，仍为成功对照，历史失败不解除。

制品12035074475摘要/CRC/13原件与job日志核验；事件17296条/8153824字节、31对进程/939对线程、七真实命名，丢失/解码/已知后代和命名缺口0，自有会话停止排空无强清理。偏移8–15字节原件6131行对应源计数5319次回调；812组进出双行及一组简化QPC/字节碰撞，离线初版计数假设错误原件保留，复算不改原数据、不声称重建唯一内核事件。v5等待语义仍false，根其它调度156797和文件发起287105按范围省略。

原件gzip往返及完整核验见`.adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/`。实际bd2失败窗口与根因仍未取得，本轮不再追加回放。17:33本机native0、日常/TASK摘要及保存配置原生ID/原指针保持；产品823与工具768无tracked改动。RSS+29.221361MiB、Mac失败、严格恢复与其余原门禁保持，DPI专项已取消；TASK revision2/in_progress/pending，不合并、关闭或发布。
"""
impact_file.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-RAW-SCHEDULING-RUN-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
receipt = build_receipt(task, receipt_id=rid, result="failed", changed_paths=[impact_path, rpath],
    checks=[
        {"name": "local:windows-raw-ci-log-admission", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/ci-admission.json"},
        {"name": "local:windows-one-raw-scheduling-capture", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/verification.json"},
        {"name": "local:windows-bd2-actual-failure-window", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/recording-analysis.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-823-IDLE-PAIR/comparison.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(impact_file.read_bytes()).hexdigest(),
        "summary": "完整CI原日志准入与单次固定bd2成功录制已核验；原失败窗口/根因、RSS预算及全部未完门禁保持。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="单次549调度字节录制完整但bd2失败未复现；保留成功对照与原失败，不再追加回放，不关闭TASK或发布。")
validate_object(receipt)
write(PROJECT / rpath, receipt)
files = [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
          "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
         for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"]
write(ROOT / "manifest.json", {"files": files, "frozen": True, "runId": 38876971001,
    "actualFailureWindowObtained": False, "anotherOriginalRunRequested": False})
subprocess.run([sys.executable, str(ROOT / "verify-run.py")], cwd=PROJECT, check=True)
print(json.dumps({"sealedFiles": len(files), "receipt": rpath, "result": "failure-window-not-obtained"}))
