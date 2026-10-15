"""One exact-source isolated candidate build. No release, installation, or UI work."""
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
branch = "feature/0.1.10-windows-query-etw"
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}


def save(name, value):
    path = root / name
    if path.exists():
        raise RuntimeError("Existing request/evidence must be inspected instead of repeated.")
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, capture_output=True,
                                text=True, input="protocol=https\nhost=github.com\n\n",
                                timeout=20, check=True)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def api(path, post=None):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n'
                  'header = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "25", "--silent", "--show-error",
                "--fail", "--write-out", "\n%{http_code}"]
        if post is None:
            args += ["--retry", "1", "--retry-all-errors", "--retry-delay", "1"]
        else:
            payload = root / "candidate-dispatch-payload.json"
            save(payload.name, post)
            args += ["--request", "POST", "--data-binary", "@" + str(payload)]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=60)
        body, _, status = response.stdout.rpartition(b"\n")
        if response.returncode:
            raise RuntimeError("GitHub transport unavailable. Read existing runs before repeating any request.")
        return (json.loads(body) if body.strip() else None), int(status)

    remote, _ = api("git/ref/heads/" + branch)
    assert remote["object"]["sha"] == source
    workflow, _ = api("actions/workflows/windows-candidate.yml")
    assert workflow["state"] == "active" and workflow["path"] == ".github/workflows/windows-candidate.yml"
    prior_runs, _ = api(f"actions/workflows/{workflow['id']}/runs?head_sha={source}&per_page=30")
    candidates = [run for run in prior_runs["workflow_runs"] if run["head_sha"] == source]
    save("candidate-preflight.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "branch": branch,
        "workflowId": workflow["id"], "workflow": workflow["path"],
        "existingRuns": [{key: row[key] for key in ("id", "head_sha", "status", "conclusion", "event")}
                         for row in candidates],
        "foregroundAuthorization": "可以开始 Windows 前台验证，我现在有 30 分钟。",
        "releaseOrPromotionRequested": False, "hostInstallerExecuted": False,
    })
    if candidates:
        save("candidate-request-result.json", {"status": "existing-run-reused",
             "source": source, "runs": candidates, "manualDispatchCount": 0})
    else:
        save("candidate-mutation-prepared.json", {
            "source": source, "branch": branch, "workflowId": workflow["id"],
            "action": "one isolated Windows candidate workflow dispatch; no release",
            "atUtc": datetime.now(timezone.utc).isoformat(),
        })
        _, status = api(f"actions/workflows/{workflow['id']}/dispatches", {"ref": branch})
        assert status == 204
        save("candidate-request-result.json", {"status": "dispatch-accepted",
             "source": source, "branch": branch, "httpStatus": status,
             "manualDispatchCount": 1, "runIdNotYetClaimed": True})
    print((root / "candidate-request-result.json").read_text(encoding="utf-8"))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
