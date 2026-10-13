"""Native cached-details/geometry subset, with focus finding and titlebar limits."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name + ".json")
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image
    return value["result"]
def visible_dialog(value):
    matches = [item for item in value["dialogs"] if item["rectangle"]["width"] > 0]
    assert len(matches) == 1
    return matches[0]
manager = native("02-manager-shift-tab")
wrapped = native("03-manager-tab-wrap")
closed = native("04-manager-closed")
panel = native("05-panel-details")
all_objects = native("06-all-update-objects")
end = native("11-end-readback")
for value in [manager, wrapped, closed, panel, all_objects, end]:
    assert value["foreground"] and value["dpr"] == 2
    assert value["uiPreferences"] == {"channel": "stable", "automaticCheck": True, "theme": "system", "error": False}
    assert value["busy"] == {"manager": False, "panel": False}
    assert value["manager"] == manager["manager"] and value["panelCache"] == manager["panelCache"]
assert manager["manager"]["currentVersion"] == "0.1.10" and manager["manager"]["availableVersion"] is None
assert manager["manager"]["lastCheckedAt"] == "2026-10-12T14:55:39.240251800+00:00"
assert manager["panelCache"]["version"] == "2.83.0"
assert manager["activeElement"]["label"] == "重新检查更新" and manager["activeElement"]["inUpdateDialog"]
assert wrapped["activeElement"]["label"] == "关闭更新中心" and wrapped["activeElement"]["inUpdateDialog"]
assert closed["activeElement"]["label"] == "查看管理器更新" and not closed["activeElement"]["inUpdateDialog"]
assert not any(item["rectangle"]["width"] > 0 for item in closed["dialogs"])
manager_dialog = visible_dialog(manager)
panel_dialog = visible_dialog(panel)
all_dialog = visible_dialog(all_objects)
assert manager_dialog["title"] == "桌面管理器更新" and manager_dialog["modal"] == "true"
assert panel_dialog["title"] == "OpenCodex 面板更新" and panel_dialog["modal"] == "true"
assert all_dialog["title"] == "更新中心" and all_dialog["modal"] == "true"
assert manager_dialog["checkboxes"] == [{"label": "更新前备份配置（推荐）", "checked": True, "disabled": False}]
assert manager_dialog["results"] == ["已是最新版本"]
assert manager_dialog["versions"] == ["当前 0.1.10→可用 —"]
assert panel_dialog["results"] == ["检查失败"] and panel_dialog["versions"] == ["当前 未知→可用 2.83.0"]
assert all_dialog["versions"] == manager_dialog["versions"] + panel_dialog["versions"]
assert next(item for item in manager_dialog["buttons"] if item["label"] == "安装管理器更新")["disabled"]
assert next(item for item in panel_dialog["buttons"] if item["label"] == "备份并更新面板")["disabled"]
for dialog in [manager_dialog, panel_dialog, all_dialog]:
    assert dialog["progress"] == []
    assert dialog["header"]["bottom"] <= dialog["body"]["top"]
    assert dialog["body"]["bottom"] <= dialog["footer"]["top"]
    for note in dialog["notes"]:
        assert note["scrollHeight"] == note["clientHeight"] and note["overflowY"] == "auto"
        assert note["tabIndex"] == 0
assert all_objects["activeElement"]["tag"] == "BODY" and not all_objects["activeElement"]["inUpdateDialog"]
assert not all_dialog["activeFocusInside"], "Actual unresolved native focus finding must remain"
tree = native("10-current-tree-alignment")
assert tree["rowCount"] == len(tree["rows"]) == 39 and tree["checkControlCount"] == 0
assert tree["originalPinRetained"] and tree["dpr"] == 2 and tree["foreground"]
assert not tree["programmaticFocusOrRouteOrScrollPerformed"]
for row in tree["rows"]:
    assert row["purposeFirstLineOffset"] == 0
    for control in row["controls"]:
        assert control["firstLineOffset"] == 0
        if "btn" in control["className"]:
            assert control["height"] >= 34
maximized = read("07-native-maximized.json")
minimized = read("08-after-restore-capture-error.json")
recovered = read("10-native-recovered.json")
for metrics in [maximized, minimized, recovered]:
    assert metrics["sourceCommit"] == source and metrics["applicationPid"] == 37276
    assert metrics["hwnd"] == 1051698 and metrics["dpi"] == 192
assert maximized["maximized"] and maximized["foreground"] and maximized["backdropAttribute"] == 2
assert minimized["minimized"] and not minimized["maximized"] and not minimized["foreground"]
assert not recovered["minimized"] and not recovered["maximized"] and recovered["foreground"] and recovered["backdropAttribute"] == 2
small = read("07-native-maximized.metadata.json")
assert small["width"] == small["height"] == 13
for name in ["00-backup-initial", "01-upgrade-page", "02-manager-details", "05-panel-details",
             "06-all-update-objects", "09-window-recovered", "10-current-backup-tree"]:
    metadata = read(name + ".metadata.json")
    assert metadata["window"]["id"] == 1051698
    assert metadata["width"] >= 1180 and metadata["height"] >= 790
    assert (root / metadata["file"]).read_bytes().startswith(b"\xff\xd8\xff")
sequence = read("computer-use-sequence.json")
assert sequence["sourceCommit"] == source and sequence["applicationPid"] == 37276 and sequence["windowId"] == 1051698
assert sequence["nativeInputsOnlyViaSky"] and sequence["domReadsOnly"] and sequence["noMoreUiInputThisRound"]
assert not sequence["manualCheckInstallRestartOrSourceSwitch"] and not sequence["restoreTitlebarStepAccepted"] and not sequence["fullGateAccepted"]
assert any(item.get("captureError", "").startswith("window is minimized") for item in sequence["actions"])
assert all(datetime.fromisoformat(item["atUtc"].replace("Z", "+00:00")).timestamp() <
           datetime.fromisoformat("2026-10-13T12:50:00+08:00").timestamp() for item in sequence["actions"])
before, after = read("scene-before.json"), read("scene-after.json")
assert before["sourceCommit"] == after["sourceCommit"] == source
assert before["hashes"] == after["hashes"] and after["originalBytesAllPreserved"]
assert before["hashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert (root / "schedule-before.json").read_bytes() == (root / "schedule-after.json").read_bytes()
assert not any(after[key] for key in ["manualCheckOrInstallationInvoked", "productOrCycleRestarted",
    "after1250FurtherForegroundInputs", "physicalDpiMatrixAccepted", "fullUpdateProgressAccepted", "fullNativeGateAccepted", "releaseAccepted"])
scope = read("scope-and-limitations.json")
assert scope["sourceCommit"] == source and scope["applicationPid"] == 37276
assert not scope["notes"]["longNotesInternalScrollingAccepted"] and not scope["notes"]["actualDownloadInstallProgressCancelOrRestartAccepted"]
assert not scope["findings"][0]["fullModalFocusGateAccepted"] and not scope["findings"][1]["restoreAccepted"]
assert scope["tree"]["rows"] == 39 and not scope["tree"]["fullCheckboxThemeDensityAdminDpiMatrixAccepted"]
assert not scope["pairedPerformanceAccepted"] and not scope["realCycleAccepted"] and not scope["fullNativeGateAccepted"]
life = read("retained-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
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
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "ordinary DPI192 real cached update details and manager key focus,39-row native tree geometry, verified maximize/recovery with unresolved all-view focus and Restore limitations",
    "allViewNativeFocusFindingUnresolved": True, "titlebarRestoreAccepted": False,
    "longNotesOrRealProgressAccepted": False, "fullTreeDpiAdminMatrixAccepted": False,
    "rawNetworkOrQueryCountsAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "cachedDetails": "observed",
                  "actualTreeRows": 39, "focusFindingPreserved": True, "fullGateAccepted": False}))
