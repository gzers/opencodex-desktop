"""Dispatch one candidate only after frozen, verified complete same-source CI."""
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
assert not (ROOT / "manifest.json").exists()
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
BRANCH = "feature/0.1.10-windows-query-etw"
ci_root = ROOT.parent / "OCX-0110-20261016-WIN-RECOVERY-HANDOFF"
verification = json.loads((ci_root / "verification.json").read_text(encoding="utf-8"))
ci = json.loads((ci_root / "pr-ci-collection.json").read_text(encoding="utf-8"))
assert verification["source"] == ci["source"] == SOURCE
assert verification["wholeCiSucceeded"] and ci["wholeCiSucceeded"]
assert ci["status"] == "completed" and ci["conclusion"] == "success"
assert ci["runId"] == 38838969102 and len(ci["archivedJobs"]) == 3
assert all(j["logArchived"] and j["conclusion"] == "success"
           and j["fullTree"] == ci["headFullTree"] for j in ci["archivedJobs"])
spec = importlib.util.spec_from_file_location(
    "candidate_823_api", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, ref = api.request(f"git/ref/heads/{BRANCH}", token)
assert ref["object"]["sha"] == SOURCE
route = f"actions/workflows/windows-candidate.yml/runs?head_sha={SOURCE}&event=workflow_dispatch&per_page=20"
_, listing = api.request(route, token)
matches = [r for r in listing["workflow_runs"] if r["head_sha"] == SOURCE]
assert len(matches) <= 1
if not matches:
    assert not (ROOT / "dispatch-attempt.json").exists(), "Inspect the remote before any mutation retry"
    payload = {"ref": BRANCH, "inputs": {"purpose": "candidate"}}
    payload_path = ROOT / "dispatch-request.json"
    payload_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    (ROOT / "dispatch-attempt.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
        "ciRunId": ci["runId"], "ciFullTree": ci["headFullTree"],
        "ciVerificationSha256": hashlib.sha256((ci_root / "verification.json").read_bytes()).hexdigest(),
        "ciManifestSha256": hashlib.sha256((ci_root / "manifest.json").read_bytes()).hexdigest(),
        "automaticMutationRetry": False, "releaseRequested": False,
    }, indent=2) + "\n", encoding="utf-8")
    config = "\n".join([
        "url = " + json.dumps(api.BASE + "actions/workflows/windows-candidate.yml/dispatches"),
        "header = " + json.dumps("Authorization: Bearer " + token),
        'header = "Accept: application/vnd.github+json"',
        'header = "Content-Type: application/json"',
    ]) + "\n"
    result = subprocess.run([
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
        "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
        "--request", "POST", "--data-binary", "@" + str(payload_path),
        "--write-out", "\n%{http_code}",
    ], input=config.encode(), capture_output=True, timeout=35)
    body, _, status = result.stdout.rpartition(b"\n")
    assert result.returncode == 0 and status == b"204", "Uncertain dispatch; do not repeat the POST"
    (ROOT / "dispatch-result.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
        "httpStatus": 204, "automaticMutationRetryAllowed": False,
    }, indent=2) + "\n", encoding="utf-8")
    for _ in range(6):
        _, listing = api.request(route, token)
        matches = [r for r in listing["workflow_runs"] if r["head_sha"] == SOURCE]
        if matches:
            break
        time.sleep(3)
assert len(matches) == 1 and matches[0]["run_attempt"] == 1
run = matches[0]
(ROOT / "candidate-dispatch-readback.original.json").write_text(
    json.dumps(run, indent=2) + "\n", encoding="utf-8")
(ROOT / "candidate-handle.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "runId": run["id"], "runAttempt": 1, "purpose": "candidate",
    "maximumDispatches": 1, "releaseRequested": False, "nativeRecoveryGatePassed": False,
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": SOURCE, "runId": run["id"], "status": run["status"]}))
