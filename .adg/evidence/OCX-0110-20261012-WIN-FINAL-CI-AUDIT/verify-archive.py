"""Verify CI and incomplete-goal audit, never issue an acceptance certificate."""

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
ci = read("main-ci-final.json")
assert ci["run"]["head_sha"] == source and ci["run"]["status"] == "completed"
assert ci["run"]["conclusion"] == "success" and ci["run"]["run_attempt"] == 1
assert len(ci["jobs"]) == 6 and all(
    job["status"] == "completed" and job["conclusion"] == "success" for job in ci["jobs"]
)
build = next(job for job in ci["jobs"] if job["name"] == "build")
assert next(step for step in build["steps"] if step["name"] == "Build macOS arm64 bundle")["conclusion"] == "success"
state = read("current-state-audit.json")
assert state["sourceCommit"] == state["main"] == source
assert not state["taskAuthorityPresent"] and not state["originalTaskReconstructed"]
assert not state["scriptApprovalRecorded"] and not state["scriptsExecuted"]
assert state["scriptPlanState"] == "awaiting_explicit_second_confirmation"
assert state["candidateCount"] == len(state["applications"]) == 1
assert state["applicationCreationTimeUtc"] == state["retainedCreationTimeUtc"]
assert state["applicationToken"]["elevated"] is False and state["applicationToken"]["elevationType"] == 3
assert state["supervisorPresent"]
assert not state["realUserDataModified"] and not state["localInstallerExecuted"]
assert state["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
audit = read("requirement-audit.json")
assert audit["sourceCommit"] == source and audit["objectiveScopeUnchanged"]
assert audit["overallResult"] == "not_achieved"
assert len(audit["requirements"]) == 14 and any(
    entry["status"] == "incomplete" for entry in audit["requirements"]
)
assert not audit["requiredChecksClosureAccepted"] and not audit["formalTaskReceipt"]
assert not audit["formalCertificate"] and not audit["originalPerformanceFailureCleared"]
assert len(audit["pendingUserPrerequisites"]) == 2 and audit["liveCiWaitEnded"]
assert audit["desktopConnectionRestored"] and audit["independentNativeWorkCanContinue"]
assert audit["blockedGoalMarkingDeferred"]
assert state["desktopSessionState"] == "connected/session2 Active"
assert "15119" in (ROOT / "windows-sessions-recheck.txt").read_text(encoding="utf-8-sig")
caption = read("native-caption/native-caption-summary.json")
assert caption["applicationPid"] == state["applicationPid"]
assert caption["sourceCommit"] == source and caption["physicalDpi"] == 192
assert caption["maximizedBySystemButton"] and caption["restoredBySystemAltSpaceMenu"]
assert caption["minimizedBySystemButton"] and caption["restoredWithExistingWindow"]
assert not caption["completeCaptionGateAccepted"]
assert read("native-caption/ordinary-maximized-native.json")["maximized"]
assert read("native-caption/ordinary-minimized-main-native.json")["minimized"]
restored = read("native-caption/ordinary-after-minimize-restore-native.json")
assert restored["hwnd"] == caption["mainHwnd"] and not restored["minimized"]
assert restored["foreground"] and restored["backdropAttribute"] == 2
write("archive-verification.json", {
    "result": "passed",
    "scope": "final exact CI and evidence of unachieved objective / repeated prerequisites",
    "filesVerified": len(read("manifest.json")["entries"]),
    "sourceCommit": source, "mainCiJobCount": 6, "mainCi": "success",
    "overallObjective": "not_achieved", "taskAuthority": "missing",
    "desktop": "connected; independent native work can continue", "scriptsPermission": "not provided",
    "ordinaryCaptionSubset": "recorded max/menu restore/minimize/recovery passed; full matrix pending",
    "formalTaskReceipt": False, "formalCertificate": False,
    "fullNativeGateAccepted": False, "performanceAccepted": False,
    "realCyclesAccepted": False, "existingPerformanceFailureCleared": False,
    "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"]),
                  "overallObjective": "not_achieved"}))
