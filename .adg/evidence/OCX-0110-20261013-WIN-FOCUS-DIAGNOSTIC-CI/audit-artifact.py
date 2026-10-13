"""Read the exact CI ZIP; archive selected raw witnesses without executing its binaries."""
import hashlib
import json
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
receipt = read(root / "terminal-audit.json")
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert receipt["sourceCommit"] == source and receipt["runId"] == 38352525092
archive = Path(receipt["zipPath"])
assert hashlib.sha256(archive.read_bytes()).hexdigest() == receipt["zipSha256"]
selected = root / "artifact-raw"
output = root / "artifact-content-audit.json"
assert not selected.exists() and not output.exists(), "Do not overwrite any artifact witness"
with zipfile.ZipFile(archive) as z:
    infos = z.infolist()
    names = [info.filename for info in infos]
    assert len(names) == len(set(names))
    for info in infos:
        path = PurePosixPath(info.filename)
        assert not path.is_absolute() and ".." not in path.parts and "\\" not in info.filename
        assert not path.parts[0].endswith(":")
        assert info.file_size < 64 * 1024 * 1024
    assert z.testzip() is None
    def raw_json(name):
        return json.loads(z.read(name).decode("utf-8-sig"))
    startup = raw_json("windows-startup/startup.json")
    webview = raw_json("windows-startup/webview.json")
    cdp = raw_json("windows-startup/cdp-diagnostics.json")
    diagnostics = raw_json("windows-startup/startup-diagnostics.json")
    environment = raw_json("windows-backend/environment.json")
    exe_receipt = raw_json("windows-backend/test-executable-sha256.json")
    assert startup["result"] == "pass" and startup["sourceCommitMatched"]
    assert startup["expectedCommit"] == source and startup["mainWindowPresent"] and startup["responding"]
    assert startup["homeRemoved"] and startup["observedSeconds"] == 30
    assert startup["applicationTokenVerified"] and startup["mode"] == "any"
    assert not startup["applicationModeMatched"]
    assert startup["applicationToken"]["elevated"] and startup["applicationToken"]["elevationType"] == 1
    assert startup["applicationToken"]["processId"] == startup["pid"] == diagnostics["rootPid"]
    assert startup["applicationToken"] == diagnostics["applicationToken"]
    assert webview["result"] == "pass" and webview["mainUiMounted"] and webview["sandboxDataRootInitialized"]
    assert webview["about"]["version"] == "0.1.10" and webview["about"]["platform"] == "Windows"
    assert cdp["result"] == "pass" and cdp["phase"] == "complete"
    assert cdp["target"]["url"] == "http://tauri.localhost/#overview"
    assert environment["node"] == "v24.21.0"
    test_exes = [name for name in names if name.startswith("windows-backend/opencodex_desktop_lib-") and name.endswith(".exe")]
    assert len(test_exes) == 1
    test_hash = hashlib.sha256(z.read(test_exes[0])).hexdigest()
    assert test_hash == exe_receipt["Hash"].lower() == "bcb9d346a829da51b50b4b85ae32258488e66aa9c25d5868dace0d3a21276a76"
    app_logs = [name for name in names if name.startswith("windows-startup/sandbox-") and name.endswith("/logs/app.log")]
    assert len(app_logs) == 1
    assert "commit=" + source in z.read(app_logs[0]).decode("utf-8-sig")
    inventory = [
        {"path": info.filename, "bytes": info.file_size, "crc32": f"{info.CRC:08x}"}
        for info in infos
    ]
    keep = [
        "windows-startup/startup.json", "windows-startup/webview.json",
        "windows-startup/cdp-diagnostics.json", "windows-startup/startup-diagnostics.json",
        "windows-startup/startup-stderr.log", "windows-startup/startup-stdout.log",
        "windows-startup/main-window.png", "windows-backend/environment.json",
        "windows-backend/test-executable-sha256.json", "windows-backend/test-executable.manifest.xml",
        app_logs[0],
    ]
    selected.mkdir()
    witnesses = []
    for name in keep:
        path = selected / PurePosixPath(name)
        path.parent.mkdir(parents=True, exist_ok=True)
        value = z.read(name)
        path.write_bytes(value)
        witnesses.append({
            "zipMember": name, "archivedPath": path.relative_to(root).as_posix(),
            "bytes": len(value), "sha256": hashlib.sha256(value).hexdigest(),
        })
record = {
    "sourceCommit": source, "runId": receipt["runId"], "artifactId": receipt["artifactId"],
    "zipSha256": receipt["zipSha256"], "zipDigestMatchesApi": True, "zipCrcVerified": True,
    "zipEntries": len(inventory), "totalUncompressedBytes": sum(item["bytes"] for item in inventory),
    "selectedRawWitnesses": witnesses, "ciTestExeSha256": test_hash,
    "ciFinalExeSha256FromSmokeReceipt": startup["sha256"],
    "finalDesktopExePresentInZip": any(name.endswith("/opencodex-desktop.exe") for name in names),
    "ciSmokeResult": "pass", "ciSmokeActualToken": startup["applicationToken"],
    "ciSmokeMode": "any", "ciRequestedModeMatched": False,
    "ordinaryAndAdminPairedNativeTokenGateAccepted": False,
    "ciOverviewWebviewMounted": True,
    "cdpVersionEndpointTimeoutRetained": "lastVersionError" in cdp,
    "cdpListTargetAndMainOverviewSucceeded": True,
    "fullNativeFocusFixAccepted": False, "localExecutableStarted": False,
    "currentDesktopSourceCommit": "eae85322ebca3b755c034db97cbebf2ea5bf71fc",
    "currentCycleReplaced": False, "originalCiFailureCleared": False,
    "originalCiCauseIsolated": False, "mainIntegrated": False,
    "installerOrReleaseArtifactAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
}
assert not record["finalDesktopExePresentInZip"], "Receipt only: do not claim local final EXE availability"
(root / "artifact-inventory.json").write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
output.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "zipCRC": "passed", "exactSha": source, "entries": len(inventory), "selectedRawFiles": len(witnesses),
    "ciSmoke": "pass", "actualToken": {"elevated": True, "elevationType": 1},
    "finalDesktopExePresentInZip": False, "nativeFocusFixAccepted": False,
}))
