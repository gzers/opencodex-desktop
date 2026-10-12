import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
result = read("real-npm-test-result.json")
assert result["sourceCommit"] == "4852820b89f7f9df7d54bc623e94f3f55d884023"
ci = json.loads((root.parent / "OCX-0110-20261012-WIN-QUERY-PROCESS-TREE/fix-final-test-executable-verification.json")
                .read_text(encoding="utf-8-sig"))
assert result["ciTestExecutableSha256"] == ci["sha256"]
assert result["status"] == "recorded-live-test-passed" and result["exitCode"] == 0
assert result["npmVersion"] == "10.9.0"
assert result["nodeSha256"] == result["originalNodeSha256"]
assert not result["credentialEnvironmentInherited"]
assert not result["applicationExecutableLaunched"] and not result["packageLifecycleScriptsExecuted"]
assert result["currentDesktopPidBefore"] == result["currentDesktopPidAfter"] == 38924
assert result["currentDesktopCountAfter"] == 1 and result["retainedOriginalDesktopLifecycle"]
assert result["home"].startswith(result["fixtureRoot"] + "\\")
assert result["test"] == "modules::about::remote::tests::live_remote_query_returns_version_and_integrity"
assert "1 passed; 0 failed; 0 ignored" in (root / "native-test-stdout.log").read_text(encoding="utf-8-sig")
assert (root / "native-test-stderr.log").read_text(encoding="utf-8-sig").strip() == ""
assert not read("initial-harness-error.json")["testProcessStarted"]
assert not read("second-harness-error.json")["testProcessStarted"]
check = read("independent-cleanup-and-cycle-check.json")
assert check["fixtureNodeProcessesStillPresent"] == 0 and not check["fixtureOwnHomeNpmrcPresent"]
assert check["cycleMonitorLiveHandleVerified"] and check["originalCycleScheduleUnchanged"]
assert check["desktopApplicationCount"] == 1 and not check["newMainNativeAcceptance"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".cab"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "one explicitly run production Rust live metadata query with isolated HOME and real npm CLI",
    "productionFunctionRealNpmQuery": "passed", "newCandidateWebviewAccepted": False,
    "installationAccepted": False, "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "liveQuery": "1/1", "files": len(entries)}))
