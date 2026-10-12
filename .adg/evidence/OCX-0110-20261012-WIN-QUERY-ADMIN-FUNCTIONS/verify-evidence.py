import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
expected_source = "4852820b89f7f9df7d54bc623e94f3f55d884023"
expected_exe = "097421b7971853e5ef38667100de6eee0b96366adfee5221dfb86ca640c4e23c"
expected_blob = "3dc48b4251cd8465f933bd59387e77dcfc8f339b"
ci = json.loads((root.parent / "OCX-0110-20261012-WIN-QUERY-PROCESS-TREE/fix-final-test-executable-verification.json")
                .read_text(encoding="utf-8-sig"))
assert ci["sha256"] == expected_exe and ci["sourceCommit"] == expected_source
for prefix, elevated, token_type in [("admin", True, 2), ("ordinary", False, 3)]:
    value = read(f"{prefix}-function-test-result.json")
    assert value["sourceCommit"] == expected_source and value["ciTestExecutableSha256"] == expected_exe
    assert value["testSourceBlob"] == expected_blob and value["exitCode"] == 0
    for token_name in ["helperToken", "testProcessToken"]:
        token = value[token_name]
        assert token["status"] == "observed"
        assert token["elevated"] is elevated and token["elevationType"] == token_type
        assert token["processId"] > 0 and token["creationTimeUtc"] and token["executable"]
    assert value["testProcessToken"]["processId"] != value["helperToken"]["processId"]
    assert value["desktopCountAfter"] == 1 and value["sameDesktopLifecycleRetained"]
    assert not value["credentialsInherited"] and not value["desktopApplicationLaunched"]
    assert not value["packageLifecycleScriptsExecuted"] and not value["securitySettingsChanged"]
    output = (root / f"{prefix}-test-stdout.log").read_text(encoding="utf-8-sig")
    assert "4 passed; 0 failed; 0 ignored" in output and "FAILED" not in output
    diagnostics = (root / f"{prefix}-test-stderr.log").read_text(encoding="utf-8-sig")
    assert "external fixture watchdog cleanup" not in diagnostics
    timeout = re.search(r"metadata fixture timeout: elapsed=([0-9.]+)s, result=Err\(Timeout\)", diagnostics)
    assert timeout and 2.0 <= float(timeout.group(1)) < 4.0
check = read("independent-post-test-verification.json")
assert check["desktopCount"] == 1 and check["originalDesktopPid"] == 38924
assert check["desktopTokenStillOrdinary"] and check["cycleMonitorLiveHandleConfirmed"]
assert check["scheduleOriginalBytesRetained"] and check["actualScheduleSha256"] == check["baselineScheduleSha256"]
assert check["helperTerminalConfirmed"] and not check["originalAdminHelperStillLive"]
assert not check["newCandidateDesktopStarted"] and not check["administratorDesktopUiAccepted"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, item):
    (root / name).write_bytes((json.dumps(item, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "same-host same CI Rust EXE query-function regressions with actual ordinary and administrator test-process tokens",
    "ordinaryFunctions": "4/4, false/type3", "administratorFunctions": "4/4, true/type2",
    "newCandidateDesktopUiAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "ordinary": "4/4", "administrator": "4/4", "files": len(entries)}))
