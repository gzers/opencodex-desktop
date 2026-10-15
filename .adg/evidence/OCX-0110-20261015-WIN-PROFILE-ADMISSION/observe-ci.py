"""Read only the exact helper head's CI and PR; never dispatch, rerun, or publish."""
import json
import os
import subprocess
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
environment = dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, input="protocol=https\nhost=github.com\n\n",
    capture_output=True, text=True, check=True, timeout=20, env=environment,
)
token = dict(row.split("=", 1) for row in credential.stdout.splitlines() if "=" in row)["password"]


def get(path):
    if "--urllib" in sys.argv:
        request = urllib.request.Request(
            f"https://api.github.com/repos/gzers/opencodex-desktop/{path}",
            headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json",
                     "User-Agent": "OpenCodex-profile-admission-evidence"},
        )
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
              f'header = "Authorization: Bearer {token}"\n'
              'header = "Accept: application/vnd.github+json"\n')
    result = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
         "--fail", "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30"],
        input=config.encode(), capture_output=True, timeout=35,
    )
    if result.returncode:
        error = RuntimeError("GitHub observation unavailable; no terminal state inferred")
        error.curl_exit_code = result.returncode
        error.transport_stderr = result.stderr.decode("utf-8", errors="replace").replace(token, "[REDACTED]")
        raise error
    return json.loads(result.stdout)


stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
try:
    runs = get(f"actions/workflows/ci.yml/runs?head_sha={source}&per_page=100")
    assert runs["total_count"] == len(runs["workflow_runs"]) == 2
    assert {row["event"] for row in runs["workflow_runs"]} == {"push", "pull_request"}
    observations = []
    for row in runs["workflow_runs"]:
        assert row["head_sha"] == source and row["run_attempt"] == 1
        jobs = get(f"actions/runs/{row['id']}/jobs?per_page=100")
        assert jobs["total_count"] == len(jobs["jobs"])
        observations.append({"run": row, "jobs": jobs})
    pr = get("pulls/5")
    assert pr["head"]["sha"] == source and pr["draft"] and not pr["merged"]
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
              "observations": observations, "pr": pr,
              "hostProductStarted": False, "diagnosticRerunRequested": False}
    with (root / f"ci-observation-{stamp}.json").open("x", encoding="utf-8") as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    print(json.dumps({
        "source": source,
        "runs": [{"id": row["run"]["id"], "event": row["run"]["event"],
                  "status": row["run"]["status"], "conclusion": row["run"]["conclusion"],
                  "jobs": [{"name": job["name"], "status": job["status"], "conclusion": job["conclusion"]}
                           for job in row["jobs"]["jobs"]]} for row in observations],
        "prTitle": pr["title"], "prDraft": pr["draft"],
    }), flush=True)
except Exception as error:
    with (root / f"ci-read-error-{stamp}.json").open("x", encoding="utf-8") as stream:
        json.dump({"atUtc": datetime.now(timezone.utc).isoformat(), "errorType": type(error).__name__,
                   "curlExitCode": getattr(error, "curl_exit_code", None),
                   "transportStderr": getattr(error, "transport_stderr", None),
                   "terminalInferred": False, "rerunRequested": False}, stream, indent=2)
        stream.write("\n")
    raise SystemExit("Read failed; retained an error record, no terminal state inferred.")
