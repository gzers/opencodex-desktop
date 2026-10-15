"""One installer-only dispatch; subsequent reads retain its exact run handle."""
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
token = api.credential()
source = api.SOURCE
assert source == "3c51367f1be89abf66f69805d29b2a660a570a2c"
branch = "feature/0.1.10-windows-query-etw"
now = lambda: datetime.now(timezone.utc).isoformat()
stamp = lambda: datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")


def save(name, value):
    with (root / name).open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def get(path):
    return api.request(path, token)[1]


def observe():
    binding = root / "retention-run-binding.json"
    if not binding.exists():
        listing = get("actions/workflows/windows-candidate.yml/runs?head_sha=" + source + "&per_page=100")
        assert listing["total_count"] == len(listing["workflow_runs"]) <= 1
        if not listing["workflow_runs"]:
            return {"visible": False, "terminal": False, "atUtc": now()}
        save(binding.name, {"runId": listing["workflow_runs"][0]["id"], "harnessSource": source})
    run_id = json.loads(binding.read_text())["runId"]
    run = get(f"actions/runs/{run_id}")
    jobs = get(f"actions/runs/{run_id}/jobs?per_page=100")
    artifacts = get(f"actions/runs/{run_id}/artifacts?per_page=100")
    assert run["head_sha"] == source and run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
    assert jobs["total_count"] == len(jobs["jobs"])
    assert artifacts["total_count"] == len(artifacts["artifacts"])
    for job in jobs["jobs"]:
        if job["name"] in {"package", "query-etw"}:
            assert job["conclusion"] == "skipped"
    snapshot = {"atUtc": now(), "run": run, "jobs": jobs, "artifacts": artifacts,
                "harnessSource": source, "fixedCandidateSource": "43142bdbcc7a0b9f1325103febae3f44dce407ab",
                "hostProductOrInstallerExecuted": False}
    terminal = run["status"] == "completed"
    if terminal and not (root / "retention-terminal.json").exists():
        save("retention-terminal.json", snapshot)
    return {"snapshot": snapshot, "atUtc": snapshot["atUtc"], "runId": run_id,
            "status": run["status"], "conclusion": run["conclusion"], "terminal": terminal,
            "jobs": [{"name": job["name"], "status": job["status"], "conclusion": job["conclusion"],
                      "activeSteps": [step["name"] for step in job["steps"] if step["status"] == "in_progress"]}
                     for job in jobs["jobs"]]}


mode = sys.argv[1]
if mode == "request":
    assert not (root / "dispatch-attempt.json").exists()
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=api.CODE).decode().strip() == source
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=api.CODE).strip()
    for run_id, event in ((38713455586, "push"), (38713465220, "pull_request")):
        terminal = json.loads((root / f"run-{run_id}-terminal.original.json").read_text())
        run = terminal["run"]
        assert run["head_sha"] == source and run["event"] == event
        assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
        for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
            matches = [job for job in terminal["jobs"]["jobs"] if job["name"] == name]
            assert len(matches) == 1 and matches[0]["conclusion"] == "success"
        logs = json.loads((root / f"run-{run_id}-log-verification.json").read_text())
        assert logs["safeEntriesAndCrcVerified"]
        if event == "push":
            assert logs["actualCheckout"]["backend-windows"] == source
    remote = get("git/ref/heads/" + branch)
    assert remote["object"]["sha"] == source
    previous = get("actions/workflows/windows-candidate.yml/runs?head_sha=" + source + "&per_page=100")
    assert previous["total_count"] == 0 and previous["workflow_runs"] == []
    save("dispatch-admission.original.json", {"atUtc": now(), "remoteRef": remote, "previousRuns": previous})
    payload = {"ref": branch, "inputs": {"purpose": "installer-retention", "admission_run": "38713455586"}}
    save("dispatch-payload.json", payload)
    save("dispatch-attempt.json", {
        "atUtc": now(), "harnessSource": source, "fixedCandidateSource": "43142bdbcc7a0b9f1325103febae3f44dce407ab",
        "requestsAllowed": 1, "freshRunnerPerInstallerKind": True, "automaticRetryAllowed": False,
        "desktopOrPerformanceRequested": False, "hostInstallerExecuted": False, "releaseRequested": False})
    try:
        status, _ = api.request("actions/workflows/windows-candidate.yml/dispatches", token, payload)
        result = {"atUtc": now(), "httpStatus": status, "remoteReadbackRequired": True, "automaticRetryAllowed": False}
    except Exception as error:
        result = {"atUtc": now(), "errorType": type(error).__name__, "remoteReadbackRequired": True,
                  "automaticRetryAllowed": False}
    save("dispatch-result.json", result)
    print(json.dumps(result))
elif mode in {"observe", "watch"}:
    assert (root / "dispatch-attempt.json").exists()
    duration = int(sys.argv[2]) if len(sys.argv) > 2 else 600
    assert 1 <= duration <= 900
    deadline = time.monotonic() + duration
    previous = None
    terminal = False
    while True:
        try:
            value = observe()
            terminal = value["terminal"]
            compact = {key: value.get(key) for key in ("runId", "status", "conclusion", "jobs")}
            fingerprint = json.dumps(compact, sort_keys=True)
            if fingerprint != previous:
                if "snapshot" in value:
                    save("retention-observation-" + stamp() + ".json", value["snapshot"])
                print(json.dumps({"atUtc": value["atUtc"], **compact}), flush=True)
                previous = fingerprint
        except Exception as error:
            save("retention-read-error-" + stamp() + ".json",
                 {"atUtc": now(), "type": type(error).__name__, "terminalInferred": False,
                  "dispatchRepeated": False})
        if terminal or time.monotonic() >= deadline:
            result = {"atUtc": now(), "remoteRunTerminal": terminal, "remoteRunStoppedOrRerun": False}
            save("retention-watch-result-" + stamp() + ".json", result)
            print(json.dumps(result), flush=True)
            break
        time.sleep(min(30, max(0, deadline - time.monotonic())))
else:
    raise SystemExit("Expected request or watch")
