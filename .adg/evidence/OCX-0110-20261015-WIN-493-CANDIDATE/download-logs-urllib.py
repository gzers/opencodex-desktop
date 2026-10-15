"""Recover only the missing terminal log ZIP; no redirect receives an API credential."""
import hashlib
import json
import os
import subprocess
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
target = root / "candidate-original-job-logs.zip"
assert not target.exists()
terminal = json.loads((root / "candidate-terminal.json").read_text(encoding="utf-8"))
assert terminal["run"]["id"] == 38673288645 and terminal["run"]["conclusion"] == "success"
credential = subprocess.run(
    ["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
    text=True, capture_output=True, check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


opener = urllib.request.build_opener(NoRedirect())
failures = []
for attempt in range(3):
    try:
        request = urllib.request.Request(
            "https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/38673288645/logs",
            headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json",
                     "User-Agent": "OpenCodex-log-evidence"})
        try:
            response = opener.open(request, timeout=20)
        except urllib.error.HTTPError as error:
            assert error.code in {301, 302, 303, 307, 308}
            location = error.headers["Location"]
            parsed = urllib.parse.urlparse(location)
            assert parsed.scheme == "https" and parsed.hostname
            assert parsed.hostname.endswith((".blob.core.windows.net", ".actions.githubusercontent.com"))
            error.close()
            response = opener.open(urllib.request.Request(
                location, headers={"User-Agent": "OpenCodex-log-evidence"}), timeout=20)
        with response:
            assert response.status == 200
            size = 0
            with target.open("xb") as stream:
                while data := response.read(65536):
                    size += len(data)
                    assert size <= 16 * 1024 * 1024
                    stream.write(data)
        report = {
            "atUtc": datetime.now(timezone.utc).isoformat(), "runId": 38673288645,
            "status": "downloaded-crc-and-contents-pending", "bytes": size,
            "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
            "readAttempts": attempt + 1, "priorReadErrors": failures,
            "crossHostAuthorizationForwarded": False, "tlsValidationDisabled": False,
            "ciRerunRequested": False, "applicationStarted": False, "installerExecuted": False}
        with (root / "candidate-urllib-log-download.json").open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
        print(json.dumps(report))
        break
    except Exception as error:
        failures.append({"atUtc": datetime.now(timezone.utc).isoformat(), "errorType": type(error).__name__,
                         "partialArchiveRetained": target.exists(), "signedRedirectOrHeadersSaved": False})
        if target.exists() or attempt == 2:
            with (root / "urllib-log-read-error.json").open("x", encoding="utf-8") as stream:
                json.dump({"failures": failures, "ciRerunRequested": False, "automaticOverwrite": False}, stream, indent=2)
                stream.write("\n")
            raise SystemExit("Log read unavailable; partial bytes preserved, no candidate rerun.")
