"""Verify final icon implementation evidence without promoting version gates."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a"
image = "3b5c5176139fd9da4b7c5f444c2b5e7af08d2e6df9aaacd07fc9007e6ec1e989"
approval = read("user-approval-and-scope.json")
assert approval["codeCommit"] == source and approval["codeBase"] == "d3b83b3822bf10a511556d80dc851c2ccfb17cf5"
assert approval["userApproval"] == "我看了效果可以，可以继续"
assert approval["additional16PixelSafetyMargin"] == 1 and not approval["fullVersionGatePromoted"]
assets = read("local-assets-red-green.json")
assert len(assets["oldWindowsIconRegression"]) == 7
assert all("opaque-white-backing-or-artwork" in errors for errors in assets["oldWindowsIconRegression"].values())
assert len(assets["newWindowsIcoFrames"]) == 7 and len(assets["trayFrames"]) == 14
assert assets["defaultConfigurationSemanticallyUnchanged"] and len(assets["unchangedMacAndDefaultResources"]) == 5
formatter = read("official-rustfmt-completed.json")
assert formatter["status"] == "official-formatter-passed"
assert formatter["exitCode"] == formatter["checkExitCode"] == 0
assert not formatter["toolchainInstalled"] and not formatter["globalEnvironmentChanged"]
assert read("official-rustfmt-result.json")["exitCode"] == 3221225781
summary = read("final-result.json")
assert summary["source"] == source and summary["nativeExeSha256"] == image
assert set(summary["ci"]) == {"38499759600", "38499860360", "38499860139"}
for identifier, ci in summary["ci"].items():
    assert ci["status"] == "completed" and ci["conclusion"] == "success"
    if identifier != "38499860139":
        assert all(ci["jobs"][name] == "success" for name in
                   ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"])
        assert ci["jobs"]["build"] == "skipped"
    else:
        assert ci["jobs"]["package"] == "success"
archive = read("candidate-archive-verification.json")
assert archive["source"] == source and archive["runId"] == 38499860139
assert len(archive["archives"]) == len(archive["packages"]) == 2
assert all(item["sha256Matches"] and item["zipCrcPassed"] for item in archive["archives"])
assert not archive["installerInvokedOnUserHost"] and not archive["authorizationForwardedAcrossHosts"]
verification = read("installer-verification.json")
assert verification["sourceCommit"] == source and verification["result"] == "pass"
assert not verification["codeSigned"] and not verification["releaseAccepted"]
assert all(item["result"] == item["firstScreen"] == "pass" and item["uninstallRemovedExecutable"]
           for item in verification["installers"])
icons = read("executable-icon-identity.json")
assert icons["source"] == source and icons["executableSha256"] == image
assert icons["allIcoFramePixelsMatchApprovedSource"] and len(icons["groups"][0]["frames"]) == 7
assert icons["packageAndExactBundleHashMatches"] and icons["embeddedCommitMatches"]
assert not icons["executableLoadedOrStarted"]  # This is the earlier offline snapshot.
zero = read("source-switch-zero.json")
assert zero["localNativeCount"] == 0 and zero["oldApplicationGone"] and zero["oldSupervisorGone"]
native = read("native-candidate-result.json")
assert native["isolatedNativeCount"] == 1 and native["newSourceTitlebarObserved"]
assert native["currentSession"]["sourceCommit"] == source and native["currentSession"]["executableSha256"] == image
assert native["currentSession"]["applicationPid"] == summary["nativeApplicationPid"] == 39680
assert native["currentSession"]["supervisorPid"] == summary["nativeSupervisorPid"] == 44136
assert native["currentSession"]["applicationToken"]["elevated"] is False
assert native["currentSession"]["applicationToken"]["elevationType"] == 3
assert native["protectedSandbox"] == read("before-source-switch.json")["protectedSandbox"]
assert native["dailyExeSha256"] == read("before-source-switch.json")["dailyExeSha256"]
assert read("preview-originals/window-final.json")["originalWindowIconsRestored"]
assert read("preview-originals/window-final.json")["trayCleanupExitCode"] == 0
assert read("preview-originals/tray-restored.json")["result"]["originalIconRestored"]
frame = root / read("native-window-screenshot-metadata.json")["file"]
assert frame.read_bytes().startswith(b"\xff\xd8")
assert read("native-window-screenshot-metadata.json")["hwnd"] == summary["nativeHwnd"] == 2625640
assert summary["bothMonochromeTrayPngsEmbedded"]
for key in ["systemThemeSwitchPhysicallyAccepted", "taskbarTrayPixelMatrixAccepted",
            "userDailyInstallerExecuted", "userDailyInstallationReplaced", "fullVersionTaskClosed", "releaseAccepted"]:
    assert summary[key] is False
pr = read("pr-after-final-writeback.json")
assert pr["number"] == 3 and pr["head"]["sha"] == source and pr["draft"]
assert pr["state"] == "open" and not pr["merged"] and not pr["auto_merge"]
assert pr["body"] == (root / "pr-body-final.md").read_text(encoding="utf-8")
files = sorted(path for path in root.rglob("*") if path.is_file()
               and path.name not in {"manifest.json", "archive-verification.json"})
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == manifest, "Frozen evidence bytes changed"
report = {"archiveIntegrity": "passed", "files": len(files), "source": source,
          "ciAndCandidateSuccess": True, "icoFramesMatched": 7,
          "nativeSourceAndOrdinaryTokenMatched": True, "uniqueNativeInstance": 1,
          "fullVersionGatePromoted": False, "releaseAccepted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
