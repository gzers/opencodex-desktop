"""Package only the admitted source once; do not start any host product."""
import hashlib
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlencode

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
BRANCH = "codex/0.1.10-windows-mesh-initial-size"
RUN = 38891188021
admission = json.loads((ROOT / "ci-admission.json").read_text(encoding="utf-8"))
assert admission["source"] == SOURCE and admission["runId"] == RUN
assert admission["wholeCiSucceeded"] and admission["originalCheckoutAndJobsVerified"]
assert admission["frontendPassed"] == 538 and admission["macPassed"] == 923
assert admission["windowsPassed"] == 650
attempt = ROOT / "candidate-dispatch-attempt.json"
assert not attempt.exists(), "Inspect the previous accepted/uncertain dispatch; never repeat"
spec = importlib.util.spec_from_file_location(
    "candidate_api", Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
                         "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, ref = api.request("git/ref/heads/" + BRANCH, token)
assert ref["object"]["sha"] == SOURCE
query = "actions/workflows/windows-candidate.yml/runs?" + urlencode({
    "head_sha": SOURCE, "event": "workflow_dispatch", "per_page": 30})
_, prior = api.request(query, token)
assert not prior["workflow_runs"], "Inspect an existing candidate instead of dispatching"
payload = {"ref": BRANCH, "inputs": {
    "purpose": "candidate", "admission_run": str(RUN), "diagnostic_path": "isolated-replay"}}
(ROOT / "candidate-dispatch-request.json").write_bytes(
    (json.dumps(payload, indent=2) + "\n").encode())
attempt.write_bytes((json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "admissionRunId": RUN,
    "admissionSha256": hashlib.sha256((ROOT / "ci-admission.json").read_bytes()).hexdigest(),
    "purpose": "candidate", "queryReplayRequested": False, "hostProductLaunchRequested": False,
    "automaticRetry": False, "releaseRequested": False,
}, indent=2) + "\n").encode())
try:
    status, _ = api.request("actions/workflows/windows-candidate.yml/dispatches",
                            token, payload, "POST")
    assert status == 204
    accepted = True
except Exception as error:
    accepted = None
    (ROOT / "candidate-dispatch-transport-failure.json").write_bytes((json.dumps({
        "exceptionType": type(error).__name__, "message": str(error),
        "acceptanceUnknown": True, "automaticRetry": False,
    }, indent=2) + "\n").encode())
_, observed = api.request(query, token)
(ROOT / "candidate-after-dispatch.original.json").write_bytes(
    (json.dumps(observed, indent=2) + "\n").encode())
runs = observed["workflow_runs"]
assert len(runs) <= 1
record = {"source": SOURCE, "httpAccepted": accepted, "runId": runs[0]["id"] if runs else None,
          "status": runs[0]["status"] if runs else "await-readback",
          "automaticRetry": False, "queryReplayRequested": False,
          "hostProductLaunched": False, "releaseRequested": False}
(ROOT / "candidate-dispatch-readback.json").write_bytes(
    (json.dumps(record, indent=2) + "\n").encode())
print(json.dumps(record))
