import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name)
    assert value["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
    assert value["applicationPid"] == 29936
    return value["result"]
def indexed(notifications):
    return {item["id"]: item for item in notifications["items"]}
prepared = native("native-prepared.json")
rotated = native("native-failure-and-other-candidate.json")
revisited = native("native-matching-success.json")
same_failed = native("native-same-candidate-failure.json")
same_succeeded = native("native-same-candidate-success.json")
assert rotated["rejected"] and same_failed["rejected"]
first_failure = indexed(rotated["afterFailure"]).keys() - indexed(prepared["before"]).keys()
assert len(first_failure) == 1
old_id = first_failure.pop()
assert not indexed(rotated["afterFailure"])[old_id]["resolved"]
assert not indexed(rotated["afterOtherCandidate"])[old_id]["resolved"]
assert not indexed(revisited["notifications"])[old_id]["resolved"]
new_failure = indexed(same_failed["afterFailure"]).keys() - indexed(same_failed["before"]).keys()
assert len(new_failure) == 1
new_id = new_failure.pop()
assert old_id != new_id
assert not indexed(same_failed["afterFailure"])[new_id]["resolved"]
assert indexed(same_succeeded["notifications"])[new_id]["resolved"]
assert indexed(same_succeeded["notifications"])[new_id]["resolvedAt"]
assert not indexed(same_succeeded["notifications"])[old_id]["resolved"]
for original_id, original in indexed(prepared["before"]).items():
    assert indexed(same_succeeded["notifications"])[original_id]["resolved"] == original["resolved"]
backups = {item["id"]: item for item in same_succeeded["backups"]}
assert backups["bk_20261012044452_a921f503"]["pinned"]
for result in [rotated["otherRestore"], revisited["matchingRestore"], same_succeeded["matchingRestore"]]:
    assert not result["refreshRequired"] and not result["protectionReconciliationPending"]
    assert not backups[result["protectionBackupId"]]["protected"]
assert same_succeeded["policy"]["mode"] == "manual"
assert same_succeeded["preferences"] == {"channel": "stable", "automaticCheck": False, "theme": "system"}
assert read("filesystem-verification.json")["activePreferencesBytesUnchanged"]
assert read("filesystem-verification.json")["originalPayloadRestored"]
assert read("same-candidate-filesystem-verification.json")["activeBytesUnchanged"]
assert read("same-candidate-filesystem-verification.json")["originalPayloadRestored"]
assert read("event-scope-after-same-failure.json") == read("event-scope-after-same-success.json")
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "real restore failure, other candidate and rotation isolation, continuous same-candidate recovery",
    "rotatedFailureNotification": old_id, "matchingFailureNotification": new_id,
    "fullEventGateAccepted": False, "fullNativeGateAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
