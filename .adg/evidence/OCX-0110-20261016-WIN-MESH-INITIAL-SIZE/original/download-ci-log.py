"""Read one completed job from the existing CI; never replay a workflow."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
WORKTREE = ROOT.parents[2]
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38891188021
JOBS = {"frontend": 116940815709, "backend": 116940815459, "backend-windows": 116940815642}
name = sys.argv[1]
job_id = JOBS[name]
spec = importlib.util.spec_from_file_location(
    "bounded_api", Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
                        "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, job = api.request(f"actions/jobs/{job_id}", token)
assert job["run_id"] == RUN and job["head_sha"] == SOURCE
assert job["status"] == "completed" and job["conclusion"] == "success"
canonical = ROOT / f"ci-{RUN}-{name}.original.log"
receipt = ROOT / f"ci-{RUN}-{name}.download.json"
if canonical.exists():
    assert receipt.exists(), "Inspect existing bytes instead of overwriting"
    record = json.loads(receipt.read_text(encoding="utf-8"))
    data = canonical.read_bytes()
    assert len(data) == record["bytes"] and hashlib.sha256(data).hexdigest() == record["sha256"]
    print(json.dumps({"job": name, "alreadyVerified": True}))
    sys.exit(0)

stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
attempt = ROOT / f"{name}-transfer-{stamp}.original.log"
opener = urllib.request.build_opener(api.NoRedirect())
url = api.BASE + f"actions/jobs/{job_id}/logs"
request = urllib.request.Request(url, headers={
    "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
    "User-Agent": "OpenCodex-bounded-ci-evidence",
})
hosts = ["api.github.com"]
try:
    try:
        with opener.open(request, timeout=25) as response:
            assert response.status == 200
            data = response.read(16 * 1024 * 1024 + 1)
            attempt.write_bytes(data)
    except urllib.error.HTTPError as redirect:
        assert redirect.code in (301, 302, 303, 307, 308)
        url = redirect.headers["Location"]
        redirect.close()
        parsed = urlparse(url)
        assert parsed.scheme == "https" and not parsed.username and not parsed.password
        assert parsed.hostname.endswith((".blob.core.windows.net", ".githubusercontent.com",
                                         ".actions.githubusercontent.com"))
        hosts.append(parsed.hostname)
        # The signed redirect is held in memory. No GitHub credentials follow it.
        config = "url = " + json.dumps(url) + "\n"
        result = subprocess.run([
            r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
            "--http1.1", "--tlsv1.2", "--silent", "--show-error",
            "--connect-timeout", "10", "--max-time", "45",
            "--max-filesize", str(16 * 1024 * 1024),
            "--output", str(attempt), "--write-out", "%{http_code}",
        ], input=config.encode(), capture_output=True, timeout=50)
        assert result.returncode == 0 and result.stdout == b"200", (
            f"Anonymous log read failed: curl {result.returncode}, HTTP "
            + result.stdout.decode(errors="replace"))
        data = attempt.read_bytes()
    assert 0 < len(data) <= 16 * 1024 * 1024
    text = data.decode("utf-8-sig")
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    _, commit = api.request("git/commits/" + checkout, token)
    local_tree = subprocess.check_output(["git", "rev-parse", SOURCE + "^{tree}"],
                                         cwd=WORKTREE).decode().strip()
    assert commit["tree"]["sha"] == local_tree, "Do not admit another source tree"
    commit_path = ROOT / f"checkout-{checkout}.original.json"
    if commit_path.exists():
        assert json.loads(commit_path.read_text())["tree"]["sha"] == local_tree
    else:
        commit_path.write_bytes((json.dumps(commit, indent=2) + "\n").encode())
    attempt.rename(canonical)
    record = {
        "runId": RUN, "jobId": job_id, "job": name, "headSource": SOURCE,
        "actualCheckout": checkout, "actualTree": commit["tree"]["sha"],
        "wholeTreeEquivalentToHead": True, "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "certificateVerificationEnabled": True,
        "redirectCredentialsForwarded": False, "hosts": hosts,
        "workflowReexecuted": False,
    }
    receipt.write_bytes((json.dumps(record, indent=2) + "\n").encode())
    print(json.dumps(record))
except Exception as error:
    failure = ROOT / f"{name}-log-read-failure-{stamp}.json"
    failure.write_bytes((json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "runId": RUN, "jobId": job_id,
        "exceptionType": type(error).__name__, "message": str(error),
        "partialFile": attempt.name, "hosts": hosts,
        "credentialsForwardedToRedirect": False, "workflowReexecuted": False,
    }, indent=2) + "\n").encode())
    raise
