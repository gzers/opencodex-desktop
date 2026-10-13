"""Verify the tool and current pending snapshot; not cycle acceptance."""
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
snapshot = root / "actual-background-check-20261013-0839"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
result = read(snapshot / "audit-result.json")
initial = read(snapshot / "native-initialization.json")
baseline = read(snapshot / "schedule-baseline.json")
prefix = (snapshot / "observations-prefix.ndjson").read_bytes()
observations = [json.loads(line) for line in prefix.splitlines() if line.strip()]
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert result["sourceCommit"] == source and result["applicationPid"] == 37276
assert result["status"] == "pending_original_deadlines" and result["observationsAudited"] == len(observations) == 117
assert result["allThreeOriginalHandlesCurrentlyLive"]
assert {value["pid"] for value in result["currentActualProcesses"]} == {37276, 24300, 3920}
assert all(value["actualLiveHandle"] and value["exactCreationMatched"] for value in result["currentActualProcesses"])
assert not any(result[key] for key in ["rawNetworkOrQueryCountAccepted", "earliestQueryStartElapsedProven",
    "automaticTriggerProven", "originalTaskRequiredChecksRecovered", "realFullCycleAccepted",
    "pairedPerformanceAccepted", "fullNativeGateAccepted", "formalTaskReceipt", "releaseAccepted",
    "ownerTerminalToolResultVerified", "monitorCleanupPreferenceRestored"])
for name, digest in result["rawSnapshotHashes"].items():
    assert hashlib.sha256((snapshot / name).read_bytes()).hexdigest() == digest
assert (snapshot / "native-initialization.json").read_bytes() == (
    root.parent / "OCX-0110-20261012-WIN-EAE-REAL-STABLE-PANEL-CYCLE/native-initialization.json").read_bytes()
assert (snapshot / "schedule-baseline.json").read_bytes() == (
    root.parent / "OCX-0110-20261012-WIN-EAE-REAL-STABLE-PANEL-CYCLE/schedule-baseline.json").read_bytes()
spec = importlib.util.spec_from_file_location("real_cycle_auditor", root / "methods/audit-cycle.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
recomputed = module.evaluate(initial, baseline, observations)
assert all(result[key] == value for key, value in recomputed.items())
assert module.self_test(initial, baseline, observations) == read(snapshot / "validator-self-test.json")
assert all(value["validatorRejected"] and value["nativeEvidence"] is False
           for value in read(snapshot / "validator-self-test.json")["tests"])
# Importing the pure validator must not create runtime snapshots or modify product state.
files = sorted(path for path in root.rglob("*") if path.is_file()
               and path.name not in {"manifest.json", "archive-verification.json"}
               and "__pycache__" not in path.parts)
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pending"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read(root / "manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "read-only terminal-audit tooling and117 actual pending observations; strict validator guards, no product cycle acceptance",
    "actualAuditStatus": result["status"], "originalLiveHandlesVerified": True,
    "validatorOnlySyntheticRefusals": 3, "realResponsesOrClocksInjected": False,
    "foregroundInputsPerformed": False, "originalTimerLimitExtended": False,
    "rawNetworkOrQueryCountsAccepted": False, "automaticTriggerProven": False,
    "realFullCycleAccepted": False, "pairedPerformanceAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "actualCycleAudit": result["status"],
                  "observations": len(observations), "fullCycleAccepted": False}))
