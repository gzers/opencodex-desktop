"""Freeze and verify only completed foreground records; leave a live exit observer outside the archive."""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
manifest_name = "foreground-resume-manifest-20261015.json"
report_name = "foreground-resume-verification-20261015.json"
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"

if "--build" in sys.argv:
    assert not (root / manifest_name).exists() and not (root / report_name).exists()
    terminal = read("administrator-transition-observer-result.json")
    assert not terminal["forcedTermination"] and not terminal["inputInjected"]
    prior_paths = {row["path"] for row in read("checkpoint-manifest-20261015.json")}
    prior_paths.update({"checkpoint-manifest-20261015.json", "checkpoint-verification-20261015.json"})
    prior_paths.update({
        "baseline-git-proof.py", "verify-baseline-git-proof.py", "audit-staging.py", ".gitattributes",
    })
    # The exit observer can finish after this bounded transition checkpoint.
    # Its ready record is immutable; its future result is a separate observation.
    excluded = {manifest_name, report_name, "administrator-exit-observer-result.json"}
    paths = sorted(
        (path for path in root.iterdir() if path.is_file() and path.name not in prior_paths | excluded),
        key=lambda path: path.name,
    )
    manifest = [
        {"path": path.name, "bytes": path.stat().st_size,
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

ordinary = read("ordinary-launch.json")
administrator = read("administrator-launch.json")
reopened = read("ordinary-tray-reopened-user-resume.json")
hidden = read("ordinary-hidden-main-hwnd.json")
ordinary_exit = read("ordinary-exit-observer-result.json")
ordinary_post = read("ordinary-exit-postcheck.json")
for launch in (ordinary, administrator):
    assert launch["source"] == source
    assert launch["sha256"] == "2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3"
    assert not launch["hostInstallerExecuted"] and not launch["forcedScaleUsed"]
assert ordinary["applicationToken"]["elevated"] is False and ordinary["applicationToken"]["elevationType"] == 3
assert administrator["applicationToken"]["elevated"] is True and administrator["applicationToken"]["elevationType"] == 2
assert ordinary["dataRoot"] == administrator["dataRoot"]
assert hidden["pid"] == reopened["pid"] == ordinary["pid"] == 56448
assert hidden["creationTimeUtc"] == reopened["creationTimeUtc"] == ordinary["creationTimeUtc"]
assert hidden["hwnd"] == reopened["hwnd"] and not hidden["visible"] and reopened["visible"]
assert hidden["backdropRawValue"] == 1 and reopened["backdropRawValue"] == 2
assert ordinary_exit["exitObserved"] and ordinary_exit["exitCode"] == 0
assert not ordinary_exit["forcedTerminationByObserver"] and not ordinary_post["agentForcedTermination"]
assert ordinary_post["nativeCount"] == 0 and ordinary_post["normalExitObserved"]
assert datetime.fromisoformat(ordinary_post["atUtc"]) < datetime.fromisoformat(administrator["creationTimeUtc"])
first_admin = read("administrator-first-screen.json")
assert first_admin["pid"] == administrator["pid"] == 73048
assert first_admin["creationTimeUtc"] == administrator["creationTimeUtc"]
assert first_admin["applicationToken"]["elevated"] is True
assert first_admin["applicationToken"]["elevationType"] == 2
assert first_admin["visible"] and not first_admin["applicationMenuAttached"]
prefs_ordinary = read("ordinary-before-normal-exit-preferences.original.json")
prefs_admin = read("administrator-preferences.original.json")
assert prefs_ordinary == prefs_admin
assert prefs_admin["shell"]["keep_proxy_on_close"] is False
assert prefs_admin["updates"]["app_update_auto_check"] is False
protection = read("resume-protection-check-20261015T0812.json")
assert protection["dailyExeUnchanged"] and protection["protectedSandboxUnchanged"]
assert protection["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert protection["codeHead"] == source and not protection["codeDirty"]
transition_bytes = (root / "administrator-window-transitions.original.jsonl").read_bytes()
assert transition_bytes.endswith(b"\n")
rows = [json.loads(line) for line in transition_bytes.splitlines() if line.strip()]
terminal = read("administrator-transition-observer-result.json")
assert rows and len(rows) == terminal["samples"]
for row in rows:
    assert row["pid"] == administrator["pid"] and row["creationTimeUtc"] == administrator["creationTimeUtc"]
    assert row["hwnd"] == first_admin["hwnd"] and not row["inputInjected"] and not row["settingsChanged"]
max_indices = [index for index, row in enumerate(rows) if row["maximized"]]
min_indices = [index for index, row in enumerate(rows) if row["minimized"]]
restored_max = any(not row["maximized"] and not row["minimized"] and row["visible"]
                   for index, row in enumerate(rows) if max_indices and index > max_indices[0])
restored_min = any(not row["minimized"] and row["visible"]
                   for index, row in enumerate(rows) if min_indices and index > min_indices[0])
isolation = read("installer-isolation-availability-readonly.json")
assert isolation["queriedReadOnly"] and not isolation["featureEnabledByAgent"]
assert not isolation["virtualMachineCreated"] and not isolation["installerExecuted"]

report = {
    "archiveIntegrity": "passed",
    "manifestFiles": len(manifest),
    "source": source,
    "ordinaryUserAssistedTrayReopenVerified": True,
    "ordinaryBackdropRestored": True,
    "ordinaryNormalExitVerified": True,
    "ordinaryExitCode": 0,
    "nativeCountBeforeAdministrator": 0,
    "sameConfiguredSandboxRoot": True,
    "administratorProductRealTokenVerified": True,
    "administratorNativeSingleTitleBarObserved": True,
    "administratorWindowSamples": len(rows),
    "administratorMaximizeRestoreObserved": bool(max_indices) and restored_max,
    "administratorMinimizeRestoreObserved": bool(min_indices) and restored_min,
    "administratorActiveMicaObserved": any(row["foreground"] and row["backdropRawValue"] == 2 for row in rows),
    "administratorProductExitedDuringTransitions": terminal["productExited"],
    "administratorExitResultOutsideThisCheckpoint": True,
    "trayQuitVerified": False,
    "physicalDpiVerified": False,
    "snapVerified": False,
    "performanceGatePassed": False,
    "fullNativeGatePassed": False,
    "taskOrVersionCompletionClaimed": False,
}
if "--build" in sys.argv:
    (root / report_name).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_name)
print(json.dumps(report, ensure_ascii=False))
