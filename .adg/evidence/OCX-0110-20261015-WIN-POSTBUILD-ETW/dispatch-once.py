"""Dispatch one changed-condition capture after exact CI and original-log admission."""
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
attempt_path = root / "dispatch-attempt.json"
assert not attempt_path.exists(), "Read back the previous dispatch before any new mutation."
push_id, pr_id = 38699864356, 38699869579
for run_id, event in ((push_id, "push"), (pr_id, "pull_request")):
    terminal = read(f"run-{run_id}-terminal.original.json")
    run = terminal["run"]
    assert run["head_sha"] == api.SOURCE and run["event"] == event
    assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
    jobs = terminal["jobs"]["jobs"]
    for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
        found = [job for job in jobs if job["name"] == name]
        assert len(found) == 1 and found[0]["status"] == "completed" and found[0]["conclusion"] == "success"
    logs = read(f"run-{run_id}-log-verification.json")
    assert logs["safeEntriesAndCrcVerified"] and not logs["crossHostCredentialsForwarded"]
    if event == "push":
        assert logs["actualCheckout"]["backend-windows"] == logs["actualCheckout"]["backend"] == api.SOURCE
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=api.CODE).decode().strip() == api.SOURCE
assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=api.CODE).strip()
token = api.credential()
_, branch = api.request("git/ref/heads/feature/0.1.10-windows-query-etw", token)
assert branch["object"]["sha"] == api.SOURCE
_, live_ci = api.request("actions/runs/" + str(push_id), token)
assert live_ci["status"] == "completed" and live_ci["conclusion"] == "success" and live_ci["run_attempt"] == 1
payload = {
    "ref": "feature/0.1.10-windows-query-etw",
    "inputs": {"purpose": "query-etw", "admission_run": str(push_id),
               "diagnostic_path": "post-build-runner-temp"},
}
(root / "dispatch-payload.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
attempt = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": api.SOURCE,
    "authorization": "Standing human approval for revised administrator background ETW and original failure-evidence goal",
    "changedCondition": "Normal original-application build stage before first execution; owned runner-TEMP volume and inherited ephemeral runner profile",
    "previousSuccessfulReplayRepeatedUnchanged": False,
    "originalFailedArtifact": 11881042603, "originalLibraryRunsAllowed": 1,
    "fixtureDeadlinesAndDefaultConcurrencyChanged": False, "automaticMutationRetryAllowed": False,
    "artificialStressOrPrewarm": False, "desktopOrInstallerRequested": False,
    "historicalFailureCleared": False, "releaseRequested": False,
}
attempt_path.write_text(json.dumps(attempt, indent=2) + "\n", encoding="utf-8")
try:
    status, response = api.request("actions/workflows/windows-candidate.yml/dispatches", token, payload)
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "httpStatus": status,
              "response": response, "remoteRunMustBeReadBack": True, "automaticRetryAllowed": False}
except Exception as error:
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "errorType": type(error).__name__,
              "httpStatus": getattr(error, "code", None), "remoteRunMustBeReadBack": True,
              "automaticRetryAllowed": False}
(root / "dispatch-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
