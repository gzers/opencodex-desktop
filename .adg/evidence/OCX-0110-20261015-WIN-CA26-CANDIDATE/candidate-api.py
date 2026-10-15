"""One candidate request or read-only observation, distinguishing the earlier ETW mode."""
import concurrent.futures
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
branch = "feature/0.1.10-windows-query-etw"
env = dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20, env=env)
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def request(path, payload=None):
    config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
              f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
    if payload:
        config += 'request = "POST"\nheader = "Content-Type: application/json"\n'
        config += 'data-binary = "@' + payload.as_posix() + '"\n'
    result = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
         "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
         "--write-out", "\n%{http_code}"],
        input=config.encode(), capture_output=True, timeout=35)
    body, _, status = result.stdout.rpartition(b"\n")
    if result.returncode:
        raise RuntimeError("GitHub read/transport unavailable; no automatic dispatch retry.")
    return (json.loads(body) if body.strip() else None), int(status)


def get(path):
    return request(path)[0]


def observe():
    prior = json.loads((root / "candidate-preflight.json").read_text(encoding="utf-8"))
    candidates = get(f"actions/workflows/windows-candidate.yml/runs?head_sha={source}&per_page=30")
    assert candidates["total_count"] == len(candidates["workflow_runs"])
    new = [row for row in candidates["workflow_runs"] if row["id"] not in prior["priorRunIds"]]
    assert len(new) <= 1, "Multiple new runs require manual identity review."
    if not new:
        return {"atUtc": datetime.now(timezone.utc).isoformat(), "runNotVisibleYet": True,
                "terminalInferred": False, "dispatchRepeated": False}
    run = get("actions/runs/" + str(new[0]["id"]))
    assert run["head_sha"] == source and run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        jobs_future = pool.submit(get, f"actions/runs/{run['id']}/jobs?per_page=100")
        artifacts_future = pool.submit(get, f"actions/runs/{run['id']}/artifacts?per_page=100")
        jobs, artifacts = jobs_future.result(), artifacts_future.result()
    assert jobs["total_count"] == len(jobs["jobs"])
    packages = [row for row in jobs["jobs"] if row["name"] == "package"]
    if packages:
        assert len(packages) == 1 and packages[0]["conclusion"] != "skipped"
    diagnostics = [row for row in jobs["jobs"] if row["name"] == "query-etw"]
    assert all(row["conclusion"] == "skipped" for row in diagnostics)
    snapshot = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "run": run,
                "jobs": jobs, "artifacts": artifacts, "hostDesktopStarted": False, "hostInstallerExecuted": False}
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    save(f"candidate-observation-{stamp}.json", snapshot)
    if not (root / "candidate-run-binding.json").exists():
        save("candidate-run-binding.json", {"source": source, "runId": run["id"],
             "createdAt": run["created_at"], "priorDifferentModeRunId": 38662786466})
    else:
        assert json.loads((root / "candidate-run-binding.json").read_text())["runId"] == run["id"]
    if run["status"] == "completed" and not (root / "candidate-terminal.json").exists():
        save("candidate-terminal.json", snapshot)
    return {"runId": run["id"], "source": source, "status": run["status"], "conclusion": run["conclusion"],
            "jobs": [{"name": row["name"], "status": row["status"], "conclusion": row["conclusion"],
                      "activeSteps": [step["name"] for step in row["steps"] if step["status"] == "in_progress"]}
                     for row in jobs["jobs"]]}


mode = sys.argv[1] if len(sys.argv) > 1 else "observe"
if mode == "request":
    assert not (root / "candidate-dispatch-attempt.json").exists()
    ci = json.loads((root.parent / "OCX-0110-20261015-WIN-CI-ETW/ci-terminal-38660621496.json").read_text())
    assert ci["run"]["head_sha"] == source and ci["run"]["conclusion"] == "success"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=code).decode().strip() == source
    subprocess.run(["git", "diff", "--quiet", "HEAD", "--"], cwd=code, check=True)
    remote = get("git/ref/heads/" + branch)
    assert remote["object"]["sha"] == source
    prior = get(f"actions/workflows/windows-candidate.yml/runs?head_sha={source}&per_page=30")
    assert prior["total_count"] == len(prior["workflow_runs"])
    assert len(prior["workflow_runs"]) == 1 and prior["workflow_runs"][0]["id"] == 38662786466
    existing_jobs = get("actions/runs/38662786466/jobs?per_page=100")
    assert next(row for row in existing_jobs["jobs"] if row["name"] == "package")["conclusion"] == "skipped"
    save("candidate-preflight.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "branch": branch,
        "admissionRun": 38660621496, "priorRunIds": [38662786466], "priorRunMode": "query-etw/package-skipped",
        "hostNativeInstancesStarted": 0, "hostInstallerExecuted": False, "releaseRequested": False})
    save("candidate-dispatch-payload.json", {"ref": branch, "inputs": {"purpose": "candidate", "admission_run": ""}})
    save("candidate-dispatch-attempt.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
        "purpose": "prepare exact-current-source candidate before next foreground signal",
        "singleCandidateRequest": True, "etwRerunRequested": False,
        "hostDesktopStarted": False, "hostInstallerExecuted": False,
        "releaseRequested": False, "automaticRequestRetryAllowed": False})
    try:
        _, status = request("actions/workflows/windows-candidate.yml/dispatches",
                            root / "candidate-dispatch-payload.json")
        result = {"atUtc": datetime.now(timezone.utc).isoformat(), "httpStatus": status,
                  "remoteRunMustBeReadBack": True, "automaticRetryAllowed": False}
    except Exception as error:
        result = {"atUtc": datetime.now(timezone.utc).isoformat(), "errorType": type(error).__name__,
                  "remoteRunMustBeReadBack": True, "automaticRetryAllowed": False}
    save("candidate-dispatch-result.json", result)
    print(json.dumps(result), flush=True)
elif mode == "observe":
    print(json.dumps(observe()), flush=True)
elif mode == "watch":
    result_path = root / "candidate-watch-result.json"
    assert not result_path.exists()
    started, count, latest, prior_print = time.monotonic(), 0, None, None
    while time.monotonic() - started < 600:
        try:
            latest = observe()
            count += 1
            text = json.dumps(latest)
            if text != prior_print:
                print(text, flush=True)
                prior_print = text
            if latest.get("status") == "completed":
                break
        except Exception as error:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
            save(f"candidate-read-error-{stamp}.json", {"atUtc": datetime.now(timezone.utc).isoformat(),
                 "errorType": type(error).__name__, "terminalInferred": False, "dispatchRepeated": False})
        time.sleep(30)
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "observations": count,
              "elapsedSeconds": time.monotonic() - started, "lastObservation": latest,
              "remoteTerminalObserved": bool(latest and latest.get("status") == "completed"),
              "dispatchRepeated": False, "hostDesktopStarted": False}
    save(result_path.name, result)
    print(json.dumps(result), flush=True)
else:
    raise SystemExit("Unknown read/request mode.")
