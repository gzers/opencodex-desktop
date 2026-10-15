import importlib.util
import json
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ghread", ROOT / "github-reader.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
token = module.credential()
source = "8be87e2d54a895d4fe02e31300100a1ffed90121"
run_ids = (38763397150, 38763409850)
deadline = time.monotonic() + 900
try:
    while time.monotonic() < deadline:
        summary = []
        for run_id in run_ids:
            status, body = module.request(f"actions/runs/{run_id}", token)
            assert status == 200 and body["head_sha"] == source and body["run_attempt"] == 1
            item = {"run": run_id, "status": body["status"], "conclusion": body["conclusion"]}
            summary.append(item)
            if body["status"] == "completed":
                path = ROOT / f"run-{run_id}-terminal.original.json"
                if not path.exists():
                    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
        snapshot = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runs": summary}
        (ROOT / "ci-progress.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
        if all(item["status"] == "completed" for item in summary):
            (ROOT / "ci-terminal.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
            break
        time.sleep(25)
    else:
        (ROOT / "ci-watch-bound.json").write_text(json.dumps({
            "atUtc": datetime.now(timezone.utc).isoformat(), "stillPending": True,
            "sameRunIdsRetained": list(run_ids), "noRerunTriggered": True,
        }, indent=2) + "\n", encoding="utf-8")
except Exception as error:
    error_name = "ci-watch-error-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + ".json"
    (ROOT / error_name).write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "errorType": type(error).__name__,
        "sameRunIdsRetained": list(run_ids), "noRerunTriggered": True,
    }, indent=2) + "\n", encoding="utf-8")
    raise
