"""Update the existing draft to describe the current anchor behavior."""
import importlib.util
import json
from pathlib import Path

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["head"]["sha"] == api.SOURCE and before["state"] == "open" and before["draft"]
assert not before["merged"] and before["auto_merge"] is None
payload = {
    "title": "fix(windows): 持久登记启动根并显式恢复不可写的新安装",
    "body": """Windows 首次使用可写的安装目录 data 时，原流程没有持久登记启动根，后续换安装位置可能选择新的空目录。现在先持有真实根锁完成准入，再登记当前用户启动指针，业务服务随后启动。后续保留原 anchor、自定义活动根引用和外置 HOME；已登记指针不重复写入。登记失败停止，写入结果不确定则下次启动回读核对，不自动复位。

同一流程在新鲜默认根的真实写权限拒绝时提供系统目录选择，确认后锁内验证、持久保存并重启；取消退出。损坏、过新、锁冲突、未决迁移及无法核查的既有根不自动回退，已登记根消失或损坏不重建。显式测试沙箱绕过个人指针。

补充换安装位置、旧根/外置 HOME 保留、根锁与登记顺序、失败释放及不确定提交回读测试。测试中的指针使用临时文件/闭包，不能替代真实 HKCU、普通/管理员令牌或安装器验收。当前生产提交为 02a0d4a8，验证状态见该提交的 CI 检查和 docs/governance-main 证据；旧 a2 候选只证明其自身，不冒充本提交。

保留 bd2 原失败制品/官方 Node 24.21.0、首次 CI ETW 与原并发及 2/4/5/8 秒、120 秒和采集预算。历次枚举、跳转夹具、审计外层超时与 Clippy 失败原件均保留；查询失败窗口及根因尚未取得，Mac CPU/RSS 失败与其他原门禁仍开放。审计夹具批量准备不改变生产逐条持久化或原断言。

保持草稿，不自动合并、关闭、签名、晋升或发布；没有启动本机产品、写本机个人指针、创建账户/VM或替换日常安装。
""",
}
attempt = root / "pr-update-attempt.json"
assert not attempt.exists(), "Read back a possible mutation instead of repeating it"
(root / "pr-update-payload.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
                                            encoding="utf-8")
attempt.write_text(json.dumps({"source": api.SOURCE, "number": 5, "attempts": 1}) + "\n", encoding="utf-8")
api.request("pulls/5", token, payload=payload, method="PATCH")
_, after = api.request("pulls/5", token)
assert after["body"] == payload["body"] and after["title"] == payload["title"]
assert after["head"]["sha"] == api.SOURCE and after["state"] == "open" and after["draft"]
assert not after["merged"] and after["auto_merge"] is None
(root / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"number": 5, "source": api.SOURCE, "readbackMatched": True, "draft": True,
                  "merged": False, "autoMerge": None}))
