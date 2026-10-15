"""Observe one requested exact-head packaging run; bounded local observer only."""
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
preflight = json.loads((root / "candidate-preflight.json").read_text(encoding="utf-8-sig"))
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
deadline = time.monotonic() + 1500

try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, text=True,
                                input="protocol=https\nhost=github.com\n\n", capture_output=True,
                                check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def get(path):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
        result = subprocess.run([r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                                 "--connect-timeout", "10", "--max-time", "25", "--fail",
                                 "--silent", "--show-error"], input=config.encode(),
                                capture_output=True, timeout=30)
        if result.returncode:
            raise RuntimeError("Read-only transport unavailable; no restart or dispatch.")
        return json.loads(result.stdout)

    with (root / "candidate-observations.jsonl").open("x", encoding="utf-8", newline="\n") as stream:
        run_id = None
        last_state = None
        while time.monotonic() < deadline:
            now = datetime.now(timezone.utc).isoformat()
            try:
                if run_id is None:
                    runs = get(f"actions/workflows/{preflight['workflowId']}/runs?head_sha={source}&per_page=30")
                    exact = [row for row in runs["workflow_runs"] if row["head_sha"] == source
                             and row["event"] == "workflow_dispatch"]
                    if not exact:
                        raise RuntimeError("Requested run not yet visible.")
                    assert len(exact) == 1, "Do not silently select between duplicate dispatched runs."
                    run_id = exact[0]["id"]
                    (root / "candidate-run-binding.json").write_text(json.dumps(
                        {"source": source, "runId": run_id, "createdAt": exact[0]["created_at"],
                         "workflowId": preflight["workflowId"]}, indent=2) + "\n", encoding="utf-8")
                run = get(f"actions/runs/{run_id}")
                assert run["head_sha"] == source and run["event"] == "workflow_dispatch"
                jobs = get(f"actions/runs/{run_id}/jobs?per_page=100")
                snapshot = {"atUtc": now, "runId": run_id, "source": source,
                            "status": run["status"], "conclusion": run["conclusion"],
                            "jobs": [{"id": row["id"], "name": row["name"], "status": row["status"],
                                      "conclusion": row["conclusion"],
                                      "activeSteps": [step["name"] for step in row.get("steps", [])
                                                      if step["status"] == "in_progress"]}
                                     for row in jobs["jobs"]]}
                stream.write(json.dumps(snapshot) + "\n"); stream.flush()
                state = json.dumps({key: value for key, value in snapshot.items() if key != "atUtc"})
                if state != last_state:
                    print(json.dumps(snapshot), flush=True); last_state = state
                if run["status"] == "completed":
                    artifacts = get(f"actions/runs/{run_id}/artifacts?per_page=100")
                    (root / "candidate-terminal.json").write_text(json.dumps(
                        {"atUtc": now, "run": run, "jobs": jobs, "artifacts": artifacts},
                        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
                    break
            except Exception as error:
                snapshot = {"atUtc": now, "runId": run_id, "readError": type(error).__name__,
                            "terminalInferred": False, "restartOrDispatchRequested": False}
                stream.write(json.dumps(snapshot) + "\n"); stream.flush()
            time.sleep(min(25, max(0, deadline - time.monotonic())))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
