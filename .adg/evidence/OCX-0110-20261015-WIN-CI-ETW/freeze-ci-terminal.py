"""Freeze existing run handles only after the API reports their real terminal state."""
import json
from pathlib import Path

root = Path(__file__).resolve().parent
snapshot = json.loads(max(root.glob("observation-*.json")).read_text(encoding="utf-8"))
for number in (38660621496, 38660625647):
    run = snapshot["reads"]["run-" + str(number)]
    jobs = snapshot["reads"]["jobs-" + str(number)]
    assert run["id"] == number and run["head_sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
    assert run["status"] == "completed", "Remote run remains live; do not freeze or restart it."
    assert jobs["total_count"] == len(jobs["jobs"])
    assert all(row["status"] == "completed" for row in jobs["jobs"])
    path = root / f"ci-terminal-{number}.json"
    assert not path.exists()
    path.write_text(json.dumps({"atUtc": snapshot["atUtc"], "run": run, "jobs": jobs}, indent=2) + "\n",
                    encoding="utf-8")
print(json.dumps({"frozenRunIds": [38660621496, 38660625647], "restarts": 0, "dispatches": 0}))
