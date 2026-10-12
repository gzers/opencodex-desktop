import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
native = read("native-query-channel-result.json")
assert native["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert native["applicationPid"] == 29936
result = native["result"]
assert "observationError" not in result
assert result["queryStillPendingBeforeSwitch"] and result["queryStillPendingAfterSwitch"]
assert result["queryStartedAtMs"] < result["switchStartedAtMs"] < result["switchCompletedAtMs"] < result["queryCompletedAtMs"]
snapshot = result["snapshots"][0]
assert snapshot["loading"] and snapshot["nativePreferences"] == {"channel": "beta", "automaticCheck": False}
assert snapshot["nativeCache"] == result["cacheBefore"] == result["cacheAfterQuery"]
assert result["queryFinalState"]["error"] is False and result["queryFinalState"]["loading"] is False
assert result["nativeManagerAfterQuery"]["channel"] == "beta"
assert all(result["cleanup"][key] for key in [
    "originalChannelRestored", "automaticCheckStillDisabled", "nativeInvokeUnchanged",
])
assert result["cleanup"]["preferencesStoreChannel"] == "stable"
assert read("filesystem-verification.json")["originalPreferencesBytesRestored"]
before, after = [read(name)["entries"] for name in ["schedule-before.json", "schedule-after.json"]]
assert before["manager-stable"] == after["manager-stable"]
assert after["panel-latest"]["failures"] == 0
assert after["panel-latest"]["last_success"] > before["panel-latest"]["last_success"]
assert after["panel-latest"]["next_due"] - after["panel-latest"]["last_success"] == 86400
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
    "result": "passed", "scope": "actual panel query in flight across one stable-to-beta native switch",
    "queryElapsedMs": result["queryCompletedAtMs"] - result["queryStartedAtMs"],
    "fullChannelMatrixAccepted": False, "fullNativeGateAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
