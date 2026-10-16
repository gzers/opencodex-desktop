"""Update the existing draft body once; preserve RSS and original timeout failures."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
spec = importlib.util.spec_from_file_location(
    "pr_updater", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)


def write(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


assert not (ROOT / "pr-update-attempt.json").exists(), "Inspect the existing attempt rather than repeating it"
before = json.loads((ROOT / "pr-before.original.json").read_text(encoding="utf-8-sig"))
token = api.credential()
_, current = api.request("pulls/6", token)
write("pr-pre-update.original.json", current)
assert current["body"] == before["body"] and current["head"]["sha"] == before["head"]["sha"]
assert current["head"]["sha"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
assert current["draft"] and current["state"] == "open" and not current["merged"]
parts = current["body"].split("\n\n")
paragraph, = [p for p in parts if p.startswith("基线正常exit0后才启动候选。")]
replacement = (
    "基线正常exit0后才启动候选。候选计数结束后，关闭动作未赶上前台截止，被时间守卫拒绝；"
    "两次只读退出观察到期仍未退出的历史失败保持。用户随后正常退出，2026-10-16 22:42:01"
    "已在native0下恢复原配置并保留测量克隆；22:43:55读回原目录ID/创建时刻、偏好及HKCU指针"
    "摘要与测试前相同，已记录的14个进程代际全部结束。候选实际退出码和确切内核退出时刻仍未知，"
    "不补写exit0。完成计数仅用于原口径算术和角色归账，**不作为完整配对生命周期或完整性能门禁通过**。"
    "原始失败没有覆盖，未延长前台授权、重新启动产品或强杀。"
)
body = current["body"].replace(paragraph, replacement, 1)
assert body != current["body"] and "**+20.040155MiB仍超过原+10MiB，failed**" in body
body += (
    "\n\n配置恢复后续证据：docs/governance-main@3542068eba11e614fc71733b8fb554034d03e163"
    " 的 .adg/evidence/OCX-0110-20261016-WIN-B4-RESTORATION/。仅恢复与当前代际清理子集通过；"
    "严格产品恢复交接、RSS根因、bd2实际失败窗口、Mac依赖和其它原门禁仍未完成，保持草稿。"
)
write("pr-update-request.json", {"body": body})
write("pr-update-attempt.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(), "number": 6,
    "bodyOnly": True, "automaticMutationRetry": False,
    "mergeClosePublishRequested": False,
})
_, response = api.request("pulls/6", token, {"body": body}, method="PATCH")
write("pr-update-response.original.json", response)
_, final = api.request("pulls/6", token)
write("pr-final.original.json", final)
assert final["body"] == body and final["draft"] and final["state"] == "open" and not final["merged"]
assert final["head"]["sha"] == current["head"]["sha"] and final["base"]["ref"] == current["base"]["ref"]
print(json.dumps({"number": 6, "draft": True, "state": final["state"],
                  "restorationFollowupAdded": True, "originalRssFailurePreserved": True}))
