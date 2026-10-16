"""Update only the existing draft's reviewable description, then read it back."""
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
spec = importlib.util.spec_from_file_location(
    "handoff_pr_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["head"]["sha"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert before["draft"] and before["state"] == "open" and not before["merged"] and before["auto_merge"] is None
assert not (ROOT / "pr-before.original.json").exists()
(ROOT / "pr-before.original.json").write_text(json.dumps(before, indent=2) + "\n", encoding="utf-8")
payload = {"title": "fix(windows): 完善启动根恢复并减少光场重复分配",
           "body": (ROOT / "pr-description.md").read_text(encoding="utf-8").strip()}
try:
    _, response = api.request("pulls/5", token, payload=payload, method="PATCH")
    (ROOT / "pr-patch-response.original.json").write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
except Exception as error:
    (ROOT / "pr-patch-error.json").write_text(json.dumps(
        {"errorType": type(error).__name__, "mutationAutomaticallyRetried": False}) + "\n", encoding="utf-8")
_, after = api.request("pulls/5", token)
(ROOT / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n", encoding="utf-8")
assert after["title"] == payload["title"] and after["body"] == payload["body"]
assert after["draft"] and not after["merged"] and after["auto_merge"] is None
assert after["head"]["sha"] == before["head"]["sha"]
print(json.dumps({"updatedDraft": 5, "head": after["head"]["sha"], "readbackMatched": True,
                  "merged": False, "autoMergeEnabled": False}))
