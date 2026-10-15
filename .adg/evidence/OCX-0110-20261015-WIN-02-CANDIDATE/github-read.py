"""Bounded GitHub reads; credentials stay in memory and redirects are rejected."""
import json
import os
from pathlib import Path
import subprocess
import urllib.error
import urllib.request

SOURCE = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
BASE = "https://api.github.com/repos/gzers/opencodex-desktop/"
ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def credential():
    result = subprocess.run(
        ["git", "credential", "fill"], cwd=CODE, input="protocol=https\nhost=github.com\n\n",
        text=True, capture_output=True, check=True, timeout=20,
        env={**os.environ, "GIT_TERMINAL_PROMPT": "0", "GCM_INTERACTIVE": "never"})
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)["password"]


def request(path, token, payload=None, method=None):
    assert not path.startswith(("/", "http")) and ".." not in path.split("/")
    headers = {"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
               "User-Agent": "OpenCodex-bounded-postbuild-diagnostic"}
    data = None
    if payload is not None:
        data = json.dumps(payload).encode()
        headers["Content-Type"] = "application/json"
    query = urllib.request.Request(BASE + path, headers=headers, data=data, method=method)
    try:
        with urllib.request.build_opener(NoRedirect()).open(query, timeout=25) as response:
            body = response.read(6 * 1024 * 1024 + 1)
            assert len(body) <= 6 * 1024 * 1024
            return response.status, json.loads(body) if body else None
    except urllib.error.HTTPError:
        raise
    except urllib.error.URLError:
        # Only retry a read. A possibly accepted mutation must be read back,
        # never automatically repeated. Native curl keeps certificate checks.
        if payload is not None or method not in (None, "GET"):
            raise
        config = (
            'url = "' + BASE + path + '"\n'
            'header = "Authorization: Bearer ' + token + '"\n'
            'header = "Accept: application/vnd.github+json"\n'
        )
        result = subprocess.run(
            [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
             "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "25",
             "--write-out", "\n%{http_code}"], input=config.encode(), capture_output=True, timeout=30)
        if result.returncode:
            raise RuntimeError("Native read transport failed without changing remote state")
        body, _, status = result.stdout.rpartition(b"\n")
        assert len(body) <= 6 * 1024 * 1024 and status == b"200"
        return 200, json.loads(body)
