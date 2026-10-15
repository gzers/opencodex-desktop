"""Record the current implementation and retained failures in the existing draft."""
import importlib.util
import json
from pathlib import Path

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["head"]["sha"] == api.SOURCE and before["draft"] and before["state"] == "open"
assert not before["merged"] and before["auto_merge"] is None
body = """Windows 新安装的数据根不可写时，原启动流程在业务服务初始化前失败，设置里的切根入口无法使用。现在延迟创建业务界面，仅对未配置的新鲜默认根的真实写权限拒绝提供系统目录选择；锁内验证后持久保存当前用户的启动指针并重启。取消退出；损坏、过新、锁冲突和未决迁移不自动回退，已选根丢失或损坏也不重建。显式测试沙箱绕过个人选择。

覆盖真实启动准入、已选根优先、目录锁持有/释放、取消不提交、保存失败、非法目录及链接边界，补齐权限错误映射。Windows verbatim 路径测试使用保留 ParentDir 的原始输入；审计保留测试批量准备同样的 5004 行，再经生产路径持久追加末行，原有 12 个断言块与生产逐条 sync_all 保持，并新增清理前行数断言。

保留全部失败：6ab09772 枚举漏配、bc14165c 跳转夹具错误、c9f92ce7 的 PR 库超出原 120 秒外层上限、020e668c 的 Clippy 写法错误。c9 的 push 完整通过（Mac 918 / Windows 643，忽略 6/7），不拿它替代当前 a2e4ea17 的检查。当前源码及同提交检查以 PR head 为准。

沿用固定 bd2 原失败 EXE、官方 Node 24.21.0 与首次 CI ETW；历史回放和当前构建分开记录，原并发、2/4/5/8 秒、120 秒和采集预算不改。c9 PR 虽七项查询与 15 项启动根测试返回通过，仍因完整库未完成判失败；不把强制外层清理称作正常完成。

普通权限拒绝写、原生选根/重启、普通/管理员同根及新指针安装保留仍待匹配候选验收。旧 431 安装器 seeded 文件保留只按原范围复用。bd2/e6 实际查询失败窗口及根因、Mac CPU/RSS 失败和其余原门禁仍开放。

证据在 docs/governance-main 的“20261015-Windows启动根显式恢复修复”。保持草稿，不自动合并、关闭、签名、晋升或发布；未启动本机产品、替换日常版或迁移真实数据。
"""
payload = {"title": "fix(windows): 显式恢复不可写启动根并保留首次回归取证", "body": body}
attempt = root / "pr-final-update-attempt.json"
assert not attempt.exists(), "Inspect the existing mutation instead of automatically retrying"
(root / "pr-final-payload.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
                                           encoding="utf-8")
attempt.write_text(json.dumps({"source": api.SOURCE, "number": 5, "attempts": 1}) + "\n")
api.request("pulls/5", token, payload=payload, method="PATCH")
_, after = api.request("pulls/5", token)
assert after["body"] == body and after["title"] == payload["title"]
assert after["head"]["sha"] == api.SOURCE and after["draft"] and after["state"] == "open"
assert not after["merged"] and after["auto_merge"] is None
(root / "pr-final.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"number": 5, "head": api.SOURCE, "readbackMatched": True,
                  "draft": True, "merged": False, "autoMerge": None}))
