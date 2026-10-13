"""Verify a title/body-only update; the PR stays an unmerged draft."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
before = read("draft-pr-before-native-writeback.json")
after = read("draft-pr-after-native.json")
assert before["number"] == after["number"] == 1
assert before["head"]["sha"] == after["head"]["sha"] == "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert before["base"]["sha"] == after["base"]["sha"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert before["draft"] and after["draft"] and before["state"] == after["state"] == "open"
assert not before["merged"] and not after["merged"] and not after["auto_merge"]
assert after["title"] == "fix(updates): 切换详情视图后保留弹窗焦点"
assert after["body"] == (root / "pr-body.md").read_text(encoding="utf-8")
assert "真机复验子集通过" in after["body"] and "caption还原按钮点击未改变" in after["body"]
receipt = read("pr-writeback-receipt.json")
assert receipt["updatedFields"] == ["title", "body"] and receipt["nativeFocusSubsetAccepted"]
assert receipt["nativeFocusSubsetEvidenceCommit"] == "5dee74bfb629f8de68fadc5a10f1b2e453289c4b"
assert not any(receipt[key] for key in ["merged", "captionRestoreClickAccepted", "mergeRequested",
                                     "releaseRequested", "sourceCommitChanged", "foregroundInputIssued",
                                     "fullNativeGateAccepted", "formalTaskReceipt"])
files = sorted(path for path in root.iterdir() if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
entries = [{"path": path.name, "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "PR1 title/body-only native focus subset writeback; exact head/base and draft state retained",
    "nativeFocusSubsetEvidenceReferenced": True, "mainIntegrated": False,
    "captionRestoreClickAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "pr": 1, "draft": True, "merged": False}))
