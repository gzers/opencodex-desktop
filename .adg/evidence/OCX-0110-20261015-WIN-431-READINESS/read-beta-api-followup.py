"""Read the beta Release once after the earlier transport error; never publish."""
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import urllib.error
import urllib.request

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FIRST-CI-ETW"
spec = importlib.util.spec_from_file_location("api", prior / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
target = root / "beta-api-followup.json"
assert not target.exists()
url = api.BASE + "releases/tags/beta"
token = api.credential()
started = datetime.now(timezone.utc).isoformat()
request = urllib.request.Request(url, headers={
    "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
    "User-Agent": "OpenCodex-readonly-source-readiness",
})
result = {"requestStartedUtc": started, "url": url, "authenticated": True,
          "releaseOrSourceChanged": False, "productCheckInvoked": False, "betaSixHourStarted": False}
try:
    try:
        response = urllib.request.build_opener(api.NoRedirect()).open(request, timeout=20)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        body = response.read(1024 * 1024 + 1)
        status = response.code
    assert len(body) <= 1024 * 1024
    with (root / "beta-api-followup-body.original.txt").open("xb") as stream:
        stream.write(body)
    result.update(statusCode=status, bodyBytes=len(body), bodySha256=hashlib.sha256(body).hexdigest(),
                  followedRedirect=False, credentialsOrHeadersSaved=False)
except Exception as error:
    result.update(errorType=type(error).__name__, absenceInferred=False)
result["observedAtUtc"] = datetime.now(timezone.utc).isoformat()
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
