"""One exact-head diagnostic request after completed CI and raw checkout-log verification."""
import gzip
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
sha = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
attempt_path = root / "diagnostic-dispatch-attempt.json"
assert not attempt_path.exists(), "Read back the existing request instead of repeating a possible dispatch."
terminal = json.loads((root / "ci-terminal-38660621496.json").read_text(encoding="utf-8"))
assert terminal["run"]["head_sha"] == sha and terminal["run"]["event"] == "push"
assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
for name in ("frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"):
    matches = [row for row in terminal["jobs"]["jobs"] if row["name"] == name]
    assert len(matches) == 1 and matches[0]["conclusion"] == "success"
log = gzip.decompress((root / "ci-38660621496-backend-windows-checkout.log.gz").read_bytes()).decode("utf-8-sig")
match = re.search(r"log -1 --format=%H\s*\n[^\n]*?\b([0-9a-f]{40})\b", log)
assert match and match[1] == sha, "Actual Windows CI checkout is not the admitted collector source."
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=code).decode().strip() == sha
subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "test/smoke", ".github/workflows"],
               cwd=code, check=True)
payload = {
    "ref": "feature/0.1.10-windows-query-etw",
    "inputs": {"purpose": "query-etw", "admission_run": "38660621496"},
}
payload_path = root / "diagnostic-dispatch-payload.json"
payload_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
attempt = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "collectorSource": sha, "admissionRun": 38660621496,
    "authorization": "批准修正版后续都可以管理员后台 ETW 录制; original bounded failure-evidence goal",
    "scope": "one fixed bd2 original library on ephemeral Windows CI, no native desktop/package",
    "originalLibraryRunsAllowed": 1, "automaticMutationRetryAllowed": False,
    "releaseRequested": False, "localDesktopStarted": False,
}
attempt_path.write_text(json.dumps(attempt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
config = (
    'url = "https://api.github.com/repos/gzers/opencodex-desktop/actions/workflows/windows-candidate.yml/dispatches"\n'
    'request = "POST"\n'
    f'header = "Authorization: Bearer {token}"\n'
    'header = "Accept: application/vnd.github+json"\n'
    'header = "Content-Type: application/json"\n'
    'data-binary = "@' + payload_path.as_posix() + '"\n')
try:
    response = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
         "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
         "--output", str(root / "diagnostic-dispatch-response.body.original.txt"),
         "--write-out", "%{http_code}"],
        input=config.encode(), capture_output=True, timeout=35)
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "curlExit": response.returncode,
              "httpStatus": response.stdout.decode(errors="replace").strip(),
              "remoteRunMustBeReadBack": True, "automaticRetryAllowed": False}
except subprocess.TimeoutExpired:
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "observationTimeout": True,
              "remoteRunMustBeReadBack": True, "automaticRetryAllowed": False}
(root / "diagnostic-dispatch-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
