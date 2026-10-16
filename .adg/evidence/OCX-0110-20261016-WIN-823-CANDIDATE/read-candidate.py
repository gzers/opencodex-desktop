"""Read the one existing candidate handle; never dispatch or execute a binary."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
handle = json.loads((ROOT / "candidate-handle.json").read_text(encoding="utf-8"))
assert handle["source"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert handle["runId"] == 38841117656 and handle["runAttempt"] == 1
spec = importlib.util.spec_from_file_location(
    "candidate_823_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, run = api.request(f"actions/runs/{handle['runId']}", token)
_, jobs = api.request(f"actions/runs/{handle['runId']}/jobs?per_page=100", token)
_, artifacts = api.request(f"actions/runs/{handle['runId']}/artifacts?per_page=100", token)
assert run["head_sha"] == handle["source"] and run["run_attempt"] == 1
assert run["event"] == "workflow_dispatch" and run["path"] == ".github/workflows/windows-candidate.yml"
assert jobs["total_count"] == len(jobs["jobs"])
expected_artifacts = {"opencodex-desktop-0.1.10-windows-x64-candidate", "windows-candidate-evidence"}
artifact_names = {a["name"] for a in artifacts["artifacts"]}
if run["status"] == "completed" and (
    run["conclusion"] != "success" or expected_artifacts <= artifact_names
):
    terminal = ROOT / "candidate-terminal.json"
    if not terminal.exists():
        terminal.write_text(json.dumps(
            {"run": run, "jobs": jobs, "artifacts": artifacts}, indent=2) + "\n", encoding="utf-8")
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"candidate-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs, "artifacts": artifacts}, indent=2) + "\n", encoding="utf-8")
summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(), "source": handle["source"],
    "runId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{k: j[k] for k in ("id", "name", "status", "conclusion")} for j in jobs["jobs"]],
    "artifacts": [{k: a.get(k) for k in ("id", "name", "size_in_bytes", "digest", "expired")}
                  for a in artifacts["artifacts"]],
    "newDispatchRequested": False, "artifactAdmitted": False, "executableAdmitted": False,
    "productStartedLocally": False, "nativeRecoveryGatePassed": False, "releaseRequested": False,
}
(ROOT / "candidate-progress.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
