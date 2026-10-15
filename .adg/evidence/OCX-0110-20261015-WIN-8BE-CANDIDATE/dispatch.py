import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "8be87e2d54a895d4fe02e31300100a1ffed90121"
spec = importlib.util.spec_from_file_location(
    "ghread", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
terminal = json.loads((ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/ci-terminal.json").read_text())
assert terminal["source"] == SOURCE
assert len(terminal["runs"]) == 2
assert all(run["status"] == "completed" and run["conclusion"] == "success" for run in terminal["runs"])
for run_id in (38763397150, 38763409850):
    status, body = api.request(f"actions/runs/{run_id}", token)
    assert status == 200 and body["head_sha"] == SOURCE
    assert body["status"] == "completed" and body["conclusion"] == "success" and body["run_attempt"] == 1
_, branch = api.request("git/ref/heads/feature/0.1.10-windows-query-etw", token)
assert branch["object"]["sha"] == SOURCE
_, existing = api.request(
    "actions/workflows/windows-candidate.yml/runs?head_sha=" + SOURCE + "&per_page=20", token)
(ROOT / "before-dispatch-runs.original.json").write_text(json.dumps(existing, indent=2) + "\n")
assert not existing["workflow_runs"], "Existing candidate found; do not duplicate dispatch"
payload = {"ref": "feature/0.1.10-windows-query-etw",
           "inputs": {"purpose": "candidate", "admission_run": ""}}
with (ROOT / "dispatch-attempt.json").open("x", encoding="utf-8") as f:
    json.dump({"atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
               "candidateRequestsAllowed": 1, "automaticMutationRetryAllowed": False,
               "releaseRequested": False}, f, indent=2)
(ROOT / "dispatch-payload.json").write_text(json.dumps(payload, indent=2) + "\n")
status, _ = api.request("actions/workflows/windows-candidate.yml/dispatches", token, payload, "POST")
assert status == 204
(ROOT / "dispatch-result.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "httpStatus": status,
    "source": SOURCE, "readbackRequired": True, "automaticMutationRetryAllowed": False,
}, indent=2) + "\n")
print(json.dumps({"httpStatus": status, "source": SOURCE, "singleCandidateDispatch": True}))
