"""Read one exact first push CI; no dispatch, rerun, or frozen-file mutation."""
import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "Frozen evidence"
SOURCE = "768e231811a2363b32f5fa67c8e74cbc31d751e1"
reader_path = ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py"
spec = importlib.util.spec_from_file_location("bounded_reader", reader_path)
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
token = reader.credential()
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")


def save(name, value):
    path = ROOT / name
    assert not path.exists()
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


binding = ROOT / "ci-binding.json"
if binding.exists():
    run_id = json.loads(binding.read_text())["runId"]
else:
    _, listing = reader.request(f"actions/runs?head_sha={SOURCE}&per_page=20", token)
    save(f"ci-list-{stamp}.original.json", listing)
    runs = [r for r in listing["workflow_runs"] if r["head_sha"] == SOURCE
            and r["event"] == "push" and r["path"] == ".github/workflows/ci.yml"
            and r["run_attempt"] == 1]
    assert len(runs) == 1, "Require exactly one original push CI"
    run_id = runs[0]["id"]
    save("ci-binding.json", {"source": SOURCE, "runId": run_id, "event": "push", "runAttempt": 1})
_, run = reader.request(f"actions/runs/{run_id}", token)
_, jobs = reader.request(f"actions/runs/{run_id}/jobs?per_page=100", token)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1 and run["event"] == "push"
save(f"ci-run-{stamp}.original.json", run)
save(f"ci-jobs-{stamp}.original.json", jobs)
summary = {
    "source": SOURCE, "runId": run_id, "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{"id": j["id"], "name": j["name"], "status": j["status"], "conclusion": j["conclusion"]}
             for j in jobs["jobs"]],
}
(ROOT / "ci-progress.json").write_bytes((json.dumps(summary, indent=2) + "\n").encode())
if run["status"] == "completed" and not (ROOT / "ci-terminal.json").exists():
    save("ci-terminal.json", summary)
print(json.dumps(summary))
