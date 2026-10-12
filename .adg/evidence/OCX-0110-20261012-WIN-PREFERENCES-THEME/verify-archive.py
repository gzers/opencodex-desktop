"""Verify original native finding and local tests; exact CI remains separate."""

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
        assert path.suffix.lower() not in {".exe", ".msi", ".zip", ".cab", ".dll"}
        payload = path.read_bytes()
        entries.append(
            {"path": path.relative_to(ROOT).as_posix(), "bytes": len(payload),
             "sha256": hashlib.sha256(payload).hexdigest()}
        )
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

for entry in read("manifest.json")["entries"]:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    data = path.read_bytes()
    assert len(data) == entry["bytes"]
    assert hashlib.sha256(data).hexdigest() == entry["sha256"], entry["path"]

finding = read("finding.json")
assert finding["sourceCommit"] == "ee65f1158c4f0bb7c283ff09e24120a7884472d0"
assert finding["persistedSchema"] == 2 and finding["persistedAppearanceTheme"] == "system"
assert finding["nativeTheme"] == "system" and finding["nativeThemeNeedsImport"]
assert not finding["topLevelThemePresent"] and finding["readDidNotModifyPreferences"]
assert finding["preferenceSha256Before"] == finding["preferenceSha256After"]
local = read("local-validation.json")
assert local["fixSource"] == "cf71ea4f84b82b2e4c1bb3a15de88fd5815dc066"
red = (ROOT / local["red"]["log"]).read_text(encoding="utf-8-sig")
green = (ROOT / local["green"]["log"]).read_text(encoding="utf-8-sig")
full = (ROOT / local["fullUi"]["log"]).read_text(encoding="utf-8-sig")
assert "3 failed | 9 passed" in red and "12 passed" in green
assert "480 passed" in full and "95 passed" in full
assert local["typecheck"]["exitCode"] == local["productionBuild"]["exitCode"] == local["rustfmt"]["exitCode"] == 0
assert (ROOT / "build.log").read_text(encoding="utf-8-sig").count("INEFFECTIVE_DYNAMIC_IMPORT") == 4
checkpoint = read("checkpoint.json")
assert not checkpoint["task182Present"] and not checkpoint["scriptsApprovalProvided"]
assert not checkpoint["scriptsExecuted"] and not checkpoint["realUserDataModified"]
assert checkpoint["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert read("cwd-candidate-final.json")["run"]["conclusion"] == "success"
write("archive-verification.json", {
    "result": "passed",
    "scope": "frozen finding and local validation evidence integrity only",
    "filesVerified": len(read("manifest.json")["entries"]),
    "fixSource": local["fixSource"],
    "nativeOriginalBugObserved": True,
    "nativeFixVerification": "pending rebuilt exact candidate",
    "rustRegression": "pending exact CI",
    "scriptsExecuted": False,
    "scriptsSecondConfirmation": "pending",
    "formalTaskReceipt": False,
    "fullNativeGateAccepted": False,
    "performanceAccepted": False,
    "realCyclesAccepted": False,
    "existingPerformanceFailureCleared": False,
    "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"])}))
