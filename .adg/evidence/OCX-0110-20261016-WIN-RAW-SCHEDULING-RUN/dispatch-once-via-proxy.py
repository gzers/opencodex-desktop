"""One revised, authorized original-bd2 diagnostic on an isolated Windows runner."""
import hashlib
import importlib.util
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "Frozen evidence"
SOURCE = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
BRANCH = "feature/0.1.10-windows-etw-query-threads"
admission = json.loads((ROOT / "ci-admission.json").read_text(encoding="utf-8"))
assert admission["source"] == SOURCE and admission["runId"] == 38858859494
assert admission["wholeCiSucceeded"] and admission["originalCheckoutAndJobsVerified"]
assert not (ROOT / "recording-attempt.json").exists(), "Local recording already attempted"
spec = importlib.util.spec_from_file_location(
    "query_remote_reader", ROOT / "github-reader-via-proxy.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, ref = api.request(f"git/ref/heads/{BRANCH}", token)
assert ref["object"]["sha"] == SOURCE
route = f"actions/workflows/windows-candidate.yml/runs?head_sha={SOURCE}&event=workflow_dispatch&per_page=20"
_, observed = api.request(route, token)
runs = [r for r in observed["workflow_runs"] if r["head_sha"] == SOURCE]
assert len(runs) <= 1, "Ambiguous existing diagnostic; do not dispatch"
if runs and not (ROOT / "remote-dispatch-attempt.json").exists():
    raise RuntimeError("An existing remote dispatch is not bound to this request; inspect it instead of adopting or repeating it")
if not runs:
    assert not (ROOT / "remote-dispatch-attempt.json").exists(), "Inspect the previous dispatch instead"
    payload = {"ref": BRANCH, "inputs": {"purpose": "query-etw",
        "admission_run": "38858859494", "diagnostic_path": "isolated-replay"}}
    path = ROOT / "remote-dispatch-request.json"
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    plan = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "collectorSource": SOURCE,
        "originalFailedRun": 38537166560, "originalFailedSource": "bd2f6eb10d58115235e35d575507bb69f44666da",
        "testSha256": "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3",
        "officialNodeSha256": "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32",
        "authorization": "批准修正版后续都可以管理员后台 ETW 录制",
        "reason": "revised recorder retains the missing scheduling byte window after the v5 audit; semantics remain unverified and no failure is preclaimed",
        "maximumOriginalLibraryRuns": 1, "automaticRetryAllowed": False,
        "noPrewarm": True, "defaultConcurrency": True, "deadlinesSeconds": [2, 4, 5, 8],
        "outerSeconds": 120, "serializedMiB": 64, "queueRecords": 32768, "trackedIrps": 32768,
        "hostDesktopOrElevationPromptUsed": False, "localRecordEntryExecuted": False,
        "historicalEnvironmentClaimedIdentical": False, "failurePreclaimed": False,
        "admissionSha256": hashlib.sha256((ROOT / "ci-admission.json").read_bytes()).hexdigest(),
    }
    (ROOT / "remote-dispatch-attempt.json").write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")
    config = "\n".join([
        "url = " + json.dumps(api.BASE + "actions/workflows/windows-candidate.yml/dispatches"),
        "header = " + json.dumps("Authorization: Bearer " + token),
        'header = "Accept: application/vnd.github+json"',
        'header = "Content-Type: application/json"',
    ]) + "\n"
    result = subprocess.run([
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "--proxy", "socks5h://[::1]:3060", "--noproxy", "", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
        "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
        "--request", "POST", "--data-binary", "@" + str(path),
        "--write-out", "\n%{http_code}",
    ], input=config.encode(), capture_output=True, timeout=35)
    body, _, status = result.stdout.rpartition(b"\n")
    assert result.returncode == 0 and status == b"204", "Dispatch uncertain; inspect remote, do not retry"
    (ROOT / "remote-dispatch-result.json").write_text(json.dumps({
        "httpStatus": 204, "collectorSource": SOURCE, "automaticRetryAllowed": False,
    }, indent=2) + "\n", encoding="utf-8")
    for _ in range(6):
        _, observed = api.request(route, token)
        runs = [r for r in observed["workflow_runs"] if r["head_sha"] == SOURCE]
        if runs:
            break
        time.sleep(3)
assert len(runs) == 1 and runs[0]["run_attempt"] == 1
run = runs[0]
(ROOT / "remote-handle.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": run["id"],
    "runAttempt": 1, "purpose": "query-etw", "maximumOriginalRuns": 1,
    "noHostDesktopOrUac": True, "releaseRequested": False,
}, indent=2) + "\n", encoding="utf-8")
(ROOT / "remote-dispatch-readback.original.json").write_text(
    json.dumps(run, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": run["id"], "source": SOURCE, "status": run["status"],
                  "purpose": "query-etw", "maximumOriginalLibraryRuns": 1}))
