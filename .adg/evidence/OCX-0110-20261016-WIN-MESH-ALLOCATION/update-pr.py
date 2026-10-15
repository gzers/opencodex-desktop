"""Update the already-authorized draft through the configured local Git identity."""
import importlib.util
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"
spec = importlib.util.spec_from_file_location(
    "bounded_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["draft"] and before["state"] == "open"
assert before["head"]["sha"] == SOURCE and before["merged_at"] is None
payload = {
    "title": "fix(windows): 持久登记启动根并减少光场重复分配",
    "body": """Windows 首次成功启动未持久登记安装旁的数据根，换安装位置后可能另选新空根；默认根不可写时也缺少业务服务初始化前的显式恢复入口。现在先持锁验证并登记当前用户的启动指针，再初始化 WebView 和业务服务；仅未配置的 Windows 默认根写权限拒绝可选择合法根或取消退出，损坏、过新、冲突及未完成迁移不会静默回退。

概览光场关闭未使用的 depth/stencil 缓冲，并等待 Teleport 将背景层放到最终容器后再创建画布，避免先按旧容器分配、随后再次扩容。Logo 仍即时创建，提前离开页面时不分配新缓冲。着色器、最终画布尺寸、分辨率、颜色和画质策略保持不变。

验证：当前 head 本地 UI 99 文件 / 535 项、类型检查和生产构建通过。新增两项回归在修复前失败，覆盖最终容器中首次分配及 DOM 更新前离开页面。后台 Edge 对照确认浅色/深色静态截图 RGBA 逐字节一致；最终画布均为 1496×1342，旧容器的中间尺寸分配已消除。原 depth/stencil 优化另有真实 Windows WebView 24 组着色器像素一致证据。当前 head 的 push/PR CI 沿首次执行跟踪，尚不宣称新制品性能通过。

既有证据在 docs/governance-main：02 精确候选已验证当前账户同 SID 普通/管理员令牌、换程序位置保留原根、不可写取消/合法根恢复、损坏/过新指针拒绝、固定备份与异值恢复及本地事件重试；原状态错误退避已记录真实 30 分钟下限。不同源码/EXE 的局部证据按实际身份和源码等价范围引用。

未决：02 候选平均 RSS 比正式 0.1.9 增加 38.328092 MiB；8be 精确候选补测增加 33.249280 MiB，均超过原 10 MiB 预算，保留失败，不跨轮推导优化收益。当前补丁的新制品与原预算复测待验。恢复交接短暂双 EXE、UI 保存根/HOME、宿主安装升级卸载保留、真实更新进度、树表/事件完整矩阵、冷/热各 20 次及 p95/主线程、真实周期仍有缺口；DPI 待验，历史 bd2 查询失败窗口 ETW 和 Mac CPU/RSS 失败保留。

保持草稿，TASK-182 / IMP-20 不关闭。本 PR 不构成合并、签名、晋升或发布许可。""",
}
payload_path = ROOT / "pr-payload.json"
assert not (ROOT / "pr-mutation-attempt.json").exists(), "Do not blindly repeat a mutation"
payload_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(ROOT / "pr-before.original.json").write_text(json.dumps(before, indent=2) + "\n", encoding="utf-8")
(ROOT / "connector-update-error.original.json").write_text(json.dumps({
    "tool": "GitHub connector update_pull_request", "httpStatus": 403,
    "message": "Resource not accessible by integration",
    "configuredLocalGitCredentialUsedAsFallback": True,
}, indent=2) + "\n", encoding="utf-8")
config = "\n".join([
    "url = " + json.dumps(api.BASE + "pulls/5"),
    "header = " + json.dumps("Authorization: Bearer " + token),
    'header = "Accept: application/vnd.github+json"',
    'header = "Content-Type: application/json"',
]) + "\n"
(ROOT / "pr-mutation-attempt.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "pr": 5,
    "scope": "draft title and body", "automaticMutationRetry": False,
}, indent=2) + "\n", encoding="utf-8")
response = subprocess.run([
    r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
    "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
    "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
    "--request", "PATCH", "--data-binary", "@" + str(payload_path),
    "--write-out", "\n%{http_code}",
], input=config.encode(), capture_output=True, timeout=35)
body, _, status = response.stdout.rpartition(b"\n")
assert response.returncode == 0 and status == b"200", "PR mutation uncertain; read remote before any retry"
(ROOT / "pr-update-response.original.json").write_bytes(body + b"\n")
_, after = api.request("pulls/5", token)
assert after["title"] == payload["title"] and after["body"] == payload["body"]
assert after["draft"] and after["merged_at"] is None and after["auto_merge"] is None
assert after["head"]["sha"] == SOURCE
(ROOT / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"pr": 5, "source": SOURCE, "draft": True, "merged": False,
                  "titleAndBodyReadbackMatched": True}))
