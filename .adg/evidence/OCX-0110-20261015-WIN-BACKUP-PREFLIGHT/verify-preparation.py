"""Verify this historical no-launch preparation; future foreground records are separate evidence."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
review = read("review-only.final.original.json")
guards = read("authorization-guards.fixed.original.json")
post = read("preparation-postcheck.json")
assert review["source"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
assert review["sha256"] == "f43e81a9d96f25a29a937a54d2c779ce2459c3ecb30a73b131c57729f70cb929"
assert review["mode"] == "ordinary" and review["nativeCountBefore"] == 0
assert review["harnessToken"]["elevated"] is False and review["harnessToken"]["elevationType"] == 3
assert review["dailyExeUnchanged"] and review["protectedSandboxUnchanged"]
for key in ("defaultReviewStartsApplication", "installerExecuted", "foregroundInputIssued",
            "runtimeCreated", "dpiChangePlanned", "forcedScalePlanned", "cdpUsedForUiInput",
            "backupsPreparedOrAgeAdjusted"):
    assert not review[key]
assert not review["autoUpdate"] and not review["autoPanel"] and not review["startupCleanup"]
assert not review["keepOnClose"] and review["initialTheme"] == "dark"
assert guards["validInputChecks"] == 1 and guards["negativeGuardChecks"] == 10 and guards["purelyInMemory"]
for key in ("actualForegroundPermissionGranted", "applicationStarted", "runtimeCreated",
            "preferencesWritten", "inputInjected", "syntheticAuthorizationFileWritten"):
    assert not guards[key]
error = read("guard-preparation-first-error.json")
assert error["status"] == "preparation-failed" and not error["applicationStarted"]
assert post["preparedPowerShellMethods"] == 4 and post["syntaxErrors"] == 0
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
for key in ("newRuntimeExists", "applicationStarted", "installerExecuted", "inputInjected",
            "preferencesWritten", "foregroundAuthorizationFileCreated", "actualForegroundPermissionGranted",
            "fullVersionGatePassed", "backupOrTreeNativeTestExecuted"):
    assert not post[key]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
manifest_path = root / "manifest.json"
report_path = root / "archive-verification.json"
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest = [
        {"path": path.name, "bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for path in sorted(root.iterdir(), key=lambda item: item.name)
        if path.is_file() and path.name not in {manifest_path.name, report_path.name}
    ]
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
else:
    manifest = read(manifest_path.name)
for row in manifest:
    path = root / row["path"]
    assert path.stat().st_size == row["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(manifest),
    "scope": "Historical no-launch preparation only; later native evidence must be verified separately.",
    "source": review["source"], "candidatePayloadHashVerified": True,
    "ordinaryHarnessTokenVerified": True, "ordinaryApplicationTokenVerified": False,
    "inMemoryPositiveAuthorizationGuardChecks": 1, "inMemoryNegativeAuthorizationGuardChecks": 10,
    "actualForegroundPermissionGranted": False, "nativeInstancesAtCheckpoint": 0,
    "newRuntimeCreated": False, "backupOrTreeNativeTestExecuted": False,
    "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_path.name)
print(json.dumps(report))
