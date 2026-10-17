"""Offline replay of stored prototype QA only; no browser or native product."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
PROTOTYPE = REPO / "docs/04-项目资料/01-项目支撑资料/01-产品原型/原型"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

manifest = read(ROOT / "manifest.json")
assert {row["path"] for row in manifest["files"]} == {
    p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
for row in manifest["files"]:
    path = ROOT / row["path"]
    assert path.resolve().is_relative_to(ROOT.resolve()) and not path.is_symlink()
    assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"], row["path"]
before = read(ROOT / "remaining-before-visible/result.json")
after = read(ROOT / "remaining-after/result.json")
extras = read(ROOT / "remaining-extras/result.json")
assert len(before["cases"]) == len(after["cases"]) == 48
assert len(before["layoutFailures"]) == 12 and all(x["name"] == "backup" for x in before["layoutFailures"])
assert all(len(x["mismatches"]) == 7 and all(c["centerDelta"] == 8 for c in x["mismatches"])
           for x in before["layoutFailures"])
assert after["result"] == extras["result"] == "passed"
assert not after["layoutFailures"] and not extras["layoutFailures"]
assert len(extras["cases"]) == 4
for phase in ("", "remaining-before", "remaining-before-visible", "remaining-after", "remaining-extras"):
    result = read(ROOT / phase / "result.json")
    assert not result["failure"] and result["sourceFilesUnchanged"]
    assert result["childExit"]["code"] == 0 and result["childExit"]["signal"] is None
    assert result["loopbackServerClosed"] and not result["blockedRequests"]
    assert not result["nativeProductStarted"] and not result["productionGatePassed"]
for result in (after, extras):
    for case in result["cases"]:
        assert round(case["frame"]["width"]) == case["size"]
        assert round(case["frame"]["height"]) == (600 if case["size"] == 900 else 760)
        for row in case["geometry"]:
            assert all(abs(control["centerDelta"]) <= 1 for control in row["controls"])
for result in (before, after):
    buttons = [control for case in result["cases"] if case["name"] == "backup"
               for row in case["geometry"] for control in row["controls"]
               if control["className"] == "btn ghost"]
    assert len(buttons) == 84 and all(button["height"] == 36 for button in buttons)
for row in read(ROOT / "remaining-extras/binding.json")["sourceFiles"]:
    path = PROTOTYPE / row["path"]
    assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"]
validation = read(ROOT / "validation.json")
assert validation["existingTests"]["passed"] == 17 and validation["existingTests"]["failed"] == 0
assert validation["protection"]["nativeCount"] == 0 and validation["protection"]["originalHashesUnchanged"]
print(json.dumps({"status": "verified-prototype-tree-evidence", "files": len(manifest["files"]),
                  "postfixCases": 52, "backupActionCenterDeltaBefore": 8,
                  "backupActionCenterDeltaAfter": 0, "backupActionHeight": 36,
                  "existingTestsPassed": 17, "nativeProductStarted": False,
                  "fullWindowsGatePassed": False}))
