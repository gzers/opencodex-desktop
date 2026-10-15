"""Recheck exact run identities and original ZIP members; CI does not accept native gates."""
import gzip
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
merge = "23e55c8def4fbf4bd18f0022b771e604a4be72be"
proof = read("actual-checkout-profile-proof.json")
assert proof["source"] == source and proof["prActualCheckout"] == merge
assert proof["wholeTreesEqual"] and proof["tree"] == "cd5f3dcf1a9240bea62aa2c6458b4353d367e0b8"
for run_id, expected in ((38670771363, source), (38670774817, merge)):
    terminal = read(f"ci-terminal-{run_id}.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    assert run["id"] == run_id and run["head_sha"] == source and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert run["event"] == ("push" if expected == source else "pull_request")
    assert {job["name"] for job in jobs} == {
        "backend", "backend-windows", "frontend", "windows-smoke-evidence", "release-tools", "build"}
    for job in jobs:
        assert job["status"] == "completed"
        assert job["conclusion"] == ("skipped" if job["name"] == "build" else "success")
    summary = read(f"ci-log-summary-{run_id}.json")
    archive_path = root / f"ci-original-logs-{run_id}.zip"
    raw = archive_path.read_bytes()
    assert len(raw) == summary["archiveBytes"] and sha(raw) == summary["archiveSha256"]
    assert summary["associatedHead"] == source and not summary["failedTestLines"]
    assert not summary["originalBd2FailedRunCleared"]
    assert not summary["collectorRecordOrScopeTestsExecutedByCi"]
    assert summary["rustCounts"] == {
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6}}
    profile = next(row for row in proof["runs"] if row["runId"] == run_id)
    assert profile["profileContractChecks"] == 31
    assert profile["runnerMethodMatchesSourceWithCheckoutLineEndings"]
    assert all(value == expected for value in profile["actualCheckout"].values())
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        seen = set()
        for member in archive.infolist():
            name = member.filename.replace("\\", "/")
            path = PurePosixPath(name)
            assert not path.is_absolute() and ".." not in path.parts and ":" not in name
            assert name.casefold() not in seen and (member.external_attr >> 16) & 0o170000 != 0o120000
            seen.add(name.casefold())
        for row in summary["selectedOriginalLogs"] + profile["selectedOriginalLogs"]:
            data = archive.read(row["member"])
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
            assert gzip.decompress((root / row["savedAs"]).read_bytes()) == data
            if row["member"].endswith("_Run actions_checkout@v5.txt"):
                lines = data.decode("utf-8-sig").splitlines()
                positions = [i for i, line in enumerate(lines) if " log -1 --format=%H" in line]
                assert len(positions) == 1
                assert re.search(r"\b([0-9a-f]{40})\b", lines[positions[0] + 1]).group(1) == expected
            if "Verify isolated acceptance Profile admission" in row["member"]:
                text = data.decode("utf-8-sig")
                assert re.search(r'"checksPassed":\s*31', text)
                assert profile["runnerMethodSha256"] in text
                assert '"nativeProfileProbeExecuted": false' in text
after, request = read("pr-after-update.original.json"), read("pr-update-request.json")
assert all(after[key] == value for key, value in request.items())
assert after["head"]["sha"] == source and after["base"]["sha"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert after["state"] == "open" and after["draft"] and not after["merged"] and after["auto_merge"] is None
excluded = {"manifest.json", "archive-verification.json"}
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": sha(path.read_bytes())}
           for path in sorted(root.rglob("*"), key=lambda path: path.relative_to(root).as_posix())
           if path.is_file() and path.name not in excluded]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists() and not (root / "archive-verification.json").exists()
    (root / "manifest.json").write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source,
    "scope": "Exact helper head CI terminal and original logs only",
    "runIds": [38670771363, 38670774817], "bothCiRunsSucceeded": True,
    "requiredJobsPerRun": 5, "mainOnlyBuildSkipped": True,
    "pushActualCheckout": source, "prActualCheckout": merge, "wholeTreesEqual": True,
    "windowsRustPassedPerRun": 627, "windowsRustIgnoredPerRun": 7,
    "macRustPassedPerRun": 900, "macRustIgnoredPerRun": 6,
    "rustSuitesPerPlatformPerRun": 35, "profileContractChecksPerRun": 31,
    "originalZipAndSelectedLogBytesVerified": True, "prDraftUnmerged": True,
    "nativeTestProfilePositiveAdmissionVerified": False, "historicalFailuresCleared": False,
    "newDesktopCandidateAdmitted": False, "installerRetentionVerified": False,
    "pairedPerformanceVerified": False, "realCycleVerified": False, "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("archive-verification.json") == report
print(json.dumps(report))
