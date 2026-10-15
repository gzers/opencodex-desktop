"""Read the two existing exact-head runs; never dispatch, restart, or record ETW."""
import concurrent.futures
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "629b9c6f913b5e04badc9ce747f6bb8ba247fd89"
runs = {38567345341: "push", 38567352891: "pull_request"}
deadline = time.monotonic() + 600
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
observations = root / "ci-observations.jsonl"
assert not observations.exists(), "Inspect the existing observer instead of starting another."


def save(name, value):
    path = root / name
    assert not path.exists()
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, capture_output=True,
                                text=True, input="protocol=https\nhost=github.com\n\n",
                                timeout=20, check=True)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def get(path):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n')
        response = subprocess.run(
            [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
             "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"],
            input=config.encode(), capture_output=True, timeout=30)
        if response.returncode:
            raise RuntimeError("Read-only transport unavailable; no terminal state inferred.")
        return json.loads(response.stdout)

    def inspect(run_id):
        run = get(f"actions/runs/{run_id}")
        assert run["id"] == run_id and run["head_sha"] == source and run["event"] == runs[run_id]
        jobs = get(f"actions/runs/{run_id}/jobs?per_page=100")
        assert all(job["run_id"] == run_id for job in jobs["jobs"])
        return {"run": run, "jobs": jobs}

    with observations.open("x", encoding="utf-8", newline="\n") as stream:
        last = None
        final = {}
        while True:
            observation = {"atUtc": datetime.now(timezone.utc).isoformat(),
                           "source": source, "manuallyDispatched": False, "runs": [], "errors": []}
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                futures = {pool.submit(inspect, run_id): run_id for run_id in runs if run_id not in final}
                for future in concurrent.futures.as_completed(futures):
                    run_id = futures[future]
                    try:
                        row = future.result()
                        if row["run"]["status"] == "completed":
                            save(f"ci-terminal-{run_id}.json", row)
                            final[run_id] = row
                        else:
                            initial = root / f"ci-initial-{run_id}.json"
                            if not initial.exists():
                                save(initial.name, row)
                        observation["runs"].append(row)
                    except Exception as error:
                        observation["errors"].append({"runId": run_id, "errorType": type(error).__name__})
            for run_id, row in final.items():
                if all(item["run"]["id"] != run_id for item in observation["runs"]):
                    observation["runs"].append(row)
            compact = {key: value for key, value in observation.items() if key != "runs"}
            compact["runs"] = [
                {"id": row["run"]["id"], "event": row["run"]["event"],
                 "status": row["run"]["status"], "conclusion": row["run"]["conclusion"],
                 "jobs": {job["name"]: job["conclusion"] if job["status"] == "completed"
                          else job["status"] for job in row["jobs"]["jobs"]}}
                for row in sorted(observation["runs"], key=lambda item: item["run"]["id"])]
            stream.write(json.dumps(compact) + "\n")
            stream.flush()
            state = json.dumps({"runs": compact["runs"], "errors": compact["errors"]}, sort_keys=True)
            if state != last:
                print(json.dumps(compact), flush=True)
                last = state
            if len(final) == len(runs):
                save("observer-result.json", {"status": "all-terminal", "atUtc": compact["atUtc"],
                     "source": source, "runs": compact["runs"], "newTestRuns": 0, "newEtwSessions": 0})
                break
            if time.monotonic() >= deadline:
                save("observer-result.json", {"status": "observation-deadline-not-ci-terminal",
                     "atUtc": compact["atUtc"], "source": source, "runs": compact["runs"],
                     "errors": compact["errors"], "newTestRuns": 0, "newEtwSessions": 0})
                break
            time.sleep(min(15, max(0, deadline - time.monotonic())))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
