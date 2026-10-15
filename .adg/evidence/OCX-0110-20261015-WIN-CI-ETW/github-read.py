"""Read-only GitHub observation. Credentials stay in process memory and curl stdin."""
import concurrent.futures
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
sha = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
env = dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20, env=env)
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


def get(path):
    config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
              f'header = "Authorization: Bearer {token}"\n'
              'header = "Accept: application/vnd.github+json"\n')
    response = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
         "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30"],
        input=config.encode(), capture_output=True, timeout=35,
    )
    if response.returncode:
        return {"readError": True, "curlExitCode": response.returncode, "remoteTerminalClaimed": False}
    return json.loads(response.stdout)


paths = {
    "runs": f"actions/runs?head_sha={sha}&per_page=30",
    "pr5": "pulls/5",
}
if len(sys.argv) > 1:
    paths.update({f"run-{number}": f"actions/runs/{number}" for number in sys.argv[1:] if number.isdigit()})
    paths.update({f"jobs-{number}": f"actions/runs/{number}/jobs?per_page=100"
                  for number in sys.argv[1:] if number.isdigit()})
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
observations = {}
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    futures = {name: pool.submit(get, path) for name, path in paths.items()}
    for name, future in futures.items():
        try:
            observations[name] = future.result()
        except Exception as error:
            observations[name] = {"readError": True, "type": type(error).__name__, "remoteTerminalClaimed": False}
snapshot = {"atUtc": datetime.now(timezone.utc).isoformat(), "collectorSource": sha, "reads": observations,
            "mutations": 0, "desktopStarted": False}
(root / f"observation-{stamp}.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
for row in observations.get("runs", {}).get("workflow_runs", []):
    print(json.dumps({key: row.get(key) for key in ("id", "name", "event", "head_sha", "status", "conclusion")}))
for name, value in observations.items():
    if name.startswith("jobs-"):
        print(json.dumps({"run": name.removeprefix("jobs-"), "jobs": [
            {key: row.get(key) for key in ("name", "status", "conclusion")}
            for row in value.get("jobs", [])]}))
    if value.get("readError"):
        print(json.dumps({"read": name, **value}))
pr = observations.get("pr5", {})
print(json.dumps({"pr": 5, "state": pr.get("state"), "draft": pr.get("draft"),
                  "head": pr.get("head", {}).get("sha"), "merged": pr.get("merged")}))
