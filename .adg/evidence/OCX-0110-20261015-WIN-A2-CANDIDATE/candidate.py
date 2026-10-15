"""One exact-source candidate, followed only by reads of its retained run."""
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
source, branch = api.SOURCE, "feature/0.1.10-windows-query-etw"
assert source == "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
token = api.credential()
now = lambda: datetime.now(timezone.utc).isoformat()
stamp = lambda: datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")


def save(name, value):
    with (root / name).open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def get(path):
    return api.request(path, token)[1]


def observe():
    binding = root / "candidate-run-binding.json"
    if not binding.exists():
        listing = get("actions/workflows/windows-candidate.yml/runs?head_sha=" + source + "&per_page=100")
        assert listing["total_count"] == len(listing["workflow_runs"]) <= 1
        if not listing["workflow_runs"]:
            return {"atUtc": now(), "visible": False, "terminal": False}
        run = listing["workflow_runs"][0]
        save(binding.name, {"runId": run["id"], "source": source})
    run_id = json.loads(binding.read_text())["runId"]
    run = get(f"actions/runs/{run_id}")
    jobs = get(f"actions/runs/{run_id}/jobs?per_page=100")
    artifacts = get(f"actions/runs/{run_id}/artifacts?per_page=100")
    assert run["head_sha"] == source and run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
    assert jobs["total_count"] == len(jobs["jobs"])
    assert artifacts["total_count"] == len(artifacts["artifacts"])
    assert all(job["conclusion"] == "skipped" for job in jobs["jobs"] if job["name"] == "query-etw")
    snapshot = {"atUtc": now(), "source": source, "run": run, "jobs": jobs, "artifacts": artifacts,
                "hostProductStarted": False, "hostInstallerExecuted": False}
    terminal = run["status"] == "completed"
    if terminal and not (root / "candidate-terminal.json").exists():
        save("candidate-terminal.json", snapshot)
    return {"snapshot": snapshot, "atUtc": snapshot["atUtc"], "runId": run_id,
            "status": run["status"], "conclusion": run["conclusion"], "terminal": terminal,
            "jobs": [{"name": job["name"], "status": job["status"], "conclusion": job["conclusion"],
                      "activeSteps": [step["name"] for step in job["steps"] if step["status"] == "in_progress"]}
                     for job in jobs["jobs"]]}


mode = sys.argv[1] if len(sys.argv) > 1 else "observe"
if mode == "request":
    assert not (root / "candidate-dispatch-attempt.json").exists()
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=api.CODE).decode().strip() == source
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=api.CODE).strip()
    ci = get("actions/runs/38726727330")
    assert ci["head_sha"] == source and ci["event"] == "push"
    assert ci["status"] == "completed" and ci["conclusion"] == "success" and ci["run_attempt"] == 1
    jobs = get("actions/runs/38726727330/jobs?per_page=100")
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        found = [job for job in jobs["jobs"] if job["name"] == name]
        assert len(found) == 1 and found[0]["conclusion"] == "success"
    pr_ci = get("actions/runs/38726732452")
    pr_jobs = get("actions/runs/38726732452/jobs?per_page=100")
    assert pr_ci["head_sha"] == source and pr_ci["event"] == "pull_request"
    assert pr_ci["status"] == "completed" and pr_ci["conclusion"] == "success" and pr_ci["run_attempt"] == 1
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        found = [job for job in pr_jobs["jobs"] if job["name"] == name]
        assert len(found) == 1 and found[0]["conclusion"] == "success"
    remote = get("git/ref/heads/" + branch)
    assert remote["object"]["sha"] == source
    listing = get("actions/workflows/windows-candidate.yml/runs?head_sha=" + source + "&per_page=100")
    assert listing["total_count"] == 0 and listing["workflow_runs"] == []
    save("candidate-admission.original.json", {"atUtc": now(), "source": source, "ciRun": ci,
         "ciJobs": jobs, "prCiRun": pr_ci, "prCiJobs": pr_jobs, "remoteRef": remote, "priorCandidateRuns": listing,
         "purpose": "One stage candidate ready before a new human foreground window",
         "finalVersionGatePassed": False, "hostInstallerExecuted": False, "releaseRequested": False})
    payload = {"ref": branch, "inputs": {"purpose": "candidate", "admission_run": ""}}
    save("candidate-dispatch-payload.json", payload)
    save("candidate-dispatch-attempt.json", {"atUtc": now(), "source": source,
         "candidateRequestsAllowed": 1, "automaticRetryAllowed": False, "etwRerunRequested": False,
         "hostProductStarted": False, "hostInstallerExecuted": False, "releaseRequested": False})
    try:
        status, _ = api.request("actions/workflows/windows-candidate.yml/dispatches", token, payload)
        result = {"atUtc": now(), "httpStatus": status, "remoteReadbackRequired": True,
                  "automaticRetryAllowed": False}
    except Exception as error:
        result = {"atUtc": now(), "errorType": type(error).__name__, "remoteReadbackRequired": True,
                  "automaticRetryAllowed": False}
    save("candidate-dispatch-result.json", result)
    print(json.dumps(result))
elif mode in {"observe", "watch"}:
    assert (root / "candidate-dispatch-attempt.json").exists()
    duration = int(sys.argv[2]) if len(sys.argv) > 2 else (1 if mode == "observe" else 900)
    assert 1 <= duration <= 900
    deadline = time.monotonic() + duration
    last = None
    observations = 0
    terminal = False
    while True:
        try:
            value = observe()
            observations += 1
            terminal = value["terminal"]
            compact = {key: value.get(key) for key in ("runId", "status", "conclusion", "jobs")}
            fingerprint = json.dumps(compact, sort_keys=True)
            if fingerprint != last:
                if "snapshot" in value:
                    save("candidate-observation-" + stamp() + ".json", value["snapshot"])
                print(json.dumps({"atUtc": value["atUtc"], **compact}), flush=True)
                last = fingerprint
        except Exception as error:
            save("candidate-read-error-" + stamp() + ".json",
                 {"atUtc": now(), "errorType": type(error).__name__, "terminalInferred": False,
                  "dispatchRepeated": False})
        if terminal or time.monotonic() >= deadline:
            result = {"atUtc": now(), "observations": observations, "remoteRunTerminal": terminal,
                      "remoteRunStoppedOrRerun": False}
            save("candidate-watch-result-" + stamp() + ".json", result)
            print(json.dumps(result), flush=True)
            break
        time.sleep(min(30, max(0, deadline - time.monotonic())))
else:
    raise SystemExit("Expected request, observe or watch")
