"""Verify native focus regression subset; retain the unproved caption-restore action."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

from PIL import Image

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 10904
    assert value["executableSha256"] == image
    return value["result"]
initial = native("01-manager-initial-focus.json")
first_tab = native("02-manager-after-tab-focus.json")
manager_back = native("03-manager-back-focused.json")
manager_all = native("04-manager-return-all-focus.json")
panel_heading = native("05-panel-detail-focused.json")
panel_back = native("06-panel-back-focused.json")
panel_all = native("07-panel-return-all-focus.json")
last_control = native("08-all-recheck-focused-only.json")
wrapped = native("09-all-tab-wrapped-close.json")
escaped = native("10-escape-opener-restored.json")
assert initial["dialogOpen"] and initial["title"] == "桌面管理器更新" and initial["focus"]["isDialog"]
assert first_tab["focus"]["ariaLabel"] == "关闭更新中心" and first_tab["focus"]["insideDialog"]
for value in [manager_back, panel_back]:
    assert value["focus"]["tag"] == "BUTTON" and value["focus"]["text"] == "← 全部更新"
    assert value["focus"]["insideDialog"] and not value["focus"]["isDialog"]
for value in [manager_all, panel_all]:
    assert value["dialogOpen"] and value["title"] == "更新中心"
    assert value["focus"]["insideDialog"] and value["focus"]["isDialog"] and value["focus"]["tag"] == "DIV"
assert panel_heading["title"] == "OpenCodex 面板更新" and panel_heading["focus"]["ariaLabel"] == "查看 OpenCodex 面板更新详情"
assert panel_heading["focus"]["insideDialog"]
assert last_control["focus"]["text"] == "重新检查更新" and last_control["focus"]["insideDialog"]
assert wrapped["focus"]["ariaLabel"] == "关闭更新中心" and wrapped["focus"]["insideDialog"]
assert not escaped["dialogOpen"] and escaped["focus"]["text"] == "查看管理器更新" and escaped["focus"]["tag"] == "BUTTON"
assert all(value["progressElements"] == 0 for value in [initial, manager_all, panel_heading, panel_all])
assert next(value for value in panel_heading["buttons"] if value["text"] == "备份并更新面板")["disabled"]
for filename, maximized in [
    ("native-before-titlebar.json", False), ("native-maximized.json", True),
    ("native-after-restore-click.json", True), ("native-after-system-restore.json", False),
]:
    value = read(filename)
    assert value["sourceCommit"] == source and value["executableSha256"] == image
    assert value["applicationPid"] == 10904 and value["hwnd"] == 6685560
    assert value["dpi"] == 192 and value["backdropAttribute"] == 2 and value["dwmResult"] == 0
    assert value["visible"] and not value["minimized"] and value["maximized"] is maximized and value["foreground"]
assert read("native-before-titlebar.json")["windowBounds"] == read("native-after-system-restore.json")["windowBounds"]
assert read("native-maximized.json")["windowBounds"] == read("native-after-restore-click.json")["windowBounds"]
actions = read("native-actions.json")
expected_actions = [
    "Tab manager dialog to first control",
    "Tab close control to manager back button",
    "Enter focused manager back button",
    "open panel detail from all view",
    "ShiftTab panel heading to back button",
    "Enter focused panel back button",
    "ShiftTab all-dialog to last recheck control without activating it",
    "Tab last recheck control wraps to close",
    "Escape closes modal and returns opener",
    "native caption maximize",
    "native caption restore",
    "open native system menu for restore verification",
    "Enter native system-menu selected Restore",
]
assert [value["action"] for value in actions["actions"]] == expected_actions
assert datetime.fromisoformat(actions["lastActualInputAtUtc"].replace("Z", "+00:00")) < \
    datetime.fromisoformat(actions["windowEndBeijing"])
assert actions["managerAndPanelReturnAllFocusObserved"] and actions["escapeOpenerObserved"]
assert not actions["captionRestoreClickAccepted"] and actions["systemMenuRestoreObserved"]
assert not actions["manualCheckOrInstallIssued"] and not actions["fullNativeGateAccepted"]
for name in ["preferences.json", "data-root.json", "backup-cleanup-policy.json", "update-schedule.json"]:
    assert (root / ("before-" + name)).read_bytes() == (root / ("after-" + name)).read_bytes()
schedule = read("after-update-schedule.json")
assert schedule["entries"]["manager-stable"] == {"next_due": 1791989903, "failures": 0, "last_success": 1791903503}
assert schedule["entries"]["panel-latest"] == {"next_due": 1791989906, "failures": 0, "last_success": 1791903506}
assert not read("after-preferences.json")["updates"]["app_update_auto_check"]
for name in ["session-before.json", "session-after.json"]:
    session = read(name)
    assert session["status"] == "running" and session["sourceCommit"] == source and session["executableSha256"] == image
    assert session["applicationPid"] == 10904 and session["supervisorPid"] == 32492
    assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
    assert session["applicationToken"]["creationTimeUtc"] == "2026-10-13T14:57:31.5757733Z"
life = read("lifecycle-and-preservation.json")
assert life["sourceCommit"] == source and life["desktopCount"] == 1 and life["protectedBytesPreserved"]
assert {value["pid"] for value in life["actualHandleReadbacks"]} == {10904, 32492}
assert all(value["actualLiveHandle"] for value in life["actualHandleReadbacks"])
assert life["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert life["focusUiSubsetAccepted"] and not life["captionRestoreClickAccepted"]
assert not any(life[key] for key in ["appOrSupervisorRestarted", "manualQueryOrInstallIssued",
                                  "fullNativeGateAccepted", "pairedPerformanceAccepted",
                                  "realFullCyclesAccepted", "formalTaskReceipt", "mainIntegrated", "releaseAccepted"])
pngs = []
for path in sorted(root.glob("*.png")):
    with Image.open(path) as image_value:
        assert image_value.format == "JPEG"
        width, height = image_value.size
        image_value.verify()
    pngs.append({"path": path.name, "actualFormat": "JPEG", "width": width, "height": height,
                 "scope": "small auxiliary capture, not full window" if min(width, height) < 100 else "native returned window or menu capture"})
assert any(value["path"].startswith("11-maximized-") and value["width"] > 1000 for value in pngs)
assert any(value["path"].startswith("11-maximized-") and min(value["width"], value["height"]) < 100 for value in pngs)
correction = read("screenshot-format-wrapper-correction.json")
assert not correction["rawScreenshotBytesRewritten"] and not correction["screenshotsRecapturedForWrapperFailure"]
assert [{"path": item["path"], "actualFormat": item["actualFormat"], "width": item["width"], "height": item["height"]}
        for item in pngs] == correction["rows"]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("screenshot-scope-inventory.json", pngs)
else:
    assert not sys.argv[1:]
assert read("screenshot-scope-inventory.json") == pngs
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "sourceCommit": source,
    "scope": "ordinary retained native manager/panel return-all focus, Tab loop/Escape opener, DWM2 at192DPI, maximize and system-menu restore; caption restore click remains unproved",
    "nativeManagerAndPanelReturnAllFocusAccepted": True, "nativeTabLoopAndEscapeOpenerAccepted": True,
    "ordinaryDwm2At192DpiObserved": True, "nativeMaximizeAndSystemMenuRestoreObserved": True,
    "captionRestoreButtonClickAccepted": False, "captionNoChangeCauseIsolated": False,
    "physical100125150DpiMatrixAccepted": False, "adminNativeGateAccepted": False,
    "longNotesOrRealUpdateProgressAccepted": False, "backupEventTreeAndInstallerWholeGateAccepted": False,
    "pairedPerformanceAccepted": False, "realFullCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "mainIntegrated": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "nativeFocusRegressionSubset": "passed",
                  "captionRestoreButtonClickAccepted": False, "fullNativeAccepted": False}))
