"""Cross-check extracted MSI bytes with candidate runner receipts; normalize only in memory."""
import hashlib
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
audit = read(root / "candidate-content-audit.json")
msi = read(root / "msi-readonly.json")
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert audit["sourceCommit"] == source and audit["runId"] == 38355803684
assert msi["result"] == "pass" and msi["databaseMode"] == "read-only" and not msi["installerExecuted"]
msi_package = next(item for item in audit["packages"] if item["kind"] == "msi")
assert msi["packageSha256"] == msi_package["sha256"]
exe_record = next(item for item in msi["extractedExecutables"] if item["name"] == "opencodex-desktop.exe")
exe = Path(exe_record["path"])
value = exe.read_bytes()
sha = lambda raw: hashlib.sha256(raw).hexdigest()
assert sha(value) == exe_record["sha256"] == msi_package["expectedInstalledExecutableSha256"]
assert exe_record["productVersion"] == msi["properties"]["ProductVersion"] == "0.1.10"
marker = b"__TAURI_BUNDLE_TYPE_VAR_MSI"
receipts = list((root / "artifact-raw").rglob("installer-verification.json"))
assert len(receipts) == 2 and receipts[0].read_bytes() == receipts[1].read_bytes()
verification = read(receipts[0])
offset = next(item["bundleMarkerOffset"] for item in verification["installers"] if item["kind"] == "msi")
assert value[offset:offset + len(marker)] == marker and value.count(b"__TAURI_BUNDLE_TYPE_VAR_UNK") == 0
normalized = bytearray(value)
normalized[offset:offset + len(marker)] = b"__TAURI_BUNDLE_TYPE_VAR_UNK"
assert normalized.count(b"__TAURI_BUNDLE_TYPE_VAR_UNK") == 1
assert sha(normalized) == audit["builtExecutableSha256"]
for entry in verification["installers"]:
    expected = bytearray(normalized)
    suffix = {"nsis": b"NSS", "msi": b"MSI"}[entry["kind"]]
    expected[offset:offset + len(marker)] = b"__TAURI_BUNDLE_TYPE_VAR_" + suffix
    assert offset == entry["bundleMarkerOffset"]
    assert sha(expected) == entry["installedExecutableSha256"] == entry["expectedInstalledExecutableSha256"]
    startup_path = next(path for path in (root / "artifact-raw").rglob("startup.json") if entry["kind"] + "-startup" in path.parts)
    startup = read(startup_path)
    assert startup["sha256"] == sha(expected) and startup["expectedCommit"] == source and startup["sourceCommitMatched"]
assert source.encode() in value
assert msi["properties"]["ALLUSERS"] == "1"
assert msi["properties"]["UpgradeCode"] == "{30FC0B19-5C02-5849-A234-D548E7AFC0DE}"
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
def blob(path):
    return subprocess.check_output(["git", "-C", str(code), "rev-parse", source + ":" + path]).decode().strip()
record = {
    "sourceCommit": source, "runId": audit["runId"], "result": "passed",
    "localExtractedExecutable": str(exe), "localExtractedExeSha256": sha(value),
    "ciInstalledMsiExeReceiptMatched": True, "ciUnpatchedBuiltExeSha256": audit["builtExecutableSha256"],
    "normalizedInMemoryHashMatchesCiBuiltExe": True, "bundleMarker": "MSI", "bundleMarkerOffset": offset,
    "bothPackagePatchHashesMatchRunnerReceipts": True, "embeddedFullCommitPresent": True,
    "updateCenterBlob": blob("apps/desktop/ui/src/features/updates/UpdateCenter.vue"),
    "updateCenterTestsBlob": blob("apps/desktop/ui/tests/update-center.test.ts"),
    "msiAllUsers": "1", "msiUpgradeCode": msi["properties"]["UpgradeCode"],
    "msiInstallOnDailyHostAdmitted": False, "normalizedExecutableWritten": False,
    "installerInvoked": False, "executableStarted": False, "foregroundInputIssued": False,
    "nativeFocusFixAccepted": False, "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "mainIntegrated": False, "originalFailedCiFailureCleared": False, "originalFailedCiCauseIsolated": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
}
(root / "offline-identity-result.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "offlineIdentity": "passed", "extractedMsiExeSha256": sha(value),
    "embeddedCommit": source, "normalizedExecutableWritten": False, "nativeFocusFixAccepted": False,
}))
