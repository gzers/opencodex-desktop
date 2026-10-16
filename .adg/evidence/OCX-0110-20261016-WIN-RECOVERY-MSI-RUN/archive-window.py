"""Freeze this completed foreground window; never execute product/installers."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime")
RECOVERY = RUNTIME / "foreground-823-recovery-20261016T183434"
MSI = RUNTIME / "host-msi-prepared-20261016T1740"
PROFILE = RUNTIME / "recovery-platform-preparation-v2-20261016T1612"
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
SID = re.compile(r"S-1-\d+(?:-\d+){1,}")
assert not (ROOT / "manifest.json").exists(), "Frozen archive must not be rewritten"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(relative, value):
    path = ROOT / relative
    assert not path.exists(), str(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


originals = []


def copy(relative, source):
    data = source.read_bytes()
    text = data.decode("utf-8-sig")
    assert not SID.search(text), "Raw SID must stay private"
    target = ROOT / relative
    assert not target.exists(), str(target)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    originals.append({"path": relative, "sourceRuntimePath": str(source),
                      "bytes": len(data), "sha256": sha(data)})


recovery_names = [
    "human-window.json", "execution-scope.json",
    "platform-preflight-before-prepare.json", "platform-preflight-before-launch.json",
    "recovery-prepared.json", "recovery-launch.json", "computer-use-observation.json",
    "recovery-census.original.json", "recovery-process-order.json",
    "observe-normal-exit.ps1", "normal-exit-observer-ready.json",
    "normal-exit-observer.failure.json", "post-human-exit-readback.json",
    "recovery-restored.json", "final-protection-readback.json",
]
for name in recovery_names:
    copy("recovery/" + name, RECOVERY / name)
copy("recovery/profile-restoration-result.json", PROFILE / "profile-restoration-result.json")
for name in [
    "human-window.json", "execution-plan.json", "elevated-execution-runner.ps1",
    "elevated-launch.json", "execution-attempt.json", "execution-result.json",
    "runner-result.json", "final-state-readback.json",
]:
    copy("msi/" + name, MSI / name)
copy("methods/execution-prepared.original.ps1", RUNTIME / "host-msi-execution-20261016T1740.ps1")
prepared_method = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/methods/execution-prepared.original.ps1"
assert (ROOT / "methods/execution-prepared.original.ps1").read_bytes() == prepared_method.read_bytes()

result = json.loads((MSI / "execution-result.json").read_text(encoding="utf-8-sig"))
steps = [step for step in result["steps"] if "exitCode" in step]
assert result["result"] == "pass" and len(steps) == 6
private_logs = []
for step in steps:
    name = step["name"] + ".original.log"
    source = MSI / name
    raw = source.read_bytes()
    encoding = "utf-16" if raw.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8-sig"
    text = raw.decode(encoding)
    redacted, count = SID.subn("[REDACTED_WINDOWS_SID]", text)
    assert count > 0 and "MainEngineThread is returning 0" in redacted
    relative = "msi/logs/" + step["name"] + ".redacted.log"
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    assert not target.exists()
    target.write_bytes(redacted.encode("utf-8"))
    private_logs.append({
        "originalPrivateRuntimePath": str(source),
        "originalBytes": len(raw), "originalSha256": sha(raw), "originalEncoding": encoding,
        "publicDerivedPath": relative, "publicDerivedSha256": sha(target.read_bytes()),
        "redactedSidOccurrences": count, "publicFileIsOriginal": False,
        "transformation": "decode original encoding; replace Windows SID matches only; encode UTF-8",
    })
write("original-copies.json", originals)
write("private-log-bindings.json", private_logs)
write("scope.json", {
    "productSource": SOURCE,
    "candidateExeSha256": "2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53",
    "actualHumanWindow": ["2026-10-16T10:34:34+00:00", "2026-10-16T11:04:34+00:00"],
    "foregroundReleasedAtLocal": "2026-10-16T18:48:39+08:00",
    "productRecoveryPassed": False, "recoveryDefectEstablished": False,
    "recoveryUiBlocker": "directory picker capture and activation failed; no automatic retry",
    "hostMsiSeededRetentionSubsetPassed": True,
    "customRootIsSeededSiblingNotApplicationBinding": True,
    "fullInstallerGatePassed": False, "explicitPurgeTested": False, "nsisTested": False,
    "administratorProductUiTested": False, "administratorHarnessTokenObserved": True,
    "normalProductExitCodeKnown": False, "productAbsentAfterHumanExit": True,
    "formalIdlePairRepeated": False, "newGpuSnapshotTaken": False,
    "privateOriginalLogsRetained": True, "rawSidOrSddlPublished": False,
    "finalStateReadbackSourceWasNull": True,
    "sourceBinding": "execution-plan.json SHA-256 and candidate admission; not the null readback field",
    "historicalFailuresCleared": False, "taskClosed": False, "releaseAccepted": False,
})

readme = """# 本机恢复窗口阻塞与MSI升级、卸载保留实测

2026-10-16用户“允许前台”的人工窗口为18:34:34–19:04:34（北京时间）。实际18:48:39释放前台。复用已准入823a8a06候选和原预算，未重跑正式019/823配对。

18:37:31只启一个普通令牌823实例（false/type3、原同SID）。v2前置确认四条逻辑平台路径均不存在；实际出现“安装目录下的数据目录无法写入”的恢复提示。点击“选择目录”后，实际“选择管理器数据目录”窗口无法被Computer Use捕获/激活：`CreateForMonitor ... 0x80070057`、`failed to activate captured window`。工具停止输入并交给用户，240秒观察结束仍无替换代次，严格交接未通过；不将本次工具阻塞认定为产品缺陷，不自动重试或强杀。UI记录是现场观察摘要，截图仅在工具中展示，未虚构截图文件。

正常退出观察器在18:43:51达到300秒上限；用户随后报告“已退出”，18:45:50进程普查为零。没有在保留句柄有效期内取得退出，因此退出码和内核退出时刻未知，不补写exit0。18:46:01精确恢复install-d权限与HKCU原指针；18:46:21按原生目录ID将两份保存目录恢复到四条逻辑别名，18:49:32再次读回原ID/创建时刻及原指针摘要一致。未删除配置或原始SDDL，私有原件继续留忽略运行目录。

18:46:51–18:46:56沿冻结一次性方法，以同用户真实管理员令牌（true/type2、TokenUser摘要同普通实例）执行六步：卸载此前本机019、安装正式019、升级823、默认卸载823、重装823、再次默认卸载。六步退出码全0，六份原MSI日志均有引擎返回0；正式019实装EXE为626d63ad…，两次823实装为2614489f…，均与固定包载荷相符。安装路径包含空格，安装器禁自动启动产品，未启动第二原生实例。

六份种子文件分别位于安装目录data（偏好及备份）、兄弟custom-root（偏好及备份）和外部哨兵；升级、卸载、重装及最终卸载后逐份哈希一致。最终三套产品码在64/32位HKLM及HKCU卸载项均不存在，旧安装和测试安装主EXE均不存在、native0。旧本机安装按用户后续明确卸载授权移除；原配置目录已恢复。

这里只通过真实宿主MSI包、升级和**种子文件默认保留子集**。custom-root没有实际应用绑定，种子不代表真实业务迁移；明确清理、NSIS、受保护路径普通令牌、产品双令牌UI/首屏及其它原安装场景未由本次补齐。管理员证据属于执行器令牌，不算管理员产品令牌。`execution-attempt.json`的初始fail是执行前不可变标记，最终结果以`execution-result.json`及六份日志核验；最终状态读回的source字段初版为null，真实源码绑定来自冻结计划与候选，不从该null字段推断。

原MSI详细日志包含Windows SID，保持原字节留在忽略运行目录；公开档只含明确标注的SID脱敏派生日志，并逐份记录私有原件路径/字节数/SHA-256及精确变换。不是把派生件冒称原件。恢复私有原指针/SDDL不提交。`verify-window.py`分别复算私有原件、派生件、现场原计数、身份、载荷链、保留和还原；仅证据校验，不运行产品或安装器。

RSS+29.221361MiB超过原+10MiB、bd2实际失败窗口未取得、Mac依赖及所有其余原门禁保持；DPI专项按用户取消。TASK revision2/in_progress/pending不变，不合并、关闭、签名、晋升或发布。
"""
(ROOT / "README.md").write_bytes(readme.encode("utf-8"))
(ROOT / ".gitattributes").write_bytes(b"* -text\n")

impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
document = PROJECT / impact_path
before = document.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 恢复选择器受阻并完成本机MSI升级保留子集"
assert heading.encode("utf-8") not in before
addition = f"""{heading}

用户18:34:34新“允许前台”窗口内，v2真实平台根前置通过，18:37:31唯一普通false/type3同SID823实例实际出现不可写恢复提示。点击选择目录后，实际目录选择器捕获0x80070057/激活失败，停止工具输入并交给用户；240秒无替换代次、原始普查最大native1，严格交接未通过，不建立产品缺陷、不自动重试。正常退出观察器18:43:51超时，用户随后退出，18:45:50native0；退出码/内核退出时刻未知，不补写exit0。18:46精确恢复ACL/HKCU及两份原生目录，四逻辑别名原ID/创建时刻和原指针摘要再次核同。

同窗口18:46:51–56按冻结方法，以同用户实际管理员true/type2令牌完成卸旧本机019→正式019→823升级→默认卸载→重装→最终默认卸载，六步退出0及原MSI引擎返回0均核验。019实际载荷626/823两次261与固定包一致；含空格安装路径、六份安装data/兄弟custom-root/外部哨兵在五阶段哈希保持。最终三产品码卸载项/主EXE均不存在、native0，旧本机安装依用户后续卸载授权移除，原配置已恢复。18:48:39释放前台，无正式配对重复或新GPU取样。

证据与独立校验见`.adg/evidence/OCX-0110-20261016-WIN-RECOVERY-MSI-RUN/`。原MSI日志含SID，仅私有忽略目录保留原字节，公开明确标记脱敏派生件及原摘要，不冒称原件。执行前fail标记与最终pass分别保留；最终状态source初版null不作为源码绑定，沿冻结计划/候选身份核验。只通过宿主MSI升级/种子默认保留子集，不将兄弟custom-root种子称应用绑定，管理员执行器令牌不称产品令牌；完整安装/明确清理/NSIS/双令牌首屏等继续待验。

恢复交接、RSS+29.221361MiB、bd2实际失败窗口、Mac依赖及全部其余原门禁保持；DPI专项已取消。TASK revision2/in_progress/pending、IMP未完成，不合并、关闭或发布。
"""
document.write_bytes(before + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import build_receipt, validate_object
task_path = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
assert sha(task_path.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
task = json.loads(task_path.read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-RECOVERY-MSI-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
evidence_ref = ROOT.relative_to(PROJECT).as_posix()
receipt = build_receipt(
    task, receipt_id=rid, result="failed", changed_paths=[impact_path, rpath],
    checks=[
        {"name": "local:windows-host-msi-seeded-retention", "status": "passed",
         "evidence": evidence_ref + "/msi/execution-result.json"},
        {"name": "local:windows-recovery-prerequisites-restored", "status": "passed",
         "evidence": evidence_ref + "/recovery/final-protection-readback.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": evidence_ref + "/recovery/recovery-process-order.json"},
        {"name": "review:installer-0110", "status": "failed",
         "evidence": evidence_ref + "/scope.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
                   "after_digest": sha(document.read_bytes()),
                   "summary": "真实宿主MSI升级和种子默认保留子集通过；恢复选择器受工具阻塞未通过，已还原配置；完整门禁不提升。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="本机MSI六步0退出与六文件保留通过；恢复240秒未交接且正常退出码未知，原配置精确恢复，TASK完整原生及安装门禁仍未通过。")
validate_object(receipt)
receipt_path = PROJECT / rpath
assert not receipt_path.exists()
receipt_path.write_bytes((json.dumps(receipt, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
write("manifest.json", {
    "frozen": True,
    "files": [{"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size,
               "sha256": sha(path.read_bytes())}
              for path in sorted(ROOT.rglob("*")) if path.is_file() and path.name != "manifest.json"],
    "taskUnchanged": True, "fullInstallerGatePassed": False, "recoveryPassed": False,
    "hostMsiSeededRetentionSubsetPassed": True, "privateOriginalLogsRetained": True,
})
print(json.dumps({"receipt": rpath, "copiedOriginals": len(originals),
                  "privateMsiLogs": len(private_logs), "hostMsiSubsetPassed": True,
                  "fullGatesPassed": False}, ensure_ascii=False))
