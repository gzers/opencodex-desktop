"""Dispatch at most one candidate after the exact head's complete CI admission."""
import importlib.util
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"
BRANCH = "feature/0.1.10-windows-query-etw"
ci = json.loads((ROOT.parent / "OCX-0110-20261016-WIN-D94-CI/ci-log-verification.json")
                .read_text(encoding="utf-8"))
assert ci["source"] == SOURCE and ci["actualCheckoutTreesEqual"]
assert ci["wholeCiAdmitted"] and ci["runs"]
assert all(len(r["jobs"]) == 3 and all(j["counts"]["failed"] == 0 for j in r["jobs"])
           for r in ci["runs"])
spec = importlib.util.spec_from_file_location(
    "candidate_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, ref = api.request(f"git/ref/heads/{BRANCH}", token)
assert ref["object"]["sha"] == SOURCE
route = f"actions/workflows/windows-candidate.yml/runs?head_sha={SOURCE}&event=workflow_dispatch&per_page=20"
_, previous = api.request(route, token)
matches = [r for r in previous["workflow_runs"] if r["head_sha"] == SOURCE]
assert len(matches) <= 1, "Ambiguous existing candidate runs; do not dispatch"
if not matches:
    assert not (ROOT / "dispatch-attempt.json").exists(), "Mutation already attempted; inspect the existing run"
    payload = {"ref": BRANCH, "inputs": {"purpose": "candidate"}}
    payload_path = ROOT / "dispatch-request.json"
    payload_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    (ROOT / "dispatch-attempt.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
        "ciRuns": [r["id"] for r in ci["runs"]], "automaticMutationRetry": False,
    }, indent=2) + "\n", encoding="utf-8")
    config = "\n".join([
        "url = " + json.dumps(api.BASE + "actions/workflows/windows-candidate.yml/dispatches"),
        "header = " + json.dumps("Authorization: Bearer " + token),
        'header = "Accept: application/vnd.github+json"',
        'header = "Content-Type: application/json"',
    ]) + "\n"
    response = subprocess.run([
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
        "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
        "--request", "POST", "--data-binary", "@" + str(payload_path),
        "--write-out", "\n%{http_code}",
    ], input=config.encode(), capture_output=True, timeout=35)
    body, _, status = response.stdout.rpartition(b"\n")
    assert response.returncode == 0 and status == b"204", "Dispatch uncertain; inspect remote before any retry"
    (ROOT / "dispatch-result.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "httpStatus": 204,
        "source": SOURCE, "automaticMutationRetryAllowed": False,
    }, indent=2) + "\n", encoding="utf-8")
    for _ in range(6):
        _, discovered = api.request(route, token)
        matches = [r for r in discovered["workflow_runs"] if r["head_sha"] == SOURCE]
        if matches:
            break
        time.sleep(3)
assert len(matches) == 1 and matches[0]["run_attempt"] == 1
run = matches[0]
(ROOT / "candidate-handle.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "runId": run["id"], "runAttempt": 1, "purpose": "candidate",
    "newCandidateRequestsMaximum": 1, "releaseRequested": False,
}, indent=2) + "\n", encoding="utf-8")
(ROOT / "candidate-dispatch-readback.original.json").write_text(
    json.dumps(run, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": SOURCE, "runId": run["id"], "status": run["status"]}))
