"""Verify draft-only PR identity and corrected post-interruption description."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
base = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
for filename in ["draft-pr-created.json", "draft-pr-current.json", "draft-pr-updated-after-interruption.json"]:
    pr = read(filename)
    assert pr["number"] == 1 and pr["head"]["sha"] == source and pr["base"]["sha"] == base
    assert pr["head"]["ref"] == "feature/0.1.10-update-center-focus" and pr["base"]["ref"] == "main"
    assert pr["draft"] and pr["state"] == "open" and not pr["merged"] and not pr["auto_merge"]
    assert pr["title"] == "fix(updates): 切换详情视图后保留弹窗焦点（待Windows原生复验）"
original_body = (root / "pr-body.md").read_text(encoding="utf-8")
updated_body = (root / "pr-body-after-interruption.md").read_text(encoding="utf-8")
assert read("draft-pr-created.json")["body"] == original_body
assert read("draft-pr-updated-after-interruption.json")["body"] == updated_body
assert "201份" in updated_body and "22:55:39/40" in updated_body
assert "退出时间和原因未知" in updated_body and "不改截止、手动查询或拼接两段进程计时" in updated_body
preflight = read("creation-preflight.json")
assert preflight["existingPrCount"] == 0 and preflight["draft"] and not preflight["mergeRequested"]
assert "403 Resource not accessible by integration" in preflight["connectorAttemptResult"]
assert not preflight["credentialStoredOrPrinted"]
files_changed = read("draft-pr-files.json")
assert len(files_changed) == 5 and all(item["filename"].startswith(("apps/", ".github/")) for item in files_changed)
snapshot = read("pr-readback.json")
assert snapshot["changedFiles"] == 5 and snapshot["draft"] and not snapshot["merged"]
ci = {item["id"]: item for item in snapshot["sourceCiRuns"]}
assert set(ci) == {38361436746, 38352525092}
assert ci[38361436746]["event"] == "pull_request" and ci[38352525092]["event"] == "push"
assert all(item["head_sha"] == source and item["status"] == "completed" and item["conclusion"] == "success" for item in ci.values())
for key in ["mergeOrReadyForReviewRequested", "releaseRequested", "foregroundInputIssued",
            "originalDesktopOrCycleChanged", "localCandidateStarted", "nativeFocusFixAccepted",
            "fullNativeAccepted", "originalCiFailureCleared", "originalCiCauseIsolated", "formalTaskReceipt"]:
    assert snapshot[key] is False
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "sourceCommit": source, "baseCommit": base,
    "scope": "created and attached unmerged draft PR, five code files, push/PR CI readback and updated interruption scope",
    "draftPrNumber": 1, "draftPrState": "open", "mainIntegrated": False,
    "mergeRequested": False, "candidateNativeAccepted": False,
    "originalCiFailureCleared": False, "originalCiCauseIsolated": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "draftPr": 1,
                  "merged": False, "nativeFixAccepted": False}))
