"""Real final ordinary desktop keyboard/refresh geometry subset, not full tree matrix."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
    return value["result"]
before = native("before-tree-input.json")
assert before["rowCount"] == 3 and not before["activeElement"]["inTree"]
scenes = [
    ("01-month-expanded.json", "收起 10", 5),
    ("02-native-right.json", "展开 manual-preferences", 5),
    ("03-native-enter.json", "收起 manual-preferences", 22),
    ("04-native-right-backup.json", "展开 bk_20261012044452_a921f503", 22),
    ("05-native-space.json", "收起 bk_20261012044452_a921f503", 24),
    ("06-native-right-file.json", "backup-manifest.json", 24),
    ("07-native-down-file.json", "preferences.json", 24),
    ("08-native-left-parent.json", "收起 bk_20261012044452_a921f503", 24),
    ("09-native-home-root.json", "收起 backups", 24),
]
for name, label, count in scenes:
    value = native(name)
    assert value["route"] == "#settings?section=backup"
    assert value["foreground"] and value["dpr"] == 2
    assert value["activeElement"]["inTree"] and value["activeElement"]["label"] == label
    assert value["rowCount"] == count
expanded = [row["label"] for row in native("05-native-space.json")["rows"] if row["expanded"] == "true"]
assert expanded == ["backups", "2026", "10", "manual-preferences", "bk_20261012044452_a921f503"]
armed = native("refresh-observer-armed.json")
assert armed["status"] == "armed" and armed["listenerBoundToActualRefresh"]
refresh = native("actual-refresh-result.json")
assert refresh["clicked"] and refresh["status"] == "finished" and refresh["sawBusy"]
assert refresh["focusAfter"].strip() == "刷新" and refresh["foreground"]
assert refresh["priorExpanded"] == refresh["afterExpanded"] == ["收起 " + label for label in expanded]
assert [value["busy"] for value in refresh["transitions"]] == [True, False]
assert refresh["transitions"][0]["at"] < refresh["transitions"][1]["at"]
geometry = native("actual-tree-geometry.json")
assert geometry["rowCount"] == 24 and geometry["foreground"] and geometry["dpr"] == 2
assert geometry["checkControlCount"] == 0 and geometry["originalPinRetained"]
assert not geometry["programmaticFocusOrRouteOrScrollPerformed"]
for row in geometry["rows"]:
    assert abs(row["purposeFirstLineOffset"]) <= 1
    for control in row["controls"]:
        assert abs(control["firstLineOffset"]) <= 1
        if "btn" in control["className"]:
            assert control["height"] >= 34
metrics = read("actual-tree-native-metrics.json")
assert metrics["sourceCommit"] == source and metrics["applicationPid"] == 37276
assert metrics["hwnd"] == 1051698 and metrics["dpi"] == 192
assert metrics["foreground"] and metrics["backdropAttribute"] == 2
action = read("computer-use-actions-and-limitations.json")
assert action["source"] == source and action["window"]["id"] == 1051698
assert action["nativeInputOnlyViaSky"] and action["domReadOnly"]
assert not action["programmaticRoutingScrollingFocusOrKeyboard"]
assert not action["afterDisconnectFurtherUiInputs"]
assert not action["updateDetailsUiAcceptedThisTurn"] and not action["checkboxMatrixAccepted"]
assert "0x80070057" in action["failedFinalCapture"]
assert (root / "native-tree-last-successful-frame.jpg").read_bytes().startswith(b"\xff\xd8\xff")
after = read("scene-after-filesystem-and-lifecycle.json")
initial = read("scene-before.json")
assert after["sourceCommit"] == source and after["applicationPid"] == 37276
assert after["beforeHashes"] == after["afterHashes"] == initial["beforeHashes"]
assert after["preferencesScheduleBindingAllBytesUnchanged"]
assert after["originalApplicationAndMonitorLiveHandlesVerified"] and after["desktopApplicationCount"] == 1
assert after["monitorPid"] == 3920 and after["monitorExecSession"] == 46484
assert after["applicationToken"]["elevated"] is False and after["applicationToken"]["elevationType"] == 3
assert after["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(after[key] for key in [
    "manualCheckOrInstallationInvoked", "updateDetailsUiCompleted", "realCycleAccepted",
    "completeTreeCheckboxOrDpiMatrixAccepted", "fullGateAccepted", "releaseAccepted",
])
post = native("post-disconnection-tree-read.json")
assert post["rowCount"] == 24 and post["route"] == "#settings?section=backup"
assert [row["label"] for row in post["rows"] if row["expanded"] == "true"] == expanded
# Keep method provenance and prohibit using a route/focus-mutating geometry helper.
old_methods = root.parent / "OCX-0110-20261012-WIN-TREE-KEYBOARD/methods"
for name in [
    "ocx-0110-tree-keyboard-read.js", "ocx-0110-tree-native-refresh-observe.js",
    "ocx-0110-tree-native-refresh-result.js",
]:
    assert (root / "methods" / name).read_bytes() == (old_methods / name).read_bytes()
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip"} for path in files)
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
    "result": "passed", "sourceCommit": source,
    "scope": "real final ordinary DPI192 desktop BackupFiles keyboard, asynchronous refresh/focus and24 production-row first-line geometry",
    "capturedFinalFrame": "last successful pre-disconnection frame; not asserted as stable post-disconnect frame",
    "fullTreeCheckboxOrDpiMatrixAccepted": False, "updateDetailsUiAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "actualRows": 24, "nativeKeyboardAndRefresh": "passed",
                  "files": len(entries)}))
