import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "afff490a430a81f8c233d44928aa3681a23e91e0"
ordinary = read("ordinary-pre-transition-session.json")
admin = read("administrator-single-session.json")
independent = read("independent-administrator-token.json")
metrics = read("administrator-native-metrics-elevated-reader.json")
webview = read("administrator-webview-state.json")
for value in [ordinary, admin, independent, metrics, webview]:
    assert value["sourceCommit"] == source
assert ordinary["applicationToken"]["elevated"] is False
assert ordinary["applicationToken"]["elevationType"] == 3
assert admin["applicationToken"]["elevated"] is True and admin["applicationToken"]["elevationType"] == 2
assert independent["applicationToken"]["elevated"] is True and independent["applicationToken"]["elevationType"] == 2
assert independent["applicationToken"]["processId"] == admin["applicationPid"] == 42372
assert independent["applicationToken"]["creationTimeUtc"] == admin["applicationToken"]["creationTimeUtc"]
assert independent["applicationParentPid"] == admin["supervisorPid"] == 44052
assert independent["applicationSessionId"] == 2 and independent["retainedApplicationCount"] == 1
assert independent["vmInventory"]["status"] == "query_succeeded" and independent["vmInventory"]["count"] == 0
assert not independent["uiInputPerformed"] and not independent["securitySettingsChanged"]
assert metrics["applicationPid"] == 42372 and metrics["hwnd"] == 381618848 and metrics["dpi"] == 192
assert metrics["visible"] and not metrics["foreground"]
assert webview["result"]["dpr"] == 2 and webview["result"]["foreground"] is False
assert webview["result"]["originalPin"]["pinned"]
assert read("ordinary-transition-terminal.json")["applicationCount"] == 0
assert read("ordinary-transition-terminal.json")["finalOwnedSession"]["finishedAtUtc"] < admin["startedAtUtc"]
assert read("instrumentation-limitations.json")["ordinaryNativeReader"]["guardRelaxed"] is False
assert not (root / "administrator-native-metrics.json").exists()
roundtrip = read("administrator-backup-roundtrip.json")
assert roundtrip["applicationPid"] == 42372
r = roundtrip["result"]
assert "observationError" not in r and not r.get("fallbackPreferenceSaveUsed", False)
assert r["changedTheme"] == "light" and r["restoredTheme"] == "system"
assert r["restoredPreferenceDtoMatches"] and r["beforeRestorePin"]["pinned"] and r["pinAfterRestore"]["pinned"]
assert not r["restored"]["refreshRequired"] and not r["restored"]["protectionReconciliationPending"]
assert not r["protectionAfterRestore"]["protected"]
assert all(r["cleanup"][key] for key in [
    "preferenceDtoRestored", "temporaryPinRemoved", "originalUserPinRetained",
    "automaticChecksDisabled", "nativeInvokeUnchanged",
])
assert r["cleanup"]["productionStoreTheme"] == "system" and not r["cleanup"]["productionStoreError"]
assert read("administrator-roundtrip-filesystem.json")["originalBytesRestored"]
terminal = read("administrator-cleanup-terminal.json")
assert not terminal["originalAdministratorApplicationPresent"] and not terminal["originalAdministratorSupervisorPresent"]
assert terminal["applicationCount"] == 0 and not terminal["normalExitAcceptance"]
returned = read("returned-ordinary-session.json")
assert returned["applicationPid"] == 38924 and returned["applicationToken"]["elevated"] is False
assert returned["applicationToken"]["elevationType"] == 3 and returned["sourceCommitMatched"]
assert terminal["finalSession"]["finishedAtUtc"] < returned["startedAtUtc"]
verification = read("returned-ordinary-verification.json")
assert verification["matchesOriginalPreferences"] and verification["retainedApplicationCount"] == 1
assert verification["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
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
    "result": "passed", "scope": "ordinary false/type3 and administrator true/type2 real same-host application tokens; administrator native backup roundtrip",
    "administratorNativeUiMatrixAccepted": False,
    "administratorMicaForegroundAccepted": False,
    "safeInstallerEnvironmentAvailable": False, "fullNativeGateAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
