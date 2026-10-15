"""Bounded source-readiness observation only, never a product check or a real cycle."""
import hashlib
import json
import os
import subprocess
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=code).decode().strip() == source
defaults_path = "apps/desktop/tauri/config/runtime.defaults.json"
defaults = subprocess.check_output(["git", "show", f"{source}:{defaults_path}"], cwd=code)
endpoint = json.loads(defaults)["updates"]["desktop"]["channels"]["beta"]
assert endpoint == "https://github.com/gzers/opencodex-desktop/releases/download/beta/latest.json"
credential = subprocess.run(
    ["git", "credential", "fill"], cwd=code, input="protocol=https\nhost=github.com\n\n",
    text=True, capture_output=True, check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


opener = urllib.request.build_opener(NoRedirect())
observations = []
for kind, url, authenticated in (
    ("authenticated-api", "https://api.github.com/repos/gzers/opencodex-desktop/releases/tags/beta", True),
    ("public-metadata", endpoint, False),
):
    headers = {"User-Agent": "OpenCodex-readiness-evidence"}
    if authenticated:
        headers.update({"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json"})
    started = datetime.now(timezone.utc).isoformat()
    request = urllib.request.Request(url, headers=headers)
    try:
        try:
            response = opener.open(request, timeout=20)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = response.read(1024 * 1024 + 1)
            assert len(body) <= 1024 * 1024
            status = response.code
        with (root / f"beta-{kind}-body.original.txt").open("xb") as stream:
            stream.write(body)
        observations.append({
            "kind": kind, "url": url, "requestStartedUtc": started,
            "responseObservedUtc": datetime.now(timezone.utc).isoformat(),
            "authenticatedRequest": authenticated, "statusCode": status,
            "bodyBytes": len(body), "bodySha256": hashlib.sha256(body).hexdigest(),
            "followedRedirect": False, "credentialOrHeadersSaved": False})
    except Exception as error:
        observations.append({"kind": kind, "url": url, "requestStartedUtc": started,
                             "errorType": type(error).__name__, "terminalOrAbsenceInferred": False})
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
    "runtimeDefaultsGitBlob": subprocess.check_output(
        ["git", "rev-parse", f"{source}:{defaults_path}"], cwd=code).decode().strip(),
    "observations": observations,
    "bothEndpointsObserved404": all(row.get("statusCode") == 404 for row in observations),
    "scope": "Metadata availability prerequisite only; no product or scheduling proof",
    "productCheckInvoked": False, "beta6hStarted": False, "sourceOrClockChanged": False,
    "betaSourcePublished": False, "realCycleVerified": False, "fullVersionGatePassed": False,
}
with (root / "beta-source-prerequisite.json").open("x", encoding="utf-8") as stream:
    json.dump(report, stream, indent=2)
    stream.write("\n")
print(json.dumps(report))
