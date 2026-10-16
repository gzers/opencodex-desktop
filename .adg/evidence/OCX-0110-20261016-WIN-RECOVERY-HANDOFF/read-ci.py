"""Follow the first CI runs for this exact fix, without reruns or GUI work."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "Frozen checkpoint; use a new directory"
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
spec = importlib.util.spec_from_file_location(
    "recovery_fix_ci_reader",
    ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py",
)
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, listing = api.request(f"actions/workflows/ci.yml/runs?head_sha={SOURCE}&per_page=20", token)
runs = [r for r in listing["workflow_runs"] if r["head_sha"] == SOURCE]
assert runs and len(runs) <= 2
assert len({r["event"] for r in runs}) == len(runs)
observations = []
for run in runs:
    assert run["run_attempt"] == 1 and run["event"] in {"push", "pull_request"}
    _, jobs = api.request(f"actions/runs/{run['id']}/jobs?per_page=100", token)
    observations.append({"run": run, "jobs": jobs})
_, pr = api.request("pulls/5", token)
assert pr["head"]["sha"] == SOURCE and pr["state"] == "open" and pr["draft"]
assert not pr["merged"] and pr["auto_merge"] is None
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"ci-{stamp}.original.json").write_text(
    json.dumps({"runs": observations, "pullRequest": pr}, indent=2) + "\n", encoding="utf-8")
summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "runs": [
        {**{k: row["run"][k] for k in ("id", "event", "status", "conclusion")},
         "jobs": [{k: job[k] for k in ("id", "name", "status", "conclusion")}
                  for job in row["jobs"]["jobs"]]}
        for row in observations
    ],
    "draftPullRequest": 5, "prTitle": pr["title"],
    "rerunRequested": False, "candidateBuildRequested": False,
    "productStarted": False, "nativeRecoveryGatePassed": False,
}
(ROOT / "ci-progress.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary, ensure_ascii=False))
