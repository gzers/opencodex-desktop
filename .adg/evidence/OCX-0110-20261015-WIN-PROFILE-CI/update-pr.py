"""Apply the prepared title/body once; read back before any further write."""
import json
import os
import subprocess
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, input="protocol=https\nhost=github.com\n\n",
    text=True, capture_output=True, check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(row.split("=", 1) for row in credential.stdout.splitlines() if "=" in row)["password"]


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def request(method="GET", data=None):
    for attempt in range(3 if method == "GET" else 1):
        try:
            req = urllib.request.Request(
                "https://api.github.com/repos/gzers/opencodex-desktop/pulls/5",
                method=method, data=data,
                headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json",
                         "Content-Type": "application/json", "User-Agent": "OpenCodex-acceptance-evidence"})
            with urllib.request.urlopen(req, timeout=20) as response:
                return json.load(response)
        except Exception as error:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
            save(f"pr-transport-error-{stamp}.json", {
                "atUtc": datetime.now(timezone.utc).isoformat(), "method": method,
                "errorType": type(error).__name__, "readAttempt": attempt + 1,
                "writeAutomaticallyRepeated": False})
            if method != "GET" or attempt == 2:
                raise


assert not (root / "pr-update-attempt.json").exists(), "Read back an earlier attempt; never repeat blindly."
before = request()
assert before["head"]["sha"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert before["draft"] and not before["merged"] and before["auto_merge"] is None
save("pr-before-update.original.json", before)
save("pr-update-attempt.json", {"atUtc": datetime.now(timezone.utc).isoformat(),
                              "method": "PATCH title and body only",
                              "mergeRequested": False, "automaticRepeatAllowed": False})
try:
    response = request("PATCH", (root / "pr-update-request.json").read_bytes())
    save("pr-update-response.original.json", response)
    after = request()
    save("pr-after-update.original.json", after)
    payload = json.loads((root / "pr-update-request.json").read_text(encoding="utf-8"))
    assert all(after[key] == value for key, value in payload.items())
    assert after["head"]["sha"] == before["head"]["sha"] and after["base"]["sha"] == before["base"]["sha"]
    assert after["draft"] and not after["merged"] and after["state"] == "open" and after["auto_merge"] is None
    print(json.dumps({"prNumber": 5, "head": after["head"]["sha"], "metadataReadBackMatched": True,
                      "draft": True, "merged": False}))
except Exception as error:
    save("pr-update-error.json", {"atUtc": datetime.now(timezone.utc).isoformat(),
                                "errorType": type(error).__name__,
                                "readBackRequiredBeforeAnyRetry": True})
    raise SystemExit("PR metadata result incomplete; read back before any retry.")
