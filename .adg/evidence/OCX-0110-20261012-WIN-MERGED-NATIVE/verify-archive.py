"""Verify exact merged artifact and partial native theme readback evidence."""

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    (ROOT / name).write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )


if sys.argv[1:] == ["--build"]:
    entries = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in {"manifest.json", "archive-verification.json"}:
            continue
        assert path.suffix.lower() not in {".exe", ".msi", ".cab", ".zip", ".dll"}
        payload = path.read_bytes()
        entries.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(payload),
                        "sha256": hashlib.sha256(payload).hexdigest()})
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

for entry in read("manifest.json")["entries"]:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    payload = path.read_bytes()
    assert len(payload) == entry["bytes"]
    assert hashlib.sha256(payload).hexdigest() == entry["sha256"], entry["path"]

source = "afff490a430a81f8c233d44928aa3681a23e91e0"
candidate = read("run-38213528684-final.json")
assert candidate["run"]["head_sha"] == source
assert candidate["run"]["status"] == "completed" and candidate["run"]["conclusion"] == "success"
receipt = read("installer-verification.json")
assert receipt["sourceCommit"] == source and receipt["result"] == "pass" and receipt["version"] == "0.1.10"
assert all(item["result"] == "pass" for item in receipt["installers"])
downloads = read("download-verification.json")
assert len(downloads) == 2 and all(
    item["sourceCommit"] == source and item["matched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"] for item in downloads
)
launch = read("native-launch-input.json")
assert launch["sourceCommit"] == source and launch["ciInstalledExeMatched"]
assert launch["executableSha256"] == next(
    item["installedExecutableSha256"] for item in receipt["installers"] if item["kind"] == "msi"
)
assert not launch["installerExecutedLocally"]
session = read("ordinary-current-session.json")
assert session["sourceCommit"] == source and session["sourceCommitMatched"]
assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
assert read("old-session-after-stop.json")["status"] == "stop-requested"
assert read("old-session-after-stop.json")["sourceCommit"] == "e1a1aaa54ccc56530d5da507d0666b6ec8d2c138"
files = read("native-preference-file-verification.json")
assert files["sourceCommit"] == source
assert files["persistedSchema"] == 2 and files["persistedTheme"] == files["nativeTheme"] == files["cacheTheme"] == "system"
assert files["themeNeedsImport"] is False and files["pinSurvivedRestart"]
assert files["preferenceSha256Before"] == files["preferenceSha256After"]
assert files["mtimeBefore"] == files["mtimeAfter"] and files["noUnexpectedStartupOrReloadWrite"]
native = read("merged-native-preferences.json")["result"]
assert native["preferences"]["themeNeedsImport"] is False
assert native["reloaded"]["themeNeedsImport"] is False and native["storeError"] is False
assert native["notificationsBefore"] == native["notificationsAfter"]
blobs = read("integrated-source-byte-verification.json")
assert blobs["sourceCommit"] == source and len(blobs["entries"]) == 8
assert all(item["equal"] and item["mergedGitBlob"] == item["expectedGitBlob"] for item in blobs["entries"])
checkpoint = read("checkpoint.json")
assert checkpoint["nativeSource"] == source and checkpoint["candidateCount"] == 1
assert not checkpoint["task182Present"] and not checkpoint["scriptPermissionProvided"]
assert not checkpoint["scriptsExecuted"] and not checkpoint["realUserDataModified"]
assert checkpoint["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
write("archive-verification.json", {
    "result": "passed", "scope": "exact merged artifact and partial native theme readback",
    "filesVerified": len(read("manifest.json")["entries"]), "nativeSource": source,
    "explicitSchema2ThemeNativeReadback": "passed",
    "nativeStartupAndReloadUnexpectedWrite": "none in recorded hash/mtime observations",
    "userPinAcrossRestart": "passed for recorded pin",
    "overallMainCiAcceptance": "pending; inspect exact CI final evidence",
    "fullNativeGateAccepted": False, "formalTaskReceipt": False,
    "scriptsExecuted": False, "scriptsSecondConfirmation": "pending",
    "performanceAccepted": False, "realCyclesAccepted": False,
    "existingPerformanceFailureCleared": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"])}))
