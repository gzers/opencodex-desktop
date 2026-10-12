"""Frozen cwd observations; never grant permission or full acceptance."""

import gzip
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
        data = path.read_bytes()
        entries.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

for entry in read("manifest.json")["entries"]:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    data = path.read_bytes()
    assert len(data) == entry["bytes"]
    assert hashlib.sha256(data).hexdigest() == entry["sha256"], entry["path"]

source = "e1a1aaa54ccc56530d5da507d0666b6ec8d2c138"
for name in ["run-38210513009-final.json", "run-38210653937-final.json"]:
    run = read(name)["run"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == "success"
downloads = read("download-verification.json")
assert len(downloads) == 2
assert all(
    item["sourceCommit"] == source and item["matched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"]
    for item in downloads
)
analysis = read("candidate-regression-log-analysis.json")
raw = gzip.decompress((ROOT / "exact-candidate-job.log.gz").read_bytes())
assert len(raw) == analysis["originalBytes"]
assert hashlib.sha256(raw).hexdigest() == analysis["originalSha256"]
assert analysis["sourceCommit"] == source
assert len(analysis["tests"]) == 3 and all(item["result"] == "passed" for item in analysis["tests"])
for item in analysis["tests"]:
    assert item["line"].encode() in raw
launch = read("native-launch-input.json")
assert launch["sourceCommit"] == source and launch["ciInstalledExeMatched"]
assert not launch["installerExecutedLocally"]
assert launch["executableSha256"] == "e3dd402aff7e9e194940b013cbec017de28b0b20c5e42fa138937f0de0fe3fe9"
session = read("ordinary-current-session.json")
assert session["sourceCommit"] == source and session["sourceCommitMatched"]
assert not session["applicationToken"]["elevated"] and session["applicationToken"]["elevationType"] == 3
native = read("native-cwd-verification.json")
assert native["sourceCommit"] == source
assert native["actualInstallCwd"] == native["installPrefix"]
assert native["actualPackCwd"] == native["packDestination"]
assert native["actualPackCwd"].startswith(native["actualInstallCwd"] + "\\")
assert not native["actualInstallCwd"].startswith("\\\\?\\")
assert native["npmInstallExitCode"] == native["npmPackExitCode"] == 0
assert native["ignoreScripts"] and not native["scriptsExecuted"]
assert native["runtimeInstallResult"] == "scripts_required" and native["sourceKindAfter"] == "unresolved"
assert native["listenerRemoved"]
operation = read("no-script-native-install-observation.json")["result"]["operation"]
assert operation["state"] == "failed" and "scripts_required" in operation["error"]["message"]
assert operation["installerScriptsAllowed"] is False
checkpoint = read("checkpoint.json")
assert checkpoint["main"] == source and checkpoint["exactCandidateCount"] == 1
assert not checkpoint["task182Present"] and not checkpoint["scriptApprovalPresent"]
assert not checkpoint["realUserDataModified"] and not checkpoint["localInstallerExecuted"]
write("archive-verification.json", {
    "result": "passed",
    "scope": "frozen exact-source cwd and no-script retest evidence only",
    "filesVerified": len(read("manifest.json")["entries"]),
    "nativeSource": source,
    "nativePackInstallCwd": "passed",
    "completeRuntimeInstall": "scripts_required; not accepted",
    "scriptsPermission": "pending",
    "scriptsExecuted": False,
    "formalTaskReceipt": False,
    "fullNativeGateAccepted": False,
    "performanceAccepted": False,
    "realCyclesAccepted": False,
    "existingPerformanceFailureCleared": False,
    "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"])}))
