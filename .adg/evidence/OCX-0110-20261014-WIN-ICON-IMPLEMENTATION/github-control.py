"""Exact-head CI observation, one candidate dispatch, and a reviewable draft PR."""
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a"
branch = "feature/0.1.10-windows-transparent-icons"
base = "feature/0.1.10-release-details"
mode = sys.argv[1] if len(sys.argv) > 1 else "observe"
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    result = subprocess.run(["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
                            text=True, capture_output=True, check=True, timeout=20, cwd=code)
    token = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)["password"]

    def api(path, method="GET", data=None):
        if method == "GET":
            url = "https://api.github.com/repos/gzers/opencodex-desktop/" + path
            config = (f'url = "{url}"\nheader = "Authorization: Bearer {token}"\n'
                      'header = "Accept: application/vnd.github+json"\n'
                      'header = "X-GitHub-Api-Version: 2022-11-28"\n')
            response = subprocess.run(
                [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                 "--connect-timeout", "10", "--max-time", "25", "--retry", "1",
                 "--retry-all-errors", "--retry-delay", "2", "--fail", "--silent", "--show-error"],
                input=config.encode(), capture_output=True, timeout=60)
            if response.returncode:
                raise RuntimeError("Read-only GitHub request failed through system TLS stack")
            return json.loads(response.stdout)
        payload = json.dumps(data, ensure_ascii=False).encode() if data is not None else None
        request = urllib.request.Request("https://api.github.com/repos/gzers/opencodex-desktop/" + path,
            method=method, data=payload, headers={
                "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
                "Content-Type": "application/json", "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "ocx-windows-icon-implementation"})
        for attempt in range(2 if method == "GET" else 1):
            try:
                with urllib.request.urlopen(request, timeout=25) as response:
                    raw = response.read()
                    return json.loads(raw) if raw else {"httpStatus": response.status}
            except urllib.error.URLError:
                if method != "GET" or attempt != 0:
                    raise
                time.sleep(2)

    head = api("git/ref/heads/" + urllib.parse.quote(branch, safe="/"))
    assert head["object"]["sha"] == source
    if mode == "start":
        out = root / "github-start.json"
        assert not out.exists(), "Do not repeat external mutations"
        runs_before = api(f"actions/runs?head_sha={source}&per_page=30")
        candidates = [run for run in runs_before["workflow_runs"] if run["name"] == "Windows candidate installer"]
        assert not candidates, "An exact-source candidate already exists; observe it instead"
        pulls = api("pulls?state=open&head=" + urllib.parse.quote("gzers:" + branch, safe="") + "&per_page=30")
        assert not pulls
        body = (root / "pr-body.md").read_text(encoding="utf-8")
        created = api("pulls", "POST", {
            "title": "fix(windows): 透明应用图标与独立单色托盘",
            "head": branch, "base": base, "draft": True, "body": body,
        })
        assert created["draft"] and created["state"] == "open" and created["head"]["sha"] == source
        partial = {"atUtc": datetime.now(timezone.utc).isoformat(), "head": head, "pullRequest": created,
                   "candidateDispatchIssued": False}
        out.write_text(json.dumps(partial, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        dispatch = api("actions/workflows/windows-candidate.yml/dispatches", "POST", {"ref": branch})
        partial.update(candidateDispatchIssued=True, dispatchResponse=dispatch, candidateRunsBeforeDispatch=candidates,
                       releasePublished=False, pullMerged=False)
        out.write_text(json.dumps(partial, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"draftPr": created["number"], "url": created["html_url"], "candidateDispatched": True}))
    else:
        now = datetime.now(timezone.utc)
        runs = api(f"actions/runs?head_sha={source}&per_page=30")
        rows = []
        details = []
        for run in runs["workflow_runs"]:
            assert run["head_sha"] == source
            jobs = api(f"actions/runs/{run['id']}/jobs?per_page=100")
            artifacts = api(f"actions/runs/{run['id']}/artifacts?per_page=100")
            details.append({"run": run, "jobs": jobs, "artifacts": artifacts})
            rows.append({"id": run["id"], "name": run["name"], "event": run["event"],
                         "status": run["status"], "conclusion": run["conclusion"],
                         "jobs": {job["name"]: {"status": job["status"], "conclusion": job["conclusion"]}
                                  for job in jobs["jobs"]}})
        (root / ("ci-observation-" + now.strftime("%Y%m%dT%H%M%SZ") + ".json")).write_text(
            json.dumps({"atUtc": now.isoformat(), "source": source, "runs": details}, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8")
        print(json.dumps(rows, ensure_ascii=False))
finally:
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
