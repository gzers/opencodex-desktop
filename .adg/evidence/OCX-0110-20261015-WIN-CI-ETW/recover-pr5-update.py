"""One recovery only after a fresh authenticated read proves the earlier update is absent."""
import json
import os
import subprocess
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
assert not (root / "pr5-recovery-attempt.json").exists()
assert (root / "pr5-update-attempt.json").exists()
assert not (root / "pr5-update-response.original.json").exists()
payload = json.loads((root / "pr5-update-payload.json").read_text(encoding="utf-8"))
prior = json.loads((root / "pr5-after-update.original.json").read_text(encoding="utf-8"))
assert prior["title"] != payload["title"] and prior["body"] != payload["body"]
credential = subprocess.run(
    ["git", "credential", "fill"], text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def request(method="GET", value=None):
    config = (
        'url = "https://api.github.com/repos/gzers/opencodex-desktop/pulls/5"\n'
        f'request = "{method}"\nheader = "Authorization: Bearer {token}"\n'
        'header = "Accept: application/vnd.github+json"\n')
    if value is not None:
        config += 'header = "Content-Type: application/json"\n'
        config += 'data-binary = "@' + (root / "pr5-update-payload.json").as_posix() + '"\n'
    result = subprocess.run(
        [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--fail",
         "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30"],
        input=config.encode(), capture_output=True, timeout=35)
    if result.returncode:
        raise RuntimeError("Direct API request unavailable; no automatic mutation retry.")
    return result.stdout


before_bytes = request()
before = json.loads(before_bytes)
(root / "pr5-recovery-before.original.json").write_bytes(before_bytes)
assert before["head"]["sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
assert before["state"] == "open" and before["draft"] and not before["merged"]
assert before["title"] == prior["title"] and before["body"] == prior["body"]
(root / "pr5-recovery-attempt.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "earlierPatchResponseUnavailable": True, "twoIndependentReadbacksShowedOldBody": True,
    "credentialNotForwardedAcrossHosts": True, "automaticFurtherRetryAllowed": False,
}, indent=2) + "\n", encoding="utf-8")
try:
    updated_bytes = request("PATCH", payload)
    (root / "pr5-recovery-response.original.json").write_bytes(updated_bytes)
except Exception as error:
    (root / "pr5-recovery-request-error.json").write_text(
        json.dumps({"type": type(error).__name__, "automaticRetry": False}) + "\n", encoding="utf-8")
after_bytes = request()
(root / "pr5-recovery-after.original.json").write_bytes(after_bytes)
after = json.loads(after_bytes)
assert after["title"] == payload["title"] and after["body"] == payload["body"]
assert after["draft"] and not after["merged"] and after["auto_merge"] is None
assert after["head"]["sha"] == before["head"]["sha"]
print(json.dumps({"pr": 5, "freshReadBeforeRecoveryMatchedPrior": True,
                  "titleAndBodyReadbackExact": True, "draft": True, "merged": False}))
