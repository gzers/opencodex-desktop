import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
failure, success = [read(name) for name in [
    "native-failure-and-unrelated-query.json", "native-real-success-recovery.json",
]]
for native in [failure, success]:
    assert native["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
    assert native["applicationPid"] == 29936
f, s = failure["result"], success["result"]
assert f["failureState"]["error"] and not f["failureState"]["loading"]
assert f["failureState"]["cache"] == f["remoteBefore"] == f["failureState"]["remote"]
assert f["unrelatedManagerQuery"]["status"] == "up_to_date"
assert f["before"] == f["afterFailure"] == f["afterUnrelatedManagerQuery"]
assert s["notifications"] == f["before"]
assert not s["error"] and not s["loading"]
assert s["cache"] == s["remote"] == f["remoteBefore"]
assert s["preferences"] == {"channel": "stable", "automaticCheck": False}
before, failed, after = [read(name)["entries"]["panel-latest"] for name in [
    "schedule-before.json", "schedule-after-failure.json", "schedule-after-success.json",
]]
assert failed["failures"] == 1 and failed["last_success"] == before["last_success"]
assert failed["next_due"] < before["next_due"]
assert after["failures"] == 0 and after["last_success"] > before["last_success"]
assert after["next_due"] - after["last_success"] == 86400
assert all(value["policy"] is None and value["resolves"] == []
           for value in read("registry-policy-witness.json")["policies"])
cleanup = read("cleanup-verification.json")
assert cleanup["temporaryNpmrcRemoved"] and cleanup["preferenceBytesUnchanged"]
assert not cleanup["packageLifecycleScriptsExecuted"]
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
    "result": "passed", "scope": "actual npm query failure, last-good cache, failed deadline and real manual retry",
    "persistentNotificationRecovery": "not applicable to declared policy; no persistent failure row",
    "fullEventGateAccepted": False, "fullNativeGateAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
