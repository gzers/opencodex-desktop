import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
reader = ROOT.parent / "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"
spec = importlib.util.spec_from_file_location("original_reader", reader)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
token = module.credential()
summary = []
for run in (38734133095, 38734138996, 38737628201):
    status, body = module.request(f"actions/runs/{run}", token)
    assert status == 200
    with (ROOT / f"run-{run}.original.json").open("x", encoding="utf-8") as f:
        json.dump(body, f, ensure_ascii=False, indent=2)
    assert body["head_sha"] == module.SOURCE
    assert body["status"] == "completed" and body["conclusion"] == "success"
    summary.append({"run": run, "head": body["head_sha"], "status": body["status"],
                    "conclusion": body["conclusion"], "attempt": body["run_attempt"]})
with (ROOT / "ci-readback.json").open("x", encoding="utf-8") as f:
    json.dump({"atUtc": datetime.now(timezone.utc).isoformat(), "runs": summary},
              f, indent=2)
print(json.dumps(summary))
