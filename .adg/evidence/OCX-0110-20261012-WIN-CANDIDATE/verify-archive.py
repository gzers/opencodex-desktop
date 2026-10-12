"""Verify frozen candidate observations, never grant TASK or native acceptance."""

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
            {
                "path": path.relative_to(ROOT).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

manifest = read("manifest.json")
for entry in manifest["entries"]:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    payload = path.read_bytes()
    assert len(payload) == entry["bytes"], entry["path"]
    assert hashlib.sha256(payload).hexdigest() == entry["sha256"], entry["path"]

source = "01a1d85504781f4cb189a68ed85d1d803a1409ff"
ci = read("candidate-ci-final.json")
assert ci["run"]["head_sha"] == source
assert ci["run"]["status"] == "completed" and ci["run"]["conclusion"] == "success"
assert all(job["conclusion"] == "success" for job in ci["jobs"])
downloads = read("download-verification.json")
assert len(downloads) == 2
assert all(
    item["digestMatched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"]
    and item["sourceCommit"] == source
    for item in downloads
)
packages = read("local-package-integrity.json")
receipt = read("ci-installer-verification.json")
assert receipt["sourceCommit"] == source and receipt["result"] == "pass"
assert receipt["version"] == "0.1.10" and not receipt["releaseAccepted"]
for package in packages:
    expected = next(item for item in receipt["installers"] if item["kind"] == package["kind"])
    assert package["sha256"] == expected["sha256"] and package["matchesCi"]
candidate = read("candidate-msi-identity.json")
app = next(item for item in candidate["extractedExecutables"] if item["name"] == "opencodex-desktop.exe")
assert app["sha256"] == receipt["installers"][0]["installedExecutableSha256"]
baseline = read("baseline019-msi-readonly.json")
assert baseline["result"] == "pass" and not baseline["installerExecuted"]
assert baseline["packageSha256"] == "b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"
daily = read("daily-msi-readonly/msi-readonly.json")
assert candidate["properties"]["UpgradeCode"] == daily["properties"]["UpgradeCode"]
assert candidate["properties"]["ALLUSERS"] == daily["properties"]["ALLUSERS"] == "1"
assert read("baseline-extract-initial-failure.json")["result"] == "fail"
checkpoint = read("local-checkpoint.json")
assert not checkpoint["taskAuthorityPresent"]
assert not checkpoint["localInstallerExecuted"] and not checkpoint["realDataModified"]
assert checkpoint["localCandidateApplicationProcesses"] == []
assert checkpoint["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert read("rustfmt-check.json")["checkExitCode"] == 0
write(
    "archive-verification.json",
    {
        "result": "passed",
        "scope": "frozen evidence integrity, source identity and package observations only",
        "filesVerified": len(manifest["entries"]),
        "candidateBuildSource": source,
        "formalTaskReceipt": False,
        "nativeGateAccepted": False,
        "installerExecutedLocally": False,
        "performanceAccepted": False,
        "realCycleObservationStarted": False,
        "originalPerformanceFailureCleared": False,
        "isolationFixAcceptance": "pending exact-source CI and native tests",
    },
)
print(json.dumps({"archiveIntegrity": "passed", "files": len(manifest["entries"])}))
