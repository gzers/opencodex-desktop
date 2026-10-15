"""Update the existing draft description once; independently read it back."""
import importlib.util
import json
from pathlib import Path

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["state"] == "open" and before["draft"] and not before["merged"]
assert before["auto_merge"] is None and before["head"]["sha"] == api.SOURCE
title = "fix(windows): 显式恢复不可写启动根并保留首次回归取证"
body = """Windows 新安装的数据根不可写时，原启动流程在业务服务初始化前失败，设置里的切根入口无法使用。本补丁延迟创建主界面，仅对未配置的新鲜默认根的真实写权限拒绝提供系统目录选择；确认后锁内验证并保存当前用户的启动指针，重启统一解析。取消退出；损坏、过新、锁冲突、未决迁移不自动回退，已选根丢失或损坏也不重建。显式测试沙箱绕过个人选择。

包含真实启动/根锁集成测试，以及取消、保存失败、非法目录、链接和旧数据保留边界。新增权限错误映射到既有访问拒绝退出码。沿用固定 bd2 失败制品、官方 Node 24.21.0 和首次 CI ETW 工具；当次新构建与历史回放分别记录身份，原并发、2/4/5/8 秒和采集预算保持。

首轮 6ab09772 的三处错误枚举漏配导致编译失败，原件保留；bc14165c 已补齐。最终验证状态见本 PR 的同提交检查和 docs/governance-main 的“20261015-Windows启动根显式恢复修复”证据。普通权限拒绝写、原生选根/重启及普通/管理员同根仍待匹配候选真机验收。

此前固定 431 候选在独立 MSI/NSIS runner 的官方 0.1.9 升级、默认卸载及重装 seeded 文件保留子集通过；该结果不证明新代码的应用根绑定或完整安装门禁。历史 bd2/e6 查询失败尚无实际失败窗口 ETW/根因，Mac CPU/RSS 失败与其他原门禁仍开放。

保持草稿，不自动合并、关闭、签名、晋升或发布；本机日常安装与真实数据不变。
"""
payload = {"title": title, "body": body}
(root / "pr-update-payload.json").write_text(
    json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
attempt = root / "pr-update-attempt.json"
assert not attempt.exists(), "Inspect remote state instead of retrying a possible mutation"
attempt.write_text(json.dumps({"number": 5, "source": api.SOURCE, "mutationAttempts": 1}) + "\n")
api.request("pulls/5", token, payload=payload, method="PATCH")
_, after = api.request("pulls/5", token)
assert after["title"] == title and after["body"] == body
assert after["state"] == "open" and after["draft"] and not after["merged"] and after["auto_merge"] is None
assert after["head"]["sha"] == api.SOURCE
(root / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"number": 5, "source": api.SOURCE, "descriptionReadbackMatched": True,
                  "draft": after["draft"], "merged": after["merged"], "autoMerge": after["auto_merge"]}))
