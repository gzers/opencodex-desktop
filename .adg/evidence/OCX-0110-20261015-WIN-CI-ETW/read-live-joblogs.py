"""Try the documented job-log download for the same live handles; a 404 is not terminal."""
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
snapshot = json.loads(max(root.glob("observation-*.json")).read_text(encoding="utf-8"))
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
rows = []
for number in (38660621496, 38660625647):
    jobs = snapshot["reads"]["jobs-" + str(number)]["jobs"]
    job = next(row for row in jobs if row["name"] == "backend-windows")
    target = root / f"live-windows-job-{job['id']}.original.log"
    assert not target.exists()
    config = (
        f'url = "https://api.github.com/repos/gzers/opencodex-desktop/actions/jobs/{job["id"]}/logs"\n'
        f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
    response = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--location",
         "--proto-redir", "=https", "--fail", "--silent", "--show-error",
         "--connect-timeout", "10", "--max-time", "30", "--max-filesize", "16777216",
         "--output", str(target), "--write-out", "%{http_code}"],
        input=config.encode(), capture_output=True, timeout=35)
    row = {"runId": number, "jobId": job["id"], "curlExit": response.returncode,
           "httpStatus": response.stdout.decode(errors="replace").strip(),
           "remoteStatusInPriorSnapshot": job["status"], "remoteStoppedClaimed": False}
    if target.exists() and response.returncode == 0:
        text = target.read_text(encoding="utf-8-sig", errors="replace")
        row["bytes"] = target.stat().st_size
        row["lastLines"] = text.splitlines()[-12:]
    rows.append(row)
report = {"atUtc": datetime.now(timezone.utc).isoformat(), "requests": rows, "mutations": 0}
(root / "live-job-log-attempt.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
