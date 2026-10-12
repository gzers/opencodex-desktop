"""Frozen observations only: cannot accept TASK, version, performance or cycles."""

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
    files = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in {"manifest.json", "archive-verification.json"}:
            continue
        assert path.suffix.lower() not in {".exe", ".msi", ".cab", ".zip", ".dll"}
        data = path.read_bytes()
        files.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": files})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

manifest = read("manifest.json")
for item in manifest["entries"]:
    path = ROOT / item["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    data = path.read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"], item["path"]

source = "0a83761a99bdcb721d929f33e4c60c2e95eb6224"
for run in ["0a-run-38202731337-final.json", "0a-run-38203025770-final.json"]:
    value = read(run)["run"]
    assert value["head_sha"] == source and value["conclusion"] == "success"
assert read("0a-main-ci-checkpoint.json")["conclusion"] == "success"
assert all(
    item["sourceCommit"] == source and item["matched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"]
    for item in read("0a-download-verification.json")
)
failed = read("ordinary-first-start-failed/session.json")
assert failed["status"] == "failed" and failed["applicationToken"]["elevated"] is False
live = read("native/ordinary-live-session.json")
assert live["sourceCommit"] == source and live["sourceCommitMatched"]
assert live["applicationToken"]["elevated"] is False
assert live["applicationToken"]["elevationType"] == 3
metrics = read("native/ordinary-native-metrics.json")
active = read("native/ordinary-active-snapshot.json")["result"]
assert metrics["sourceCommit"] == source and metrics["dpi"] == 192
assert metrics["backdropAttribute"] == 2 and metrics["dwmResult"] == 0
assert active["dpr"] == 2 and active["appearance"]["material"] == "mica"
restore = read("native/ordinary-pinned-different-value-restore.json")["result"]
assert restore["beforeTheme"] == "light" and restore["afterTheme"] == "system"
assert restore["pinnedBeforeRestore"] and restore["pinnedAfterRestore"]
assert not restore["protection"]["protected"]
assert not restore["restored"]["protectionReconciliationPending"]
fault = read("native/ordinary-backup-failure-object-isolation.json")["result"]
recovery = read("native/ordinary-backup-exact-recovery.json")["result"]
assert fault["failureId"] == recovery["failureId"]
assert not next(item for item in fault["afterCheck"]["items"] if item["id"] == fault["failureId"])["resolved"]
assert recovery["recovered"]["resolved"]
assert read("native/ordinary-backup-fault-fixture.json")["preferencesUnchanged"]
isolation = read("native/ordinary-channel-panel-file-verification.json")
assert isolation["querySucceeded"] and isolation["panelStatePreserved"] and isolation["panelCachePreserved"]
assert isolation["panelBefore"] == isolation["panelAfter"]
assert isolation["panelBefore"]["next_due"] - isolation["panelBefore"]["last_success"] == 86400
red = read("tree-alignment/v3/red/geometry.json")
green = read("tree-alignment/final/geometry.json")
assert red["result"] == "failed" and "offsetFromFirstLine\":6" in red["error"].replace(" ", "")
assert green["result"] == "passed" and len(green["cases"]) == 8
for case in green["cases"]:
    assert all(
        abs(control["offsetFromFirstLine"]) <= 1
        for row in case["rows"]
        for control in row["controls"]
    )
checkpoint = read("current-checkpoint.json")
assert not checkpoint["task182Present"] and not checkpoint["localInstallerExecuted"]
assert not checkpoint["realDataModified"]
write(
    "archive-verification.json",
    {
        "result": "passed",
        "scope": "archive identity and stated partial observations only",
        "filesVerified": len(manifest["entries"]),
        "nativeSource": source,
        "treeFixMain": "4f7f33435ec1a7032e468363f191ddce50330fd8",
        "ordinaryNativeObservation": "partial DPI192/DPR2 and Mica",
        "differentValueRestoreCommand": "passed; UI confirmation pending",
        "backupEventRecovery": "passed for recorded failure and exact retry",
        "panelCompletedQueryIsolation": "passed; in-flight coverage pending",
        "treeBrowserGeometry": "8 passed; physical WebView recheck pending",
        "formalTaskReceipt": False,
        "fullNativeGateAccepted": False,
        "performanceAccepted": False,
        "realCyclesAccepted": False,
        "originalPerformanceFailureCleared": False,
        "releaseAccepted": False,
    },
)
print(json.dumps({"archiveIntegrity": "passed", "files": len(manifest["entries"])}))
