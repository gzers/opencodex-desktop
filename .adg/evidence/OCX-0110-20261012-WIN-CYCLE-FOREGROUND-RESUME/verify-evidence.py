import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
native = read("native-resume-observation.json")
assert native["applicationPid"] == 38924
assert native["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert native["result"]["foreground"] and native["result"]["effects"] == {"foreground": True, "visible": True}
assert native["result"]["preferences"] == {"channel": "stable", "automaticCheck": True}
baseline = json.loads((root.parent / "OCX-0110-20261012-WIN-REAL-STABLE-PANEL-CYCLE/schedule-baseline.json")
                      .read_text(encoding="utf-8-sig"))
assert read("schedule-after-resume.json") == baseline
assert native["result"]["manager"]["lastCheckedAt"] == "2026-10-12T10:01:07.936639900+00:00"
assert native["result"]["trace"]["events"] == []
window = read("native-window-after-resume.json")
assert window["applicationPid"] == 38924 and window["hwnd"] == 461032
assert window["foreground"] and window["backdropAttribute"] == 2 and window["dpi"] == 192
action = read("native-input.json")
assert not action["newApplicationStarted"] and not action["manualCheckInvoked"] and not action["timersOrClocksModified"]
assert (root / "same-cycle-window.jpg").read_bytes().startswith(b"\xff\xd8\xff")
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
    "result": "passed", "scope": "actual foreground restore of original real-cycle window, unchanged recorded deadlines and metadata",
    "rawAutomaticRequestCounts": "not instrumented", "realCycleAccepted": False,
    "finalMainNativeAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "realCycle": "pending", "files": len(entries)}))
