"""One admitted exact-head candidate request; subsequent operations only observe that run."""
import json
import os
import subprocess
import sys
import time
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
branch = "feature/0.1.10-windows-query-etw"
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, input="protocol=https\nhost=github.com\n\n",
    capture_output=True, text=True, check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


def now():
    return datetime.now(timezone.utc).isoformat()


def stamp():
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def request(path, payload=None):
    method = "GET" if payload is None else "POST"
    for attempt in range(3 if method == "GET" else 1):
        try:
            req = urllib.request.Request(
                "https://api.github.com/repos/gzers/opencodex-desktop/" + path,
                data=None if payload is None else json.dumps(payload).encode("utf-8"), method=method,
                headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json",
                         "Content-Type": "application/json", "User-Agent": "OpenCodex-candidate-evidence"})
            with urllib.request.urlopen(req, timeout=20) as response:
                body = response.read()
                return response.status, json.loads(body) if body else None
        except Exception as error:
            save(f"transport-error-{stamp()}.json", {
                "atUtc": now(), "method": method, "errorType": type(error).__name__,
                "readAttempt": attempt + 1, "dispatchAutomaticallyRepeated": False,
                "terminalInferred": False})
            if method != "GET" or attempt == 2:
                raise


def get(path):
    return request(path)[1]


def list_runs():
    value = get(f"actions/workflows/windows-candidate.yml/runs?head_sha={source}&per_page=100")
    assert value["total_count"] == len(value["workflow_runs"])
    return value


def observe():
    assert (root / "candidate-dispatch-attempt.json").exists()
    visible = list_runs()["workflow_runs"]
    assert len(visible) <= 1, "Multiple candidates require identity review; do not dispatch again."
    if not visible:
        return {"atUtc": now(), "runVisible": False, "terminalInferred": False, "dispatchRepeated": False}
    run = get(f"actions/runs/{visible[0]['id']}")
    assert run["head_sha"] == source and run["run_attempt"] == 1 and run["event"] == "workflow_dispatch"
    jobs = get(f"actions/runs/{run['id']}/jobs?per_page=100")
    artifacts = get(f"actions/runs/{run['id']}/artifacts?per_page=100")
    assert jobs["total_count"] == len(jobs["jobs"])
    assert artifacts["total_count"] == len(artifacts["artifacts"])
    package = [job for job in jobs["jobs"] if job["name"] == "package"]
    assert len(package) <= 1 and all(job["conclusion"] != "skipped" for job in package)
    assert all(job["conclusion"] == "skipped" for job in jobs["jobs"] if job["name"] == "query-etw")
    snapshot = {"atUtc": now(), "source": source, "run": run, "jobs": jobs, "artifacts": artifacts,
                "hostDesktopStarted": False, "hostInstallerExecuted": False}
    save(f"candidate-observation-{stamp()}.json", snapshot)
    if not (root / "candidate-run-binding.json").exists():
        save("candidate-run-binding.json", {"source": source, "runId": run["id"],
             "createdAt": run["created_at"], "priorCandidateRunsAtAdmission": 0})
    else:
        assert json.loads((root / "candidate-run-binding.json").read_text())["runId"] == run["id"]
    if run["status"] == "completed" and not (root / "candidate-terminal.json").exists():
        save("candidate-terminal.json", snapshot)
    return {"runId": run["id"], "source": source, "status": run["status"], "conclusion": run["conclusion"],
            "jobs": [{"name": job["name"], "status": job["status"], "conclusion": job["conclusion"],
                      "activeSteps": [step["name"] for step in job["steps"] if step["status"] == "in_progress"]}
                     for job in jobs["jobs"]]}


mode = sys.argv[1] if len(sys.argv) > 1 else "observe"
if mode == "request":
    assert not (root / "candidate-dispatch-attempt.json").exists()
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=code).decode().strip() == source
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=code)
    admission = get("actions/runs/38670771363")
    assert admission["head_sha"] == source and admission["event"] == "push"
    assert admission["status"] == "completed" and admission["conclusion"] == "success"
    remote = get("git/ref/heads/" + branch)
    assert remote["object"]["sha"] == source
    prior = list_runs()
    assert not prior["workflow_runs"], "Reuse existing candidate; never dispatch a duplicate."
    save("candidate-admission.original.json", {
        "atUtc": now(), "source": source, "ciRun": admission, "remoteRef": remote,
        "priorCandidateRuns": prior, "purpose": "Exact-current-source candidate before next foreground",
        "hostProductStarted": False, "hostInstallerExecuted": False, "releaseRequested": False})
    payload = {"ref": branch, "inputs": {"purpose": "candidate", "admission_run": ""}}
    save("candidate-dispatch-payload.json", payload)
    save("candidate-dispatch-attempt.json", {
        "atUtc": now(), "source": source, "oneCandidateRequest": True, "purpose": "candidate",
        "etwRerunRequested": False, "automaticDispatchRetryAllowed": False,
        "hostDesktopStarted": False, "hostInstallerExecuted": False, "releaseRequested": False})
    try:
        status, _ = request("actions/workflows/windows-candidate.yml/dispatches", payload)
        save("candidate-dispatch-result.json", {"atUtc": now(), "httpStatus": status,
             "remoteReadBackRequired": True, "automaticRetryAllowed": False})
    except Exception:
        raise SystemExit("Dispatch transport uncertain; observe existing runs, never repeat POST.")
    print(json.dumps(observe()), flush=True)
elif mode == "observe":
    print(json.dumps(observe()), flush=True)
elif mode == "watch":
    started = time.monotonic()
    previous = None
    latest = None
    observations = 0
    while time.monotonic() - started < 600:
        try:
            latest = observe()
            observations += 1
            line = json.dumps(latest)
            if line != previous:
                print(line, flush=True)
                previous = line
            if latest.get("status") == "completed":
                break
        except Exception as error:
            save(f"watch-observation-error-{stamp()}.json", {
                "atUtc": now(), "errorType": type(error).__name__, "terminalInferred": False,
                "dispatchRepeated": False})
        time.sleep(30)
    result = {"atUtc": now(), "observations": observations, "elapsedSeconds": time.monotonic() - started,
              "latest": latest, "remoteTerminalObserved": bool(latest and latest.get("status") == "completed"),
              "dispatchRepeated": False, "hostDesktopStarted": False}
    save(f"watch-result-{stamp()}.json", result)
    print(json.dumps(result), flush=True)
else:
    raise SystemExit("Unknown mode")
