"""Refresh validation after this exact CI finishes; keep the draft unchanged."""
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
assert not (ROOT / "pr-final-before.original.json").exists()
ci = json.loads((ROOT / "pr-ci-collection.json").read_text(encoding="utf-8"))
assert ci["wholeCiSucceeded"] and ci["source"] == "823a8a0615153068594daef530c2e80f1d95fc29"
body = (ROOT / "pr-description.md").read_text(encoding="utf-8").strip()
old, = [paragraph for paragraph in body.split("\n\n") if paragraph.startswith("验证：")]
new = (
    "验证：823a8a06 本机40项真实受控进程、变更Rust格式/生产及测试元数据、PowerShell语法通过。"
    "同源首次PR CI 38838969102 整轮success：前端99文件/535项、Mac35组923passed/6ignored、"
    "Windows35组650passed/7ignored，最终EXE启动冒烟成功；五个必需job成功，main-only build按配置skip。"
    "原始日志实际checkout 7633c3ba1ae1ed5d4a210ed9d3316af54f6893d8，完整tree "
    "4e1965dea1938394c47c4f6eebff79cbe7c9e2d1 与head相同。40项Windows交接作业及API制品摘要/CRC另行核验。"
    "伴随push 38838964154因phf_generator 0.13.1依赖下载连接中断/HTTP2失败维持failure，"
    "Clippy未完成、后端回归和最终EXE跳过，原件保留且未重跑。新候选须绑定完整CI并独立验证，"
    "尚无823原生恢复或性能预算通过结论。"
)
body = body.replace(old, new, 1)
(ROOT / "pr-description-final.md").write_text(body + "\n", encoding="utf-8")
spec = importlib.util.spec_from_file_location(
    "handoff_final_pr_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["head"]["sha"] == ci["source"] and before["draft"] and before["state"] == "open"
assert not before["merged"] and before["auto_merge"] is None
(ROOT / "pr-final-before.original.json").write_text(json.dumps(before, indent=2) + "\n", encoding="utf-8")
try:
    _, response = api.request("pulls/5", token, payload={"body": body}, method="PATCH")
    (ROOT / "pr-final-patch.original.json").write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
except Exception as error:
    (ROOT / "pr-final-patch-error.json").write_text(json.dumps(
        {"errorType": type(error).__name__, "automaticMutationRetry": False}) + "\n", encoding="utf-8")
_, after = api.request("pulls/5", token)
(ROOT / "pr-final-after.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
assert after["body"] == body and after["head"]["sha"] == ci["source"]
assert after["draft"] and not after["merged"] and after["auto_merge"] is None
print(json.dumps({"draftPullRequest": 5, "source": ci["source"],
                  "fullCiValidationReadbackMatched": True, "merged": False, "releaseAccepted": False}))
