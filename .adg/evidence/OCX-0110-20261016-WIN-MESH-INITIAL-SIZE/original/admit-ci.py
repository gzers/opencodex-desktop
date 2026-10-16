"""Read and verify one completed CI; ANSI removal is parsing only."""
import hashlib
import importlib.util
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38891188021
spec = importlib.util.spec_from_file_location(
    "ci_api", Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
                   "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
assert run["head_sha"] == SOURCE and run["status"] == "completed"
assert run["conclusion"] == "success" and run["run_attempt"] == 1
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
expected = {
    "frontend": 116940815709, "backend": 116940815459,
    "backend-windows": 116940815642, "release-tools": 116940815575,
    "windows-smoke-evidence": 116940815876,
}
job_rows = []
for name, jid in expected.items():
    job, = [job for job in jobs["jobs"] if job["id"] == jid]
    assert job["head_sha"] == SOURCE and job["status"] == "completed" and job["conclusion"] == "success"
    log = ROOT / f"ci-{RUN}-{name}.original.log"
    receipt = ROOT / f"ci-{RUN}-{name}.download.json"
    data = log.read_bytes()
    saved = json.loads(receipt.read_text(encoding="utf-8"))
    assert len(data) == saved["bytes"] and hashlib.sha256(data).hexdigest() == saved["sha256"]
    if name in ("frontend", "backend", "backend-windows"):
        assert saved["headSource"] == SOURCE and saved["wholeTreeEquivalentToHead"]
        assert saved["actualTree"] == "6d984c20825172036de79a62e47c3f7bf4f927c3"
    else:
        assert saved["headSource"] == SOURCE and saved["bindingSource"] == "GitHub job API head_sha"
        assert saved["actualCheckout"] is None
    job_rows.append({
        "name": name, "id": jid, "headSha": job["head_sha"], "conclusion": job["conclusion"],
        "logBytes": len(data), "logSha256": hashlib.sha256(data).hexdigest(),
    })

front_raw = (ROOT / f"ci-{RUN}-frontend.original.log").read_text(encoding="utf-8-sig")
front = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", front_raw)
assert re.search(r"Test Files\s+99 passed", front) and re.search(r"Tests\s+538 passed", front)
for name, expected_passed, expected_ignored in [("backend", 923, 6), ("backend-windows", 650, 7)]:
    text = (ROOT / f"ci-{RUN}-{name}.original.log").read_text(encoding="utf-8-sig")
    rows = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(rows) == 35
    assert (sum(int(x[0]) for x in rows), sum(int(x[1]) for x in rows),
            sum(int(x[2]) for x in rows)) == (expected_passed, 0, expected_ignored)

target = ROOT / "ci-admission.json"
assert not target.exists(), "Never rewrite admission"
value = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "branch": "codex/0.1.10-windows-mesh-initial-size", "runId": RUN, "runAttempt": 1,
    "wholeCiSucceeded": True, "originalCheckoutAndJobsVerified": True,
    "runConclusion": run["conclusion"], "frontendPassed": 538, "frontendFiles": 99,
    "macPassed": 923, "macFailed": 0, "windowsPassed": 650, "windowsFailed": 0,
    "jobs": job_rows, "candidatePackagingRequested": False,
    "hostInstallerExecuted": False, "hostProductStarted": False,
    "queryReplayRequested": False, "releaseAccepted": False, "historicalFailuresCleared": False,
    "apiJobHeadShaBindingUsedForLogsWithoutCheckoutMarker": True,
    "formatCheckFailureRetained": True, "ansiRemovedOnlyForParsing": True,
}
target.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
failure_path = ROOT / "initial-admission-parser-failure.json"
assert not failure_path.exists()
failure_path.write_bytes((json.dumps({
    "observedAtLocal": "2026-10-16T19:23:27.8903606+08:00",
    "stage": "offline frontend log count check", "exceptionType": "AssertionError",
    "cause": "ANSI color sequences separate label and count in raw console text",
    "originalLogsChanged": False, "candidateDispatchAttemptCreated": False,
    "subsequentMissingAdmissionGuard": "FileNotFoundError before dispatch request",
    "originalRawRegexMatched": bool(re.search(r"Test Files\s+99 passed", front_raw)),
    "ansiStrippedRegexMatched": bool(re.search(r"Test Files\s+99 passed", front)),
    "workflowReexecuted": False,
}, indent=2) + "\n").encode())
print(json.dumps({
    "source": SOURCE, "runId": RUN, "frontend": 538, "mac": 923, "windows": 650,
    "jobs": 5, "admissionSha256": hashlib.sha256(target.read_bytes()).hexdigest(),
}))
