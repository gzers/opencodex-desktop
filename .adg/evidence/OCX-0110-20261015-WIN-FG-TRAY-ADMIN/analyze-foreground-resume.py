"""Analyze real foreground observations without promoting missing cases to passed."""
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
ordinary = read("ordinary-launch.json")
administrator = read("administrator-launch.json")
reopened = read("ordinary-tray-reopened-user-resume.json")
hidden = read("ordinary-hidden-main-hwnd.json")
ordinary_exit = read("ordinary-exit-observer-result.json")
ordinary_post = read("ordinary-exit-postcheck.json")
first_admin = read("administrator-first-screen.json")
admin_ready = read("administrator-exit-observer-ready.json")
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
for launch in (ordinary, administrator):
    assert launch["source"] == source
    assert launch["sha256"] == "2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3"
    assert not launch["hostInstallerExecuted"] and not launch["forcedScaleUsed"]
assert ordinary["applicationToken"]["elevated"] is False
assert ordinary["applicationToken"]["elevationType"] == 3
assert administrator["applicationToken"]["elevated"] is True
assert administrator["applicationToken"]["elevationType"] == 2
assert administrator["harnessToken"]["elevated"] is True
assert ordinary["dataRoot"] == administrator["dataRoot"]
assert hidden["pid"] == reopened["pid"] == ordinary["pid"]
assert hidden["creationTimeUtc"] == reopened["creationTimeUtc"] == ordinary["creationTimeUtc"]
assert hidden["hwnd"] == reopened["hwnd"] and not hidden["visible"] and reopened["visible"]
assert hidden["backdropRawValue"] == 1 and reopened["backdropRawValue"] == 2
assert ordinary_exit["exitObserved"] and ordinary_exit["exitCode"] == 0
assert not ordinary_exit["forcedTerminationByObserver"]
assert ordinary_post["nativeCount"] == 0 and ordinary_post["normalExitObserved"]
assert not ordinary_post["agentForcedTermination"] and not ordinary_post["trayQuitClaimed"]
assert datetime.fromisoformat(ordinary_post["atUtc"]) < datetime.fromisoformat(administrator["creationTimeUtc"])
assert first_admin["pid"] == admin_ready["pid"] == administrator["pid"]
assert first_admin["creationTimeUtc"] == admin_ready["creationTimeUtc"] == administrator["creationTimeUtc"]
assert first_admin["applicationToken"]["elevated"] is True
assert first_admin["applicationToken"]["elevationType"] == 2
assert first_admin["visible"] and not first_admin["applicationMenuAttached"]
assert admin_ready["handleHeld"] and admin_ready["observerOnly"]

transition_path = root / "administrator-window-transitions.original.jsonl"
raw = transition_path.read_bytes() if transition_path.exists() else b""
# A live writer may end in a partial line. Only completed original records count.
lines = raw.split(b"\n")
rows = [json.loads(line) for line in lines[:-1] if line.strip()]
for row in rows:
    assert row["pid"] == administrator["pid"] and row["creationTimeUtc"] == administrator["creationTimeUtc"]
    assert row["hwnd"] == first_admin["hwnd"] and not row["inputInjected"] and not row["settingsChanged"]
max_indices = [index for index, row in enumerate(rows) if row["maximized"]]
minimum_indices = [index for index, row in enumerate(rows) if row["minimized"]]
restored_after_max = any(
    not row["maximized"] and not row["minimized"] and row["visible"]
    for index, row in enumerate(rows) if max_indices and index > max_indices[0]
)
resumed_after_min = any(
    not row["minimized"] and row["visible"]
    for index, row in enumerate(rows) if minimum_indices and index > minimum_indices[0]
)
exit_path = root / "administrator-exit-observer-result.json"
admin_exit = read(exit_path.name) if exit_path.exists() else None
if admin_exit:
    assert admin_exit["pid"] == administrator["pid"]
    assert admin_exit["creationTimeUtc"] == administrator["creationTimeUtc"]
    assert not admin_exit["forcedTerminationByObserver"]
result = {
    "analyzedAtUtc": datetime.now(timezone.utc).isoformat(),
    "source": source,
    "ordinaryUserAssistedTrayReopenVerified": True,
    "ordinarySamePidHwndAndCreationVerified": True,
    "ordinaryBackdropRestored": True,
    "ordinaryNormalExitVerified": True,
    "ordinaryExitCode": 0,
    "nativeCountBeforeAdministrator": 0,
    "sameConfiguredSandboxRoot": True,
    "administratorProductRealTokenVerified": True,
    "administratorSingleSystemTitleBarObserved": True,
    "administratorAccessibilityLimitedToNativeTitleBar": True,
    "administratorUiInputAttemptsProducedNoConfirmedChange": True,
    "administratorWindowObservationSamples": len(rows),
    "administratorMaximizedObserved": bool(max_indices),
    "administratorRestoreAfterMaximizeObserved": restored_after_max,
    "administratorMinimizedObserved": bool(minimum_indices),
    "administratorRestoreAfterMinimizeObserved": resumed_after_min,
    "administratorActiveMicaObserved": any(row["foreground"] and row["backdropRawValue"] == 2 for row in rows),
    "administratorExitObserverTerminal": admin_exit is not None,
    "administratorNormalExitVerified": bool(admin_exit and admin_exit["exitObserved"] and admin_exit["exitCode"] == 0),
    "trayQuitVerified": False,
    "physicalDpiVerified": False,
    "snapVerified": False,
    "performanceGatePassed": False,
    "fullNativeGatePassed": False,
    "taskCompletionClaimed": False,
}
print(json.dumps(result, ensure_ascii=False))
