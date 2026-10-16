"""Read the original baseline release build; no downloads or executions of EXEs."""
import importlib.util
import json
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
SOURCE = "de6862d40389d527527b9db8ef52e186a58b625f"
spec = importlib.util.spec_from_file_location(
    "baseline_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
queries = {
    "release-asset.original.json": "releases/assets/623593980",
    "source-runs.original.json": f"actions/runs?head_sha={SOURCE}&per_page=100",
}
with ThreadPoolExecutor(max_workers=2) as pool:
    pending = {name: pool.submit(api.request, path, token) for name, path in queries.items()}
    values = {}
    for name, handle in pending.items():
        status, body = handle.result()
        assert status == 200
        values[name] = body
        with (ROOT / name).open("xb") as out:
            out.write((json.dumps(body, ensure_ascii=False, indent=2) + "\n").encode())
asset = values["release-asset.original.json"]
assert asset["id"] == 623593980 and asset["state"] == "uploaded"
assert asset["digest"] == "sha256:b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"
assert "/v0.1.9/" in asset["browser_download_url"]
runs = values["source-runs.original.json"]
assert runs["total_count"] == len(runs["workflow_runs"]) <= 100
assert all(row["head_sha"] == SOURCE for row in runs["workflow_runs"])
print(json.dumps({
    "baselineSource": SOURCE, "assetId": asset["id"], "assetCreatedAt": asset["created_at"],
    "runs": [{k: row[k] for k in ["id", "name", "path", "event", "head_branch",
                                 "head_sha", "run_attempt", "status", "conclusion", "created_at"]}
             for row in runs["workflow_runs"]],
    "productStarted": False, "ciDispatched": False,
}))
