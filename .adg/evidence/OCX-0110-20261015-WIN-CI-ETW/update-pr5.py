"""One prepared PR description mutation, followed by an independent readback; no blind retry."""
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
attempt = root / "pr5-update-attempt.json"
assert not attempt.exists(), "Read the existing result; do not repeat an uncertain mutation."
body = (root / "pr5-description.md").read_text(encoding="utf-8")
payload = {"title": "test(windows): 固定失败制品并支持有界CI ETW取证", "body": body}
(root / "pr5-update-payload.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


def request(method="GET", data_path=None):
    config = (
        'url = "https://api.github.com/repos/gzers/opencodex-desktop/pulls/5"\n'
        f'request = "{method}"\nheader = "Authorization: Bearer {token}"\n'
        'header = "Accept: application/vnd.github+json"\n')
    if data_path:
        config += 'header = "Content-Type: application/json"\n'
        config += 'data-binary = "@' + data_path.as_posix() + '"\n'
    result = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
         "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30"],
        input=config.encode(), capture_output=True, timeout=35)
    return result


before = request()
assert before.returncode == 0
pr = json.loads(before.stdout)
assert pr["state"] == "open" and pr["draft"] and not pr["merged"]
assert pr["head"]["sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
(root / "pr5-before-update.original.json").write_bytes(before.stdout)
attempt.write_text(json.dumps({"atUtc": datetime.now(timezone.utc).isoformat(), "method": "PATCH",
                              "authorizedScope": "existing Windows diagnostic draft PR description",
                              "mergeRequested": False, "blindRetryAllowed": False}, indent=2) + "\n", encoding="utf-8")
result = request("PATCH", root / "pr5-update-payload.json")
if result.returncode == 0:
    (root / "pr5-update-response.original.json").write_bytes(result.stdout)
after = request()
if after.returncode:
    raise SystemExit("PR update readback unavailable; no automatic retry.")
(root / "pr5-after-update.original.json").write_bytes(after.stdout)
pr = json.loads(after.stdout)
assert pr["title"] == payload["title"] and pr["body"] == payload["body"]
assert pr["state"] == "open" and pr["draft"] and not pr["merged"] and pr["auto_merge"] is None
assert pr["head"]["sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
print(json.dumps({"pr": 5, "descriptionReadbackMatched": True, "draft": True, "merged": False}))
