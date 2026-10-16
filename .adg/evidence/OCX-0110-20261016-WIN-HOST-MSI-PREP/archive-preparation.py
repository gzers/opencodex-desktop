"""Archive a concrete host plan and its actual ordinary-token refusal."""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
PRODUCT_RUNTIME = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop/test/runtime")
DIAGNOSTIC_RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime")
PREPARED = DIAGNOSTIC_RUNTIME / "host-msi-prepared-20261016T1740"
assert not (ROOT / "manifest.json").exists()


def write(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


inputs = {
    "inventory.original.json": PRODUCT_RUNTIME / "host-msi-preparation-20261016T1739/msi-host-inventory.original.json",
    "methods/inventory.original.ps1": PRODUCT_RUNTIME / "read-host-msi-inventory-20261016T1739.ps1",
    "methods/execution-prepared.original.ps1": DIAGNOSTIC_RUNTIME / "host-msi-execution-20261016T1740.ps1",
    "methods/refusal-validation.original.ps1": PREPARED / "validate-ordinary-refusal.ps1",
    "execution-plan.original.json": PREPARED / "execution-plan.json",
    "plan-preparation.original.json": PREPARED / "plan-preparation.original.json",
    "ordinary-execution-refusal.original.json": PREPARED / "ordinary-execution-refusal.original.json",
    "initial-preparation-refusal.json": PREPARED / "initial-preparation-refusal.json",
}
copied = []
for relative, source in inputs.items():
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    assert not target.exists()
    data = source.read_bytes()
    target.write_bytes(data)
    copied.append({"path": relative, "sourceRuntimePath": str(source), "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
inventory = json.loads((ROOT / "inventory.original.json").read_text(encoding="utf-8-sig"))
plan = json.loads((ROOT / "execution-plan.original.json").read_text(encoding="utf-8-sig"))
prepared = json.loads((ROOT / "plan-preparation.original.json").read_text(encoding="utf-8-sig"))
refusal = json.loads((ROOT / "ordinary-execution-refusal.original.json").read_text(encoding="utf-8-sig"))
baseline, candidate = inventory["packages"]
assert baseline["properties"]["ProductCode"] == plan["packages"]["baseline"]["productCode"] == "{7C063778-E6A6-4D0E-9770-4C4349101866}"
assert candidate["properties"]["ProductCode"] == plan["packages"]["candidate"]["productCode"] == "{FF3E0F1E-C4FD-49F5-8AE0-CA6F410DA8AF}"
assert inventory["installedMsi"]["productCode"] == plan["priorLocalProductCode"] == "{7069DA97-F262-4585-98CA-AE416E6833A5}"
assert len({inventory["installedMsi"]["productCode"], baseline["properties"]["ProductCode"],
            candidate["properties"]["ProductCode"]}) == 3
assert baseline["properties"]["UpgradeCode"] == candidate["properties"]["UpgradeCode"]
assert baseline["sha256"] == plan["packages"]["baseline"]["sha256"] == "b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"
assert candidate["sha256"] == plan["packages"]["candidate"]["sha256"] == "e64491715bc5e510d06eb8a18f723b6700be800e4555a44c4e2207059bf0318c"
assert inventory["installedExe"]["sha256"] == plan["priorLocalExeSha256"] == refusal["priorExecutableSha256"]
assert all(p["properties"]["ALLUSERS"] == "1" and p["launchSequence"] == [{
    "action": "LaunchApplication", "sequence": "6601", "condition": "AUTOLAUNCHAPP AND NOT Installed"}]
    for p in inventory["packages"])
assert plan["steps"] == ["uninstall-existing-local-019", "install-official-019", "upgrade-to-823",
                        "default-uninstall-823", "reinstall-823", "final-default-uninstall-823"]
assert 'AUTOLAUNCHAPP=""' in plan["argumentsCommon"] and plan["maximumSecondsPerStep"] == 120
assert plan["administratorAndNewHumanWindowRequiredForExecute"] and not plan["automaticRetry"]
assert not plan["productLaunchAllowed"] and not plan["explicitPurgeTested"]
assert prepared["status"] == "prepared-only" and not prepared["actualHarnessToken"]["elevated"]
assert refusal["executionRejectedForOrdinaryToken"] and refusal["preparedPlanUnchanged"]
assert refusal["preparedPlanSha256"] == hashlib.sha256((ROOT / "execution-plan.original.json").read_bytes()).hexdigest()
assert not any(refusal[k] for k in ["executionAttemptCreated", "executionResultCreated", "humanWindowCreated",
    "installDirectoryCreated", "installerExecuted", "productStarted", "administratorExecutionTested",
    "actualApplicationRootBindingTested", "fullInstallerGatePassed"])
assert not (PREPARED / "execution-attempt.json").exists() and not (PREPARED / "human-window.json").exists()
write(ROOT / "verification.json", {
    "status": "verified-host-msi-preparation-only", "originalCopies": copied,
    "existingLocal019IsOfficialBaseline": False, "officialBaselineAndCandidatePackagesPinned": True,
    "perMachineMsiRequiresAdministrator": True, "actualOrdinaryTokenExecutionRejected": True,
    "currentInstallationUnchangedAtRefusal": True, "executionPlanUnchanged": True,
    "productLaunched": False, "installerExecuted": False, "administratorExecutionTested": False,
    "fullInstallerGatePassed": False, "explicitPurgeTested": False, "nsisTested": False,
    "actualApplicationRootBindingTested": False, "releaseAccepted": False,
})
(ROOT / "README.md").write_bytes("""# 本机MSI执行准备

只读Windows Installer数据库和已安装产品信息核查：本机0.1.9产品码7069DA97…、原EXE SHA43e06bc3…；正式0.1.9包产品码7C063778…/SHA b33dd0c8…；823包产品码FF3E0F1E…/SHA e6449171…。正式019和823具有相同UpgradeCode，均ALLUSERS=1。现有同版本软件不是本次正式基线包，不能直接记作正式019升级。

具体一次性计划已保存：卸载本机原产品→正式019→823升级→默认卸载→823重装→最终默认卸载。仅使用已固定的本机包，普通应用EXE始终不启动，`/qn /norestart AUTOLAUNCHAPP=""`禁自动启动；主EXE版本与精确载荷摘要逐步核对。六份可删除夹具覆盖安装目录data、custom-root兄弟目录和越界保护样本的原字节保留。custom-root只是种子夹具，不冒充应用绑定；不覆盖真正首屏、安装目录根、受保护目录普通令牌或明确清理/NSIS完整门禁。

方法位于`verification.json`中记录的忽略runtime，真实执行需`-Execute`、同一用户实际管理员令牌及新的`human-window.json`，scope为`one-host-msi-retention-execution`、最长30分钟。每个安装步骤开始前再验窗口、native0，单步最多120秒，失败/超时不强杀、不自动重试。计划还核现有注册路径、原EXE和缓存MSI摘要，状态漂移即拒绝。提权应由人类处理，本方法不点击权限对话框。

本轮只做只读盘点、语法/准备以及真实普通令牌调用Execute被拒绝的检查；执行标记、安装目录和人类窗口均未创建，旧安装未改，native0。初次把库存目录和执行边界混用被前置拒绝，随后在独立诊断runtime准备，原库存保持。没有调用安装器，也没有验证管理员执行。恢复夹具仍独立就绪；下一次前台应先完成严格恢复并正常退出，再执行本机MSI专项。

其它未完成门禁、RSS+29.221361MiB、bd2实际失败窗口缺失和Mac依赖保持；DPI专项已按用户取消，不合并、关闭或发布。
""".encode("utf-8"))
impact_path = "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
document = PROJECT / impact_path
old = document.read_bytes().replace(b"\r\n", b"\n").rstrip(b"\n")
heading = "### 2026-10-16 本机MSI正式基线链与执行守卫就绪"
assert heading not in old.decode("utf-8")
addition = f"""{heading}

只读MSI库存核对本机已装0.1.9产品码7069与正式基线019的7C06不同，不能混称正式基线；正式包b33dd0c8/823包e6449171均固定，同UpgradeCode且ALLUSERS=1。按已获本机卸载/配置清理授权，准备一次“卸旧本机包→正式019→823升级→卸载→重装→最终卸载”，命令禁自动启动，校验实际载荷并保留种子data/custom-root/兄弟样本。custom-root只作种子、不算应用绑定，明确清理/NSIS/首屏和其它安装场景不提升。

`.adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/`保存盘点、一次性计划与实际方法。真实普通令牌调用Execute在安装/写标记前被拒绝，旧EXE/注册及计划摘要不变、native0；未调用安装器。执行仍需同用户管理员实际令牌及新的最长30分钟人工窗口；每步核窗口、native0与120秒上限，状态漂移/失败不强杀或重试。初次目录边界拒绝保留，未改全局设置。

恢复夹具已另行准备，下一人工窗口先补严格交接并正常退出，再做本机MSI专项。RSS原失败、bd2未复现、Mac依赖及全部其余门禁保持；DPI专项取消，TASK revision2/in_progress/pending不变，不关闭或发布。
"""
document.write_bytes(old + b"\n\n" + addition.rstrip().encode("utf-8") + b"\n")
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import build_receipt, validate_object
task = json.loads((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_text(encoding="utf-8"))
rid = "RCP-OPENCODEX-DESKTOP-182-WIN-HOST-MSI-PREP-20261016"
rpath = f".adg/work/objects/rcp/{rid}.json"
receipt = build_receipt(task, receipt_id=rid, result="succeeded", changed_paths=[impact_path, rpath],
    checks=[
        {"name": "local:windows-host-msi-plan-identity", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/verification.json"},
        {"name": "local:windows-host-msi-ordinary-token-refusal", "status": "passed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/ordinary-execution-refusal.original.json"},
    ], acceptance_results=[],
    fact_impacts=[{"target": "imp-20", "path": impact_path,
        "after_digest": hashlib.sha256(document.read_bytes()).hexdigest(),
        "summary": "本机MSI身份链与一次性执行计划准备完成，普通令牌拒绝已核验；实际安装及完整门禁未执行。"}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary="仅本机MSI准备/普通令牌拒绝通过，未调用安装器或启动产品；真实管理员执行与TASK完整门禁保持待验。")
validate_object(receipt)
write(PROJECT / rpath, receipt)
write(ROOT / "manifest.json", {"files": [
    {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
     "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
    for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"
], "frozen": True, "installerExecuted": False, "fullInstallerGatePassed": False})
print(json.dumps({"originalFiles": len(copied), "preparedRuntime": str(PREPARED),
                  "receipt": rpath, "scope": "preparation-only", "installerExecuted": False}))
