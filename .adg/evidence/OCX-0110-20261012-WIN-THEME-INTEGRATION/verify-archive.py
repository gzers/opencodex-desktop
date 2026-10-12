"""Verify frozen merge/source/download evidence; exact merged tests still pending."""

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

theme = "cf71ea4f84b82b2e4c1bb3a15de88fd5815dc066"
for name in ["theme-run-38211754315-final.json", "theme-run-38211818636-final.json"]:
    run = read(name)["run"]
    assert run["head_sha"] == theme and run["status"] == "completed" and run["conclusion"] == "success"
downloads = read("theme-download-verification.json")
assert len(downloads) == 2 and all(
    item["sourceCommit"] == theme and item["matched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"] for item in downloads
)
merge = read("integrated-dispatch.json")
assert merge["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert merge["parents"] == ["e1a1aaa54ccc56530d5da507d0666b6ec8d2c138", theme]
assert merge["dispatchHttpStatus"] == 204 and not merge["scriptsAuthorized"]
assert all(run["head_sha"] == merge["sourceCommit"] for run in read("integrated-runs-initial.json"))
checkpoint = read("checkpoint.json")
assert not checkpoint["task182Present"] and not checkpoint["scriptPermissionProvided"]
assert not checkpoint["scriptsExecuted"] and not checkpoint["realDataModified"]
write("archive-verification.json", {
    "result": "passed", "scope": "merge/source/download frozen observations only",
    "filesVerified": len(read("manifest.json")["entries"]),
    "integratedSource": merge["sourceCommit"], "mergedExactCiAccepted": False,
    "mergedNativeThemeVerified": False, "formalTaskReceipt": False,
    "scriptsExecuted": False, "scriptsSecondConfirmation": "pending",
    "performanceAccepted": False, "realCyclesAccepted": False,
    "existingPerformanceFailureCleared": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"])}))
