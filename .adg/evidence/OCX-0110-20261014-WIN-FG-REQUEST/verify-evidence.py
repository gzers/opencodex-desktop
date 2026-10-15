"""Verify this foreground subset; never promote it to complete Windows acceptance."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
terminal = read("candidate-terminal.json")
assert terminal["run"]["id"] == 38591604133
assert terminal["run"]["head_sha"] == source
assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
job = terminal["jobs"]["jobs"][0]
for name in ("Windows regression with the package lock", "Build NSIS and MSI candidates",
             "Install, verify final EXE and uninstall on isolated runner", "Upload verified Windows installers"):
    assert any(step["name"] == name and step["status"] == "completed" and step["conclusion"] == "success"
               for step in job["steps"])
download = read("candidate-download-result.json")
assert download["source"] == source and download["runId"] == 38591604133
assert download["candidateRunStatusWhenRetrieved"] == "completed"
assert download["candidateRunConclusionWhenRetrieved"] == "success"
assert not download["hostInstallerExecuted"] and not download["releaseAccepted"]
assert len(download["artifacts"]) == 2
for row in download["artifacts"]:
    assert row["digestMatchesApi"] and row["zipCrcPassed"] and row["safeEntries"]
    assert not row["authorizationForwardedAcrossHosts"]
    api_row = next(item for item in terminal["artifacts"]["artifacts"] if item["id"] == row["id"])
    assert api_row["digest"] == "sha256:" + row["sha256"] and api_row["workflow_run"]["head_sha"] == source
receipt = read("installer-verification.raw.original.json")
assert receipt == read("installer-verification.original.json")
assert receipt["sourceCommit"] == source and receipt["version"] == "0.1.10" and receipt["result"] == "pass"
msi = next(row for row in receipt["installers"] if row["kind"] == "msi")
extraction = read("msi-readonly.original.json")
assert extraction["packageSha256"] == msi["sha256"] and extraction["result"] == "pass"
assert not extraction["installerExecuted"] and extraction["databaseMode"] == "read-only"
exe = next(row for row in extraction["extractedExecutables"] if row["name"] == "opencodex-desktop.exe")
assert exe["sha256"] == msi["installedExecutableSha256"] == msi["expectedInstalledExecutableSha256"]
launch = read("ordinary-launch.json")
assert launch["source"] == source and launch["sha256"] == exe["sha256"] and launch["pid"] == 73920
assert launch["applicationToken"]["elevated"] is False and launch["applicationToken"]["elevationType"] == 3
assert not launch["hostInstallerExecuted"] and not launch["forcedScaleUsed"]
for name in ("first-screen", "maximized", "after-restore-click", "after-coordinate-restore",
             "minimized", "after-minimize-restore"):
    state = read("native-" + name + ".json")
    assert state["pid"] == launch["pid"] and state["creationTimeUtc"] == launch["creationTimeUtc"]
    assert state["applicationToken"]["elevated"] is False and state["applicationToken"]["elevationType"] == 3
    assert state["title"] == "OpenCodeX Desktop" and not state["applicationMenuAttached"]
    assert state["remoteSessionMetric"] == 1
    assert not state["physicalDpiGateClaimed"] and not state["forcedScaleUsed"]
    assert not state["inputInjectedByThisReader"] and not state["settingsChangedByThisReader"]
first = read("native-first-screen.json")
maximized = read("native-maximized.json")
index_restore = read("native-after-restore-click.json")
coordinate_restore = read("native-after-coordinate-restore.json")
minimum = read("native-minimized.json")
resumed = read("native-after-minimize-restore.json")
assert not first["maximized"] and maximized["maximized"] and index_restore["maximized"]
assert not coordinate_restore["maximized"] and coordinate_restore["clientRect"] == first["clientRect"]
assert coordinate_restore["outerRect"] == first["outerRect"]
assert minimum["minimized"] and not minimum["foreground"] and minimum["backdropRawValue"] == 1
assert not resumed["minimized"] and resumed["foreground"] and resumed["backdropRawValue"] == 2
assert first["backdropHresult"] == resumed["backdropHresult"] == 0
assert read("native-creation-comparison-fix.json")["originalJsonCreationType"] == "System.DateTime"
assert read("native-creation-guard-review.json")["sameImage"] and read("native-creation-guard-review.json")["samePid"]
preferences = read("isolated-preferences-after.original.json")
assert preferences["shell"]["keep_proxy_on_close"] is False
assert preferences["updates"]["app_update_auto_check"] is False
post = read("normal-close-postcheck.json")
assert post["nativeCount"] == 0 and post["nativeProcesses"] == [] and not post["originalPidStillPresent"]
assert post["normalCloseTriggeredViaSky"] and not post["agentForcedTermination"]
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["dailyExeAfter"] == launch["dailyExeBefore"]
assert post["protectedSandboxAfter"] == launch["protectedSandboxBefore"]
assert not post["systemDpiOrTransparencyChanged"]
log = (root / "app-log.original.txt").read_text(encoding="utf-8-sig")
assert "commit=" + source in log and "material=mica reason=mica" in log and "material=solid reason=inactive" in log
methods = read("source-methods.json")
assert not methods["oldNativeScriptExecuted"] and methods["oldNativeScriptGlobalTransparencyWriteSkipped"]
authorization = read("foreground-authorization.json")
assert authorization["physicalDpiDeferredThisWindow"] and not authorization["systemScaleChanged"]
resume = read("resume-foreground-authorization.json")
assert resume["resumeUserText"] == "继续前台验证" and resume["escapedComputerUseWasNotBypassed"]
assert not resume["newDesktopInstanceStarted"]
result = read("foreground-result.json")
assert result["source"] == source and not result["fullNativeGatePassed"]
assert result["ordinaryRealTokenVerified"] and result["normalExitVerified"]
assert not result["administratorApplicationTested"] and not result["physicalDpiTested"]
assert not result["trayReopenTested"] and not result["snapTested"]
files = sorted((path for path in root.rglob("*") if path.is_file()
                and path.name not in {"manifest.json", "archive-verification.json"}),
               key=lambda path: path.relative_to(root).as_posix())
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert {row["path"]: row for row in read("manifest.json")} == {row["path"]: row for row in manifest}
report = {"archiveIntegrity": "passed", "files": len(files), "source": source,
          "exactSourceCandidateVerified": True, "ordinaryRealTokenVerified": True,
          "maximizedAndCoordinateRestoreVerified": True, "inactiveBackdropAndResumeVerified": True,
          "normalExitVerified": True, "nativeInstancesAfter": 0,
          "fullNativeGatePassed": False, "administratorApplicationTested": False,
          "physicalDpiTested": False, "trayReopenTested": False, "snapTested": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
