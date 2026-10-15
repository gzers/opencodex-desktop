import importlib.util
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "8be87e2d54a895d4fe02e31300100a1ffed90121"
terminal = ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/ci-terminal.json"
deadline = time.monotonic() + 1800
while not terminal.exists() and time.monotonic() < deadline:
    time.sleep(5)
if not terminal.exists():
    raise RuntimeError("Matching CI still pending; no candidate dispatch performed")
state = json.loads(terminal.read_text())
if not all(run["conclusion"] == "success" for run in state["runs"]):
    (ROOT / "not-dispatched-ci-failed.json").write_text(json.dumps(state, indent=2) + "\n")
    raise RuntimeError("Exact-head CI failed; candidate dispatch refused")
if not (ROOT / "dispatch-attempt.json").exists():
    subprocess.run([sys.executable, str(ROOT / "dispatch.py")], check=True)
if not (ROOT / "dispatch-result.json").exists():
    raise RuntimeError("Prior dispatch result uncertain; read remote, never retry mutation")
spec = importlib.util.spec_from_file_location(
    "ghread", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
run = None
for _ in range(12):
    _, rows = api.request("actions/workflows/windows-candidate.yml/runs?head_sha=" + SOURCE + "&per_page=20", token)
    matches = rows["workflow_runs"]
    assert len(matches) <= 1, "Multiple exact-source candidate runs; stop and investigate"
    if matches:
        run = matches[0]
        break
    time.sleep(5)
assert run and run["head_sha"] == SOURCE and run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
(ROOT / "candidate-first-read.original.json").write_text(json.dumps(run, indent=2) + "\n")
deadline = time.monotonic() + 2700
while time.monotonic() < deadline:
    _, run = api.request(f"actions/runs/{run['id']}", token)
    assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
    progress = {"atUtc": datetime.now(timezone.utc).isoformat(), "run": run["id"],
                "source": SOURCE, "status": run["status"], "conclusion": run["conclusion"],
                "singleDispatchOnly": True}
    (ROOT / "candidate-progress.json").write_text(json.dumps(progress, indent=2) + "\n")
    if run["status"] == "completed":
        _, jobs = api.request(f"actions/runs/{run['id']}/jobs?per_page=100", token)
        _, artifacts = api.request(f"actions/runs/{run['id']}/artifacts?per_page=100", token)
        (ROOT / "candidate-terminal.json").write_text(json.dumps({
            "atUtc": datetime.now(timezone.utc).isoformat(), "run": run,
            "jobs": jobs, "artifacts": artifacts,
        }, indent=2) + "\n")
        assert run["conclusion"] == "success", "Candidate did not succeed; retain failure"
        print(json.dumps(progress))
        break
    time.sleep(25)
else:
    raise RuntimeError("Candidate still pending; retain same run id and do not redispatch")
