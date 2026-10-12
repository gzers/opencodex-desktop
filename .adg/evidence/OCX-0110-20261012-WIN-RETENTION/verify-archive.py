"""Verify recorded partial native results; never grant full gate acceptance."""

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
    assert len(payload) == entry["bytes"]
    assert hashlib.sha256(payload).hexdigest() == entry["sha256"], entry["path"]

source = "4f7f33435ec1a7032e468363f191ddce50330fd8"
for name in ["4f-main-ci-final.json", "4f-candidate-run-final.json"]:
    run = read(name)["run"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == "success"
assert all(
    item["sourceCommit"] == source and item["matched"]
    and item["expectedDigest"] == "sha256:" + item["sha256"]
    for item in read("4f-download-verification.json")
)
launch = read("4f-native-launch-input.json")
assert launch["ciInstalledExeMatched"] and not launch["installerExecutedLocally"]
assert launch["executableSha256"] == "5e17773983bfcbdd741121210170a95922a436b4b6b9b783b12eca2062a5f6dc"
tree = read("4f-native-tree-alignment-visible.json")
assert tree["sourceCommit"] == source
assert tree["result"]["dpr"] == 2 and tree["result"]["pinSurvivedRestart"]
assert len(tree["result"]["rows"]) == 3
for row in tree["result"]["rows"]:
    assert row["purposeCenter"] == row["lineCenter"]
    assert all(control["offsetFromFirstLine"] == 0 for control in row["controls"])
    assert all(control["height"] >= 34 for control in row["controls"] if "btn" in control["className"])
assert read("4f-native-metrics.json")["dpi"] == 192
recent = read("recent-native-preparation.json")["result"]
assert recent["recentManualCount"] == 14 and not recent["preview"]["candidateIds"]
manual = read("historical-native-cleanup.json")["result"]
assert manual["staleRejected"] and len(manual["removed"]) == 3
assert manual["ordinaryManualRetained"] == 10 and manual["originalPinRetained"]
restored = read("historical-fixture-restoration.json")
assert set(restored["removedByNativeCleanup"]) == set(manual["removed"])
assert len(restored["survivingOriginalManifestsRestored"]) == 11
stage = read("automatic-native-policy-stage.json")["result"]
assert stage["policySaveDidNotDelete"] and stage["before"] == stage["after"]
automatic = read("automatic-native-cleanup.json")["result"]
assert len(automatic["created"]["removedIds"]) == 4 and automatic["created"]["cleanupError"] is None
assert automatic["newlySuccessfulBackupRetained"] and automatic["originalPinRetained"]
assert read("automatic-test-final-policy.json")["result"]["policy"]["mode"] == "manual"
corrupt = read("corrupted-native-restore-rejected.json")["result"]
assert corrupt["rejected"] and corrupt["pinRetained"] and corrupt["before"] == corrupt["after"]
assert read("corrupt-fixture-file-verification.json")["payloadRestored"]
assert read("corrupt-fixture-file-verification.json")["activePreferencesUnchanged"]
binding = read("custom-root-reference-switch.json")["result"]
assert not binding["admissionAccepted"] and binding["switched"]["blocked"] == "runtime_unverified"
install = read("panel282-native-install-summary.json")
assert install["sourceCommit"] == source and install["state"] == "failed"
assert install["listenerRemoved"] and install["sourceKindAfter"] == "unresolved"
assert not install["officialUpdateCommandExecuted"]
repro = read("npm-path-red-green.json")
assert repro["rows"][0]["result"] == "failure" and repro["rows"][1]["result"] == "success"
rejection = read("junction-approval-rejection.json")
assert rejection["approvalReview"] == "rejected" and not rejection["executed"]
assert not rejection["workaroundAttempted"] and rejection["gateResult"] == "not_run"
assert not read("checkpoint.json")["task182Present"]
assert not read("checkpoint.json")["realUserDataModified"]
write(
    "archive-verification.json",
    {
        "result": "passed",
        "scope": "archive integrity and explicitly scoped native observations",
        "filesVerified": len(manifest["entries"]),
        "nativeSource": source,
        "treeWebViewFirstLineGeometry": "three recorded rows passed; interaction matrix pending",
        "retentionCommands": "passed with synthetic dates; real duration not observed",
        "corruptRestoreRejection": "passed for recorded payload corruption",
        "customRootSwitch": "blocked/runtime_unverified; not accepted",
        "nativePanelInstall": "failed; fix pending",
        "npmFixSource": "ee65f1158c4f0bb7c283ff09e24120a7884472d0",
        "junctionEscapeGate": "not_run; automatic approval rejected the fixture",
        "formalTaskReceipt": False,
        "fullNativeGateAccepted": False,
        "performanceAccepted": False,
        "realCyclesAccepted": False,
        "existingPerformanceFailureCleared": False,
        "releaseAccepted": False,
    },
)
print(json.dumps({"archiveIntegrity": "passed", "files": len(manifest["entries"])}))
