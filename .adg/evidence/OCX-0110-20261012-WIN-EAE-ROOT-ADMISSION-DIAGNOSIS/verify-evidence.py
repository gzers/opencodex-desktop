"""A refused/unverified prerequisite diagnosis; never establish root-switch acceptance."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
native = read("current-native-status-and-root.json")
assert native["sourceCommit"] == source and native["applicationPid"] == 37276
assert native["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
r = native["result"]
assert r["runtime"] == {"kind": "unresolved", "version": None}
assert r["statusProjection"]["matrix"]["runtime"] == "not_found"
assert r["statusProjection"]["source"] == "live" and r["statusProjection"]["health"] == "unknown"
assert not r["statusProjection"]["hasProcessId"] and not r["statusProjection"]["hasPort"]
assert r["rootConfigUnchanged"] and r["beforeRoot"] == r["afterRoot"]
assert r["preferences"] == {"channel": "stable", "automaticCheck": True}
assert not r["dataRootSwitchInvoked"] and not r["manualUpdateCheckInvoked"] and not r["versionProbeInvoked"]
assert r["statusRefreshRequested"] and r["nativeCommandSuccessNotEquivalentToCollectorRefreshSuccess"]
files = read("independent-file-check.json")
for prefix in ["schedule", "preferences", "binding"]:
    assert files[prefix + "BeforeSha256"] == files[prefix + "AfterSha256"]
assert files["cycleScheduleBytesUnchanged"] and files["preferencesBytesUnchanged"] and files["rootBindingBytesUnchanged"]
assert not files["rootSwitchInvoked"] and not files["rootAdmissionAccepted"] and not files["manualUpdateCheckInvoked"]
primary = read("exact-source-admission-witness.json")
assert primary["sourceCommit"] == source and len(primary["witnesses"]) == 5
assert not primary["currentRootSuccessAccepted"]
assert primary["olderRefusalNotReclassifiedAsCurrentCommandAttempt"]
assert primary["noVerifierFlagOrEnumOverride"] and primary["requiresOriginalVerifiedIdleRuntimeGate"]
assert "inference" in primary["currentAdmissionConclusion"]
assert {w["function"] for w in primary["witnesses"]} == {
    "change_binding", "binding_runtime_blocked", "OfficialStatusSource::fetch_inner",
    "StatusCollector::refresh_with", "status_snapshot_from_collector",
}
for witness in primary["witnesses"]:
    assert witness["sourceCommit"] == source and witness["witnessLineNumbers"]
    assert len(witness["gitBlob"]) == 40 and len(witness["rawBlobSha256"]) == 64
    assert not witness["sourceCopiedIntoDocs"]
check = read("current-lifecycle-check.json")
assert check["sourceCommit"] == source and check["applicationPid"] == 37276
assert check["monitorPid"] == 3920 and check["monitorExecSession"] == 46484
assert check["liveApplicationAndMonitorHandlesVerified"] and check["applicationCount"] == 1
assert check["monitorStatus"] == "observing-real-deadlines"
assert check["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(check[name] for name in [
    "cycleAccepted", "dataRootSwitchAccepted", "realDataMigrated",
    "scriptSecondConfirmationReceived", "fullGateAccepted", "releaseAccepted",
])
excluded = {"manifest.json", "archive-verification.json"}
paths = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".cab", ".zip"} for path in paths)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in paths]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source,
    "scope": "actual final-native status projection and exact-source admission-precondition diagnosis; no root transaction performed",
    "rootSwitchAccepted": False, "currentRawCollectorRefreshResult": "not exposed by snapshot command",
    "admissionInference": "unresolved source does not establish successful verified-idle refresh",
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "rootSwitch": "not attempted or accepted", "files": len(entries)}))
