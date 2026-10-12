"""Verify source-bound Windows version-probe tests, not the complete native gate."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
red = "277f76251b2db5401c152b3517bf4e539224a75c"
fixed = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
comparison = read("atomic-source-comparison-before-tests.json")
assert comparison["redSource"] == red and comparison["fixedSource"] == fixed
assert comparison["onlyChangedPath"] == "apps/desktop/tauri/src/infrastructure/official_version_source.rs"
assert comparison["sharedAtomicFixtureBlob"] == "08752751dd626977f09dc2b73db42fd075d3b661"
assert comparison["productionVersionDeadlineSeconds"] == 2
diagnosis = read("fixture-publication-diagnosis.json")
assert diagnosis["oldLogsPreserved"] and not diagnosis["assertionsRemoved"] and not diagnosis["budgetChanged"]
assert diagnosis["classification"] == "fixture publication race, not metadata cleanup regression"
old_metadata = (root / "probe-red-ordinary-metadata-stderr.log").read_text(encoding="utf-8-sig")
assert "ParseIntError { kind: Empty }" in old_metadata
old_version = (root / "probe-red-ordinary-version-stderr.log").read_text(encoding="utf-8-sig")
assert "version query left its own nonzero descendant running" in old_version
assert "version query left its own success descendant running" in old_version
assert "query required external fixture watchdog cleanup" in old_version
for label, source, expected_ci in [
    ("probe-atomic-red", red, "failure"),
    ("probe-atomic-fixed", fixed, "success"),
]:
    run = read(label + "-ci-final.json")
    jobs = read(label + "-jobs-final.json")["jobs"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == expected_ci
    windows = [job for job in jobs if job["name"] == "backend-windows"]
    assert len(windows) == 1 and windows[0]["conclusion"] == expected_ci
    if expected_ci == "success":
        assert all(job["status"] == "completed" and job["conclusion"] == "success"
                   for job in jobs if job["name"] != "build")
    download = read(label + "-download-verification.json")
    exe = read(label + "-test-executable-verification.json")
    assert download["sourceCommit"] == source and download["verified"]
    assert download["apiDigest"] == "sha256:" + download["sha256"]
    assert exe["sourceCommit"] == source and exe["ciTestExecutableReceiptMatched"]
    assert exe["artifactId"] == download["artifactId"] and not exe["applicationExecutableLaunched"]
    for mode in (["ordinary", "administrator"] if label == "probe-atomic-fixed" else ["ordinary"]):
        for group in ["version", "metadata"]:
            value = read(f"{label}-{mode}-{group}-result.json")
            assert value["sourceCommit"] == source and value["ciTestExecutableSha256"] == exe["sha256"]
            assert value["requestedMode"] == mode and value["productionVersionDeadlineSeconds"] == 2
            assert not value["credentialsInherited"] and not value["desktopApplicationLaunched"]
            assert value["desktopCountAfter"] == 1 and value["sameOriginalDesktopLifecycle"]
            for token_name in ["helperToken", "testProcessToken"]:
                token = value[token_name]
                assert token["status"] == "observed"
                assert token["elevated"] is (mode == "administrator")
                assert token["elevationType"] == (2 if mode == "administrator" else 3)
            stdout = (root / f"{label}-{mode}-{group}-stdout.log").read_text(encoding="utf-8-sig")
            stderr = (root / f"{label}-{mode}-{group}-stderr.log").read_text(encoding="utf-8-sig")
            assert "ParseIntError" not in stderr
            if label == "probe-atomic-red" and group == "version":
                assert value["exitCode"] == 101 and "0 passed; 3 failed" in stdout
                assert "version query left its own nonzero descendant running" in stderr
                assert "version query left its own success descendant running" in stderr
                assert "query required external fixture watchdog cleanup" in stderr
            else:
                count = 3 if group == "version" else 4
                assert value["exitCode"] == 0 and f"{count} passed; 0 failed" in stdout
                assert "query required external fixture watchdog cleanup" not in stderr
                match = re.search(rf"query fixture {group}/timeout: elapsed=([0-9.]+)s, result=Timeout", stderr)
                assert match and 2.0 <= float(match.group(1)) < 4.0
summary = read("windows-test-summary-recalculation.json")
for label in ["probe-red", "probe-atomic-red", "probe-atomic-fixed"]:
    text = (root / (label + "-ci-windows.log")).read_text(encoding="utf-8-sig")
    rows = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    assert summary[label]["groups"] == len(rows)
    for name, position in [("passed", 1), ("failed", 2), ("ignored", 3)]:
        assert summary[label][name] == sum(int(row[position]) for row in rows)
assert summary["probe-atomic-red"]["failed"] == 3
assert summary["probe-atomic-fixed"]["failed"] == 0
check = read("final-test-lifecycle-and-source-check.json")
assert check["redSource"] == red and check["fixedSource"] == fixed
assert check["versionTestsIdentical"] and check["sharedAtomicFixtureIdentical"]
assert check["runtimeDefaultsUnchanged"] and check["productionVersionDeadlineSeconds"] == 2
assert check["applicationPid"] == 38924 and check["applicationCount"] == 1
assert check["originalApplicationAndMonitorLiveHandlesVerified"]
assert check["runningApplicationSource"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert check["originalCycleScheduleBytesUnchanged"] and not check["newDesktopCandidateLaunched"]
# Literal daily identity remains independent of the helper test executable.
assert check["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not check["fullNativeGateAccepted"] and not check["pairedPerformanceAccepted"]
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
    "result": "passed", "scope": "same-source Windows version-probe red/green process ownership, preserved metadata regression, actual test-process tokens and archive integrity",
    "oldVersionProbe": "0/3", "fixedOrdinaryVersionProbe": "3/3",
    "fixedAdministratorVersionProbe": "3/3", "fixedMetadata": "4/4 in each actual-token mode",
    "fullNativeGateAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "versionRed": "0/3", "versionFix": "3/3 per mode", "files": len(entries)}))
