"""Verify a historical, append-only checkpoint; never infer a completed tray/admin gate."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
manifest_name = "checkpoint-manifest-20261015.json"
report_name = "checkpoint-verification-20261015.json"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
launch = read("ordinary-launch.json")
assert launch["source"] == source and launch["pid"] == 56448
assert launch["sha256"] == "2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3"
assert not launch["hostInstallerExecuted"] and not launch["forcedScaleUsed"]
first = read("ordinary-first-screen.json")
first_close = read("ordinary-close-hidden.json")
helper = read("ordinary-after-alt-f4.json")
hidden = read("ordinary-hidden-main-hwnd.json")
resumed = read("ordinary-user-tray-visible-resumed-20261015T0724.json")
for state in (first, first_close, helper, hidden, resumed):
    assert state["pid"] == launch["pid"] and state["creationTimeUtc"] == launch["creationTimeUtc"]
    assert state["applicationToken"]["elevated"] is False
    assert state["applicationToken"]["elevationType"] == 3
    assert not state["physicalDpiGateClaimed"] and not state["forcedScaleUsed"]
    assert not state["inputInjectedByThisReader"] and not state["settingsChangedByThisReader"]
assert first["visible"] and first_close["visible"]
assert helper["hwnd"] != first["hwnd"], "Retain the erroneous helper-HWND observation."
assert hidden["hwnd"] == resumed["hwnd"] == first["hwnd"] == 2229478
assert not hidden["visible"] and not resumed["visible"]
assert hidden["backdropRawValue"] == resumed["backdropRawValue"] == 1
preferences = read("ordinary-resumed-preferences.original.json")
assert preferences["shell"]["keep_proxy_on_close"] is True
assert preferences["updates"]["app_update_auto_check"] is False
boundary = read("tray-tool-boundary.json")
assert boundary["trayUiNotExposedBySky"] and boundary["hiddenWindowNotInSkyList"]
assert not boundary["nativeVisibilityManipulationUsed"] and not boundary["newProcessStartedToRecoverUi"]
snapshot = read("protection-and-source-checkpoint-20261015.json")
assert snapshot["codeHead"] == snapshot["codeRemote"] == source
assert snapshot["docsHead"] == snapshot["docsRemote"] == "c7d3add3083e8efb5d701a7019b6cb83b5e19e42"
assert snapshot["nativeCount"] == 1 and snapshot["nativeProcesses"][0]["ProcessId"] == 56448
assert snapshot["dailyExeUnchanged"] and snapshot["protectedSandboxUnchanged"]
assert snapshot["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not snapshot["administratorStarted"] and not snapshot["ordinaryNormalExitClaimed"]
assert not snapshot["originalGatePassed"] and not snapshot["foregroundInputDuringResume"]
entry = read("tray-entry-resume-observation-20261015.json")
assert hashlib.sha256((root / entry["userImage"]).read_bytes()).hexdigest() == entry["userImageSha256"]
assert not entry["trayReopenClaimed"] and not entry["trayExitClaimed"]
interim = read("interim-result.json")
assert not interim["normalExitObserved"] and not interim["adminInstanceStarted"]
log = (root / "ordinary-resumed-app-log.original.txt").read_text(encoding="utf-8-sig")
assert "commit=" + source in log and "material=mica reason=mica" in log
assert "material=solid reason=inactive" in log
baseline = read("baseline-preflight-result.json")
assert baseline["nativeApplicationsStarted"] == 0 and not baseline["installerExecuted"]
assert not baseline["performanceMeasured"] and not baseline["performanceGatePassed"]
if baseline["result"] == "baseline_provenance_verified":
    assert baseline["officialAssetDigestMatched"]
    assert baseline["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
    budgets = baseline["originalBudgets"]
    assert budgets["startupSamplesPerVersionPerColdOrWarmModeMin"] == 20
    assert budgets["interactiveStartupP95IncreaseMsMax"] == 100
    assert budgets["interactiveStartupP95IncreasePercentMax"] == 5
    assert budgets["idleSampleSecondsPerVersionMin"] == 600
    assert budgets["cpuIncreasePercentagePointsMax"] == 0.5 and budgets["rssIncreaseMiBMax"] == 10
else:
    assert baseline["result"] == "preflight_incomplete" and "error" in baseline

if "--build" in sys.argv:
    assert not (root / manifest_name).exists() and not (root / report_name).exists()
    paths = sorted(
        (path for path in root.rglob("*") if path.is_file() and path.name not in {manifest_name, report_name}),
        key=lambda path: path.relative_to(root).as_posix(),
    )
    manifest = [
        {"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
         "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for path in paths
    ]
    (root / manifest_name).write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    manifest = read(manifest_name)
    assert len({row["path"] for row in manifest}) == len(manifest)
    for row in manifest:
        path = root / row["path"]
        assert path.resolve().is_relative_to(root.resolve())
        assert path.stat().st_size == row["bytes"]
        assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]

report = {
    "checkpointIntegrity": "passed",
    "manifestFiles": len(manifest),
    "scope": "Only the named historical checkpoint; later append-only records need a separate verification.",
    "source": source,
    "ordinaryRealTokenVerified": True,
    "closeToHiddenMainWindowObserved": True,
    "sameNativeInstanceStillAliveAtCheckpoint": True,
    "normalExitVerified": False,
    "trayReopenVerified": False,
    "administratorApplicationVerified": False,
    "snapVerified": False,
    "physicalDpiVerified": False,
    "performanceMeasured": False,
    "baselinePreflightResult": baseline["result"],
    "fullNativeGatePassed": False,
    "taskOrVersionCompletionClaimed": False,
}
if "--build" in sys.argv:
    (root / report_name).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_name)
print(json.dumps(report, ensure_ascii=False))
