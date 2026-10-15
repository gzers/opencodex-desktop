"""Analyze completed owned observations only; never infer missing administrator interactions."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
ordinary, administrator = read("ordinary-launch.json"), read("administrator-launch.json")
first_exit = read("first-exit-observer-result.json")
assert ordinary["source"] == administrator["source"] == source
assert ordinary["sha256"] == administrator["sha256"] == "9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a"
assert ordinary["dataRoot"] == administrator["dataRoot"] and ordinary["executable"] == administrator["executable"]
assert ordinary["pid"] == 49492 and administrator["pid"] == 78512
assert ordinary["applicationToken"]["elevated"] is False and ordinary["applicationToken"]["elevationType"] == 3
assert administrator["applicationToken"]["elevated"] is True and administrator["applicationToken"]["elevationType"] == 2
assert administrator["harnessToken"]["elevated"] is True and administrator["nativeCountBefore"] == 0
assert first_exit["pid"] == ordinary["pid"] and first_exit["exitObserved"] and first_exit["exitCode"] == 0
assert instant(first_exit["atUtc"]) < instant(administrator["creationTimeUtc"])
assert read("administrator-preflight.json")["nativeCount"] == 0
for identity in (ordinary, administrator):
    assert identity["authorization"]["userText"] == "我批准继续"
    assert identity["authorization"]["windowMinutes"] == 30
    assert not identity["cdpUsedForUiInput"] and not identity["hostInstallerExecuted"]
    assert not identity["preferencesChangedByLauncher"] and not identity["backupsChangedByLauncher"]
native_first = read("ordinary-first-screen.json")
minimized = read("ordinary-minimized.json")
restored = read("ordinary-restored-after-minimize.json")
assert all(value["pid"] == ordinary["pid"] and value["hwnd"] == 4786352 for value in (native_first, minimized, restored))
assert minimized["minimized"] and not minimized["foreground"] and minimized["backdropRawValue"] == 1
assert not restored["minimized"] and restored["foreground"] and restored["backdropRawValue"] == 2
assert restored["outerRect"] == native_first["outerRect"]
assert native_first["backdropRawValue"] == 2
ui = {}
for path in root.glob("*.sky.json"):
    value = read(path.name)
    assert value["window"]["id"] in {4786352, 8786154}
    for shot in value["screenshots"]:
        assert (root / shot["file"]).stat().st_size > 0
    ui[path.name.removesuffix(".sky.json")] = value
text = lambda phase: ui[phase]["accessibility"].get("tree", "")
assert "对话框 打开备份目录" in text("root-open-warning")
assert "对话框" not in text("root-open-cancelled")
assert "对话框 打开备份文件" in text("file-open-warning")
assert "对话框" not in text("file-open-cancelled")
assert "收起 manual-preferences" in text("right-key-expand-action")
assert "backup-manifest.json" in text("right-key-expand-backup")
assert "backup-manifest.json" not in text("left-key-collapsed-parent")
directory = read("actual-directory-open.json")
assert directory["ownYearFolderVisible"] and directory["window"]["app"] == r"process:C:\Windows\explorer.exe"
assert directory["window"]["title"] == "backups - 文件资源管理器"
assert not directory["fullExplorerScreenshotOrNavigationTreeSaved"]
assert len(directory["observedOwnBreadcrumbLines"]) == 7
assert any("foreground-updates-493-20261015T1130" in line for line in directory["observedOwnBreadcrumbLines"])
snapshots = {}
for path in root.glob("owned-state-*/snapshot.json"):
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    for row in value["files"]:
        data = (path.parent / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
    snapshots[value["phase"]] = value
assert len(snapshots) == 3
initial = {row["path"]: row for row in snapshots["initial-reused-root"]["files"]}
for phase in ("after-open-and-navigation", "before-ordinary-exit"):
    assert initial == {row["path"]: row for row in snapshots[phase]["files"]}
sampler_ready = read("administrator-window-sampler-ready.json")
sampler_result = read("administrator-window-sampler-result.json")
assert sampler_ready["pid"] == sampler_result["pid"] == administrator["pid"]
assert instant(sampler_ready["creationTimeUtc"]) == instant(administrator["creationTimeUtc"])
assert sampler_ready["actualToken"]["elevated"] and sampler_ready["actualToken"]["elevationType"] == 2
assert sampler_ready["handleHeld"] and sampler_ready["maxSeconds"] == 240
assert not sampler_ready["inputInjected"] and not sampler_ready["systemSettingsChanged"]
assert not sampler_ready["unrelatedWindowIdsOrTitlesSaved"]
rows = [json.loads(line) for line in (root / "administrator-window-samples.original.jsonl").read_text(
    encoding="utf-8-sig").splitlines() if line.strip()]
assert len(rows) == sampler_result["samples"] and rows
assert all(row["pid"] == administrator["pid"] and row["hwnd"] == 8786154
           and instant(row["creationTimeUtc"]) == instant(administrator["creationTimeUtc"]) for row in rows)
assert all(instant(row["atUtc"]) >= instant(sampler_ready["atUtc"]) for row in rows)
assert all(rows[i]["elapsedMilliseconds"] <= rows[i + 1]["elapsedMilliseconds"] for i in range(len(rows) - 1))
max_rows = [row for row in rows if row["maximized"]]
min_rows = [row for row in rows if row["minimized"]]
active_mica = [row for row in rows if row["foreground"] and not row["minimized"]
               and row["backdropHresult"] == 0 and row["backdropValue"] == 2]
inactive_solid = [row for row in rows if not row["foreground"]
                  and row["backdropHresult"] == 0 and row["backdropValue"] == 1]
ordinary_exit_refresh_error = next(row for row in read("ui-actions.json") if row["phase"] == "ordinary-normal-close")
assert ordinary_exit_refresh_error["actionReturned"] and "foreground window did not report a process id" in ordinary_exit_refresh_error["error"]
assert not read("administrator-helper-precondition-error.json")["uiInputIssued"]
assert read("administrator-coordinate-action.json")["actionReturned"]
report = {
    "source": source, "ordinaryPid": ordinary["pid"], "administratorPid": administrator["pid"],
    "ordinaryNormalExitCode": 0, "sameCandidateAndRootSequentialDualTokensVerified": True,
    "directoryAndFileWarningsAndCancelObserved": True, "actualOwnedDirectoryOpenObserved": True,
    "treeRightAndLeftKeyboardBehaviorObserved": True, "ordinaryMinimizeRestoreAndDwmTransitionVerified": True,
    "ordinaryStateSnapshotFilesUnchanged": len(initial),
    "administratorInputHadNoConfirmedNavigation": True, "administratorUiAutomationGatePassed": False,
    "administratorSamplerSamples": len(rows), "administratorMaximizedSamples": len(max_rows),
    "administratorMinimizedSamples": len(min_rows), "administratorForegroundMicaSamples": len(active_mica),
    "administratorInactiveSolidSamples": len(inactive_solid),
    "administratorSamplerExitObserved": sampler_result["exitObserved"],
    "administratorSamplerExitCode": sampler_result["exitCode"],
    "samplerCoversOnlyObservedInstants": True, "missingWindowStatesInferred": False,
    "administratorFullWindowLifecycleAccepted": False, "physicalDpiVerified": False,
    "installerRetentionVerified": False, "actualUpdateDownloadProgressVerified": False,
    "realThirtyMinuteBackoffVerified": False, "pairedPerformanceVerified": False,
    "historicalFailuresCleared": False, "fullVersionGatePassed": False,
}
if "--verify" in sys.argv:
    assert read("session-analysis.json") == report
else:
    with (root / "session-analysis.json").open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
print(json.dumps(report))
