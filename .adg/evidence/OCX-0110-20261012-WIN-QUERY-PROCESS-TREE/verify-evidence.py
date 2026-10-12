"""Verify the exact source comparison and actual process-test subset, not all gates."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
fix = "4852820b89f7f9df7d54bc623e94f3f55d884023"
red = "2218b8c94187efbb5139e33e312676d7beffafb2"
comparison = read("final-comparison-and-integration.json")
assert comparison["redSource"] == red and comparison["fixSource"] == fix
assert comparison["redTestBlob"] == comparison["fixTestBlob"] == "3dc48b4251cd8465f933bd59387e77dcfc8f339b"
assert comparison["mainHead"] == comparison["originMainHead"] == fix
assert comparison["mainIntegratedWithoutConflict"] and comparison["codeWorktreeClean"]
assert comparison["runningApplicationSource"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert comparison["candidateApplicationNotYetLaunched"] and not comparison["fullNativeGateAccepted"]
expected_diff = {
    "M\tapps/desktop/tauri/Cargo.toml",
    "M\tapps/desktop/tauri/src/infrastructure/mod.rs",
    "A\tapps/desktop/tauri/src/infrastructure/windows_query_job.rs",
    "M\tapps/desktop/tauri/src/modules/about/remote.rs",
}
assert set(comparison["comparisonDiff"]) == expected_diff
summary = read("ci-windows-summary-recalculation.json")
for label, source, result in [("red-final", red, "failure"), ("fix-final", fix, "success")]:
    run = read(label + "-ci-final.json")
    jobs = read(label + "-jobs-final.json")
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == result
    windows = [job for job in jobs["jobs"] if job["name"] == "backend-windows"]
    assert len(windows) == 1 and windows[0]["status"] == "completed" and windows[0]["conclusion"] == result
    if label == "fix-final":
        assert all(job["status"] == "completed" and job["conclusion"] == "success"
                   for job in jobs["jobs"] if job["name"] != "build")
        assert summary[label]["groups"] == 34 and summary[label]["passed"] == 598
        assert summary[label]["failed"] == 0 and summary[label]["ignored"] == 7
    download = read(label + "-download-verification.json")
    executable = read(label + "-test-executable-verification.json")
    assert download["sourceCommit"] == source and download["verified"]
    assert download["apiDigest"] == "sha256:" + download["sha256"]
    assert executable["sourceCommit"] == source and executable["ciTestExecutableReceiptMatched"]
    local = read(label + "-local-result.json")
    assert local["sourceCommit"] == source and local["ciTestExeSha256"] == executable["sha256"]
    assert local["testBlob"] == comparison["fixTestBlob"]
    assert not local["applicationExecutableLaunched"] and local["runningDesktopPidUnchanged"] == 38924
    text = (root / (label + "-local.log")).read_text(encoding="utf-8-sig")
    if label == "red-final":
        assert local["exitCode"] == 101
        assert "0 passed; 4 failed" in text
        for mode in ["nonzero", "overflow", "success"]:
            assert f"metadata query left its own {mode} descendant running" in text
        assert "query required external fixture watchdog cleanup" in text
    else:
        assert local["exitCode"] == 0 and "4 passed; 0 failed" in text
        assert "external fixture watchdog cleanup" not in text
        match = re.search(r"metadata fixture timeout: elapsed=([0-9.]+)s, result=Err\(Timeout\)", text)
        assert match and 2.0 <= float(match.group(1)) < 4.0
assert read("red-safe-local-overflow-result.json")["exitCode"] == 101
assert read("fix-initial-local-result.json")["exitCode"] == 0
dispatch = read("main-candidate-dispatch.json")
assert dispatch["expectedHead"] == fix and dispatch["httpStatus"] == 204 and not dispatch["releasePublished"]
recalculated = {}
for label in ["red-first", "red-safe", "red-final", "fix-initial", "fix-final"]:
    text = (root / (label + "-ci-windows.log")).read_text(encoding="utf-8-sig")
    rows = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    assert summary[label]["groups"] == len(rows)
    for field, position in [("passed", 1), ("failed", 2), ("ignored", 3)]:
        assert summary[label][field] == sum(int(row[position]) for row in rows)
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".cab", ".crate"} for path in files)
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
    "result": "passed", "scope": "same test source Windows red/green process-query regressions, archive integrity and source integration",
    "productionRootSuspendedBeforeJobAssignment": "implemented and covered by recorded fixture behavior",
    "redSameHost": {"passed": 0, "failed": 4}, "fixedSameHost": {"passed": 4, "failed": 0},
    "exactFeatureCi": "success", "mainCiAndCandidate": "separate final audit required",
    "fullNativeGateAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "formalTaskReceipt": False, "released": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "red": "0/4", "fix": "4/4", "files": len(entries)}))
