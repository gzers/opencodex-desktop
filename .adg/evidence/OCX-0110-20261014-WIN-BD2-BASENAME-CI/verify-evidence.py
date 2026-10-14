"""Verify e6 terminal CI while retaining the PR failure as a diagnostic fact."""
import gzip
import hashlib
import json
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
admission = read("ci-admission.json")
assert admission["source"] == "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
assert admission["diagnosticOnly"] and not admission["releaseAdmission"]
assert admission["allCiPassed"] is False and admission["newCiFailureRetained"]
assert len(admission["runs"]) == 2
for row in admission["runs"]:
    assert row["head"] == admission["source"] and row["status"] == "completed"
    assert row["actualTree"] == "de7d465c91d91d28fa4c924eb617250c522a01e9"
    assert row["originalLogsVerified"] and not row["unexpectedFailedTests"]
    expected = "failure" if row["event"] == "pull_request" else "success"
    assert row["conclusion"] == expected
    summary = read(f"ci-log-summary-{row['runId']}.json")
    archive = root / f"ci-original-logs-{row['runId']}.zip"
    raw = archive.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == summary["archiveSha256"]
    assert summary["zipCrcPassed"] and summary["safeEntries"]
    assert summary["collectorRecordOrScopeTestsExecutedByCi"] is False
    if expected == "failure":
        assert len(summary["failedTestLines"]) == 3
        assert summary["rustCounts"]["backend-windows"] == {
            "suites": 1, "passed": 527, "failed": 3, "ignored": 4}
    else:
        assert not summary["failedTestLines"]
        assert summary["rustCounts"]["backend-windows"] == {
            "suites": 35, "passed": 627, "failed": 0, "ignored": 7}
    with zipfile.ZipFile(archive) as bundle:
        assert bundle.testzip() is None
        for entry in bundle.infolist():
            path = PurePosixPath(entry.filename.replace("\\", "/"))
            assert not path.is_absolute() and ".." not in path.parts
print(json.dumps({"archiveIntegrity": "passed", "diagnosticOnly": True,
                  "pushSuccess": True, "prFailureRetained": True,
                  "collectorLiveRecordTested": False}))
