import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
before, moved, corner, resized, final = [read(name + ".json") for name in [
    "before-drag", "after-drag", "after-corner-drag", "after-top-edge-resize", "final-native",
]]
for value in [before, moved, corner, resized, final]:
    assert value["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
    assert value["applicationPid"] == 29936 and value["hwnd"] == 852848
    assert value["dpi"] == 192 and value["backdropAttribute"] == 2 and value["dwmResult"] == 0
b, m, c, r = [value["windowBounds"] for value in [before, moved, corner, resized]]
assert m["Left"] - b["Left"] == m["Right"] - b["Right"] == 100
assert m["Top"] - b["Top"] == m["Bottom"] - b["Bottom"] == 60
assert m == c
assert r["Top"] - m["Top"] == 60
assert all(r[key] == m[key] for key in ["Left", "Right", "Bottom"])
assert final["windowBounds"] == r
assert read("rejected-capture.json")["imageKeptOutOfPublicEvidence"]
assert not (root / "final-window-0.png").exists()
assert (root / "final-candidate-window.jpg").read_bytes().startswith(b"\xff\xd8\xff")
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
    "result": "passed", "scope": "ordinary DPI192 native caption move and top-edge resize only",
    "cornerDrag": "no observed resize", "outOfBoundsRestore": "tool rejected, not retried",
    "fullNativeGateAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
