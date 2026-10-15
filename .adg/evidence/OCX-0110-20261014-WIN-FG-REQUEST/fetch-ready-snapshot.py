"""Read uploaded candidate bytes before cache-only post-step finishes, without claiming job success."""
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
                                input="protocol=https\nhost=github.com\n\n", check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def get(path):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
        output = subprocess.run([r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                                 "--connect-timeout", "10", "--max-time", "25", "--fail",
                                 "--silent", "--show-error"], input=config.encode(), capture_output=True, timeout=30)
        if output.returncode:
            raise RuntimeError("Read-only snapshot unavailable; no dispatch or restart.")
        return json.loads(output.stdout)

    run = get("actions/runs/38591604133")
    jobs = get("actions/runs/38591604133/jobs?per_page=100")
    artifacts = get("actions/runs/38591604133/artifacts?per_page=100")
    assert run["head_sha"] == "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
    assert run["event"] == "workflow_dispatch"
    assert len(jobs["jobs"]) == 1
    required = {
        "Windows regression with the package lock",
        "Build NSIS and MSI candidates",
        "Install, verify final EXE and uninstall on isolated runner",
        "Upload verified Windows installers",
        "Preserve package evidence",
    }
    steps = {row["name"]: row for row in jobs["jobs"][0]["steps"]}
    assert all(steps[name]["status"] == "completed" and steps[name]["conclusion"] == "success" for name in required)
    assert not any(row["status"] == "completed" and row["conclusion"] == "failure" for row in steps.values())
    if run["status"] != "completed":
        assert all(row["name"].startswith("Post ") for row in steps.values() if row["status"] == "in_progress")
    assert artifacts["total_count"] >= 2
    value = {"atUtc": datetime.now(timezone.utc).isoformat(), "run": run, "jobs": jobs, "artifacts": artifacts,
             "candidateProductStepsAndUploadCompleted": True,
             "wholeRunSuccessClaimed": run["status"] == "completed" and run["conclusion"] == "success"}
    path = root / "candidate-artifact-ready.json"
    assert not path.exists()
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"runId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
                      "candidateProductStepsAndUploadCompleted": True,
                      "artifacts": [{"id": row["id"], "name": row["name"]} for row in artifacts["artifacts"]]}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
