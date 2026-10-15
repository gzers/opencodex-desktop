"""Offline verification of this bounded UI session; capture recovery and native exit remain explicit."""
import hashlib
import json
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
digest = lambda data: hashlib.sha256(data).hexdigest()
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
exe_sha = "9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a"
backup_id = "bk_20261015034010_ebc5cff3"
auth = read("foreground-authorization.json")
start = instant(auth["atUtc"])
deadline = start + timedelta(minutes=30)
assert auth["origin"] == "human" and auth["foregroundStartGranted"]
assert auth["source"] == source and auth["scope"] == "isolated-updates-tree"
assert auth["windowMinutes"] == 30 and auth["userText"] == "开始前台验证"
launch, restart = read("ordinary-launch.json"), read("ordinary-restart.json")
assert launch["pid"] == 4092 and restart["pid"] == 77760 and restart["priorPid"] == 4092
assert launch["dataRoot"] == restart["dataRoot"] and launch["executable"] == restart["executable"]
for identity in (launch, restart):
    assert identity["source"] == source and identity["sha256"] == exe_sha
    token = identity["applicationToken"]
    assert token["processId"] == identity["pid"] and token["executable"] == identity["executable"]
    assert instant(token["creationTimeUtc"]) == instant(identity["creationTimeUtc"])
    assert token["elevated"] is False and token["elevationType"] == 3
    assert start < instant(identity["creationTimeUtc"]) < deadline
    assert instant(identity["authorization"]["startedAtUtc"]) == start
    assert instant(identity["authorization"]["expiresAtUtc"]) == deadline
    assert not identity["hostInstallerExecuted"] and not identity["forcedScaleUsed"] and not identity["cdpUsedForUiInput"]
first_exit = read("first-exit-observer-result.json")
zero = read("first-exit-postcheck.json")
assert first_exit["pid"] == launch["pid"] and first_exit["exitObserved"] and first_exit["exitCode"] == 0
assert not first_exit["timeout"] and not first_exit["agentForcedTermination"]
assert zero["nativeCount"] == 0 and zero["nativeProcesses"] == [] and restart["nativeCountBefore"] == 0
assert instant(first_exit["atUtc"]) < instant(zero["atUtc"]) < instant(restart["creationTimeUtc"])
states = {}
for path in root.glob("*.sky.json"):
    value = read(path.name)
    assert value["window"]["id"] == 9110836
    assert start <= instant(value["atUtc"]) < instant(first_exit["atUtc"])
    assert value["accessibility"]["tree"]
    for shot in value["screenshots"]:
        assert (root / shot["file"]).stat().st_size > 0
    states[path.name.removesuffix(".sky.json")] = value
assert len(states) == 45
text = lambda phase: states[phase]["accessibility"].get("document_text", "")
assert "尚未检查" in text("manager-detail-unchecked") and "0.1.10" in text("manager-detail-unchecked")
assert "对话框 更新中心" in states["updates-all-from-detail-keyboard"]["accessibility"]["tree"]
assert "正在检查" in text("updates-manual-check-start")
assert "已是最新版本" in text("manager-detail-after-cross-route")
assert "测试通道" in text("beta-check-failure-terminal") and "检查失败" in text("beta-check-failure-terminal")
assert "更新下载或连接失败；已保留当前版本" in text("beta-check-failure-terminal")
for phase in ("manager-detail-unchecked", "manager-detail-after-cross-route", "beta-check-failure-terminal"):
    assert "按钮 (disabled) 安装管理器更新" in states[phase]["accessibility"]["tree"]
assert "尚无备份" in text("backup-tab-empty")
assert "展开 10" in text("backup-created-real-ui")
assert "收起 " + backup_id in text("tree-enter-expanded")
assert "展开 " + backup_id in text("tree-space-collapsed")
assert "backup-manifest.json" not in text("tree-space-collapsed")
for phase in ("tree-backup-expanded", "tree-enter-expanded", "tree-refreshed-expanded-state",
              "tree-light-expanded-full", "native-maximized-settled"):
    assert all(name in text(phase) for name in ("backup-manifest.json", "preferences.json", "名称", "用途", "操作"))
actions = read("ui-actions.json")
assert len(actions) == 41 and len({row["phase"] for row in actions}) == len(actions)
by_phase = {row["phase"]: row for row in actions}
for action in actions:
    assert start <= instant(action["atUtc"]) < instant(first_exit["atUtc"])
    assert action["window"]["id"] == 9110836
    assert action["actionReturned"]
    if action["phase"] == "first-normal-exit-request":
        assert action["kind"] == "press_key" and action["args"]["key"] == "Alt_L+F4"
        assert "foreground window did not report a process id" in action["error"]
    else:
        assert "error" not in action
for phase, key in (("manager-detail-first-tab", "Tab"), ("manager-detail-tab-back", "Tab"),
                   ("updates-all-from-detail-keyboard", "Return"),
                   ("updates-all-first-tab-after-view-switch", "Tab"),
                   ("tree-space-collapsed", "space"), ("tree-enter-expanded", "Return"),
                   ("tree-end-leaf-focus", "End"), ("tree-home-root-focus", "Home"),
                   ("tree-arrow-down-year", "Down"), ("tree-post-refresh-tab-root", "Tab")):
    assert by_phase[phase]["kind"] == "press_key" and by_phase[phase]["args"]["key"] == key
snapshots = {}
file_checks = 0
for path in root.glob("owned-state-*/snapshot.json"):
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    phase = value["phase"]
    assert path.parent.name == "owned-state-" + phase and value["source"] == source
    assert value["pid"] == (77760 if phase == "after-restart" else 4092)
    assert not value["sourceDataChanged"] and not value["uiInputInjected"] and not value["webviewProfileCopied"]
    rows = {row["path"]: row for row in value["files"]}
    assert len(rows) == len(value["files"])
    actual = {p.relative_to(path.parent).as_posix() for p in path.parent.rglob("*")
              if p.is_file() and p.name != "snapshot.json"}
    assert set(rows) == actual
    for name, row in rows.items():
        relative = PurePosixPath(name)
        assert not relative.is_absolute() and ".." not in relative.parts and "\\" not in name
        data = (path.parent / name).read_bytes()
        assert len(data) == row["bytes"] and digest(data) == row["sha256"]
        file_checks += 1
    snapshots[phase] = {"folder": path.parent, "files": rows, "atUtc": value["atUtc"]}
assert len(snapshots) == 9


def contents(phase, name):
    return (snapshots[phase]["folder"] / name).read_bytes()


def state(phase, name):
    return json.loads(contents(phase, name))


schedule = lambda phase: state(phase, "manager-state/update-schedule.json")
cache_name = "cache/updates/manager-stable.json"
stable = state("after-first-check-hidden", cache_name)
assert stable["channel"] == "stable" and stable["current_version"] == "0.1.10"
assert stable["available_version"] is None and stable["error"] is None and not stable["signature_verified"]
assert instant(by_phase["updates-manual-check-start"]["atUtc"]) < instant(stable["last_checked_at"])
# The successful check preceded closing; do not claim it was still running when hidden.
assert instant(stable["last_checked_at"]) < instant(by_phase["updates-check-hidden"]["atUtc"])
assert contents("before-beta-switch", cache_name) == contents("after-beta-switch", cache_name)
assert schedule("before-beta-switch") == schedule("after-beta-switch")
assert contents("before-beta-switch", "manager-state/notifications.json") == contents(
    "after-beta-switch", "manager-state/notifications.json")
before = schedule("before-beta-switch")["entries"]
beta = schedule("beta-check-failed")["entries"]
assert beta["manager-stable"] == before["manager-stable"]
assert beta["manager-beta"]["failures"] == 1 and beta["manager-beta"]["last_success"] is None
assert before["panel-latest"]["failures"] == 1 and beta["panel-latest"]["failures"] == 2
assert schedule("beta-check-failed") == schedule("stable-return-no-check") == schedule("after-restart")
assert contents("beta-check-failed", cache_name) == contents("stable-return-no-check", cache_name)
for phase, channel in (("before-beta-switch", "stable"), ("after-beta-switch", "beta"),
                       ("stable-return-no-check", "stable"), ("after-restart", "stable")):
    prefs = state(phase, "manager-state/preferences.json")
    assert prefs["updates"]["app_update_channel"] == channel and prefs["updates"]["app_update_auto_check"] is False
    notices = state(phase, "manager-state/notifications.json")["items"]
    assert len(notices) == 1 and notices[0]["notification_id"] == "runtime-not-found"
    assert notices[0]["resolved"] is False
for name, row in snapshots["before-first-exit"]["files"].items():
    if name.startswith(("backups/", "cache/updates/")) or name in {
        "manager-state/preferences.json", "manager-state/update-schedule.json"}:
        assert contents("before-first-exit", name) == contents("after-restart", name)
for phase in ("light-tree", "before-first-exit", "after-restart"):
    assert state(phase, "manager-state/preferences.json")["appearance"]["theme"] == "light"
backup_manifests = list(snapshots["after-restart"]["folder"].glob("backups/**/backup-manifest.json"))
assert len(backup_manifests) == 1
manifest = json.loads(backup_manifests[0].read_text(encoding="utf-8-sig"))
assert manifest["backup_id"] == backup_id and manifest["action"] == "manual-preferences"
payload = backup_manifests[0].with_name("preferences.json").read_bytes()
assert len(payload) == manifest["bytes"] and digest(payload) == manifest["sha256"]
assert json.loads(payload)["appearance"]["theme"] == "dark"
assert (root / "owned-state-before-manual-check").is_dir()
assert list((root / "owned-state-before-manual-check").rglob("*")) == []
assert not read("snapshot-first-error.json")["sourceDataChanged"]
assert not read("snapshot-first-error.json")["productFailureClaimed"]
for phase in ("first-screen", "light-tree", "maximized", "restored", "restart-first-screen"):
    name = "ordinary-" + phase + ".json"
    observation = read(name)
    assert observation["pid"] == (77760 if phase == "restart-first-screen" else 4092)
    assert observation["applicationToken"]["elevated"] is False
    assert not observation["applicationMenuAttached"] and observation["visible"]
    assert observation["remoteSessionMetric"] == 1 and observation["nativeDpi"] == 192
    assert not observation["physicalDpiGateClaimed"]
assert read("ordinary-maximized.json")["maximized"]
assert not read("ordinary-restored.json")["maximized"]
assert read("ordinary-restored.json")["outerRect"] == read("ordinary-first-screen.json")["outerRect"]
assert read("ordinary-first-screen.json")["darkModeRawValue"] == 1
assert read("ordinary-light-tree.json")["darkModeRawValue"] == 0
recovery = read("restart-capture-failure.json")
assert recovery["uiInputStoppedAfterFailedRecovery"] and not recovery["newInstanceStartedForRecovery"]
assert not recovery["forceTerminationPerformed"]
assert "FrameArrived timed out" in recovery["firstError"] and recovery["firstError"] == recovery["secondError"]
post = read("foreground-postcheck.json")
assert post["source"] == source and post["exeSha256"] == exe_sha
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"] and post["codeWorktreeClean"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not post["agentForcedTermination"] and not post["fullVersionGatePassed"]
assert post["nativeCount"] in {0, 1}
if post["nativeCount"] == 1:
    assert post["nativeProcesses"][0]["ProcessId"] == restart["pid"]
    assert post["restartExitObserved"] is False
else:
    assert post["nativeProcesses"] == []
exit_result = read("restart-exit-observer-result.json")
assert exit_result["pid"] == restart["pid"] and not exit_result["agentForcedTermination"]
assert exit_result["exitObserved"] == post["restartExitObserved"]
assert exit_result["exitCode"] == post["restartExitCode"]
if not exit_result["exitObserved"]:
    assert exit_result["timeout"] and exit_result["exitCode"] is None
else:
    assert exit_result["exitCode"] == 0
excluded = {"manifest.json", "archive-verification.json"}
paths = sorted((path for path in root.rglob("*") if path.is_file() and path.name not in excluded),
               key=lambda path: path.relative_to(root).as_posix())
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": digest(path.read_bytes())} for path in paths]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists() and not (root / "archive-verification.json").exists()
    (root / "manifest.json").write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries
report = {
    "archiveIntegrity": "passed", "scope": "493 ordinary UI subset and exact disk persistence only",
    "manifestFiles": len(entries), "source": source, "applicationSha256": exe_sha,
    "uiStates": len(states), "recordedActions": len(actions), "ownedStateSnapshots": len(snapshots),
    "copiedFileDigestChecks": file_checks, "sequentialOrdinaryTokensVerified": 2,
    "firstNormalExitCode": 0, "restartExitObserved": post["restartExitObserved"],
    "restartExitCode": post["restartExitCode"], "nativeCountAtCheckpoint": post["nativeCount"],
    "manualStableCheckRealSuccess": True, "terminalResultReusedAcrossRoute": True,
    "inFlightCheckContinuedAfterCloseProven": False, "actualDownloadOrInstallProgressVerified": False,
    "viewSwitchKeyboardFocusObservedInScreenshots": True, "fineGrainedAxFocusAvailable": False,
    "channelSwitchPreservedPanelScheduleAndNotifications": True,
    "betaManualCheckFailureRetainedAfterStableReturn": True, "fullChannelInFlightIsolationVerified": False,
    "treeKeyboardAndRefreshExpansionObserved": True, "lightAndDarkTreeScreenshotsRetained": True,
    "treePixelAlignmentMeasured": False, "asyncLeafFocusRestorationVerified": False,
    "allDensityQualityScaleAndPrototypeCheckboxMatrixVerified": False,
    "sameRootPreferenceScheduleCacheBackupPersistenceVerified": True,
    "captureRecoveryFailedTwice": True, "uiInputStoppedAfterCaptureFailure": True,
    "physicalDpiVerified": False, "pairedPerformanceVerified": False,
    "realThirtyMinuteStatusErrorBackoffVerified": False, "realCyclesVerified": False,
    "installerRetentionVerified": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "releaseAccepted": False,
}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("archive-verification.json") == report
print(json.dumps(report))
