"""Read the remaining completed jobs from the already successful CI run."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38891188021
JOBS = {
    "release-tools": 116940815575,
    "windows-smoke-evidence": 116940815876,
}
spec = importlib.util.spec_from_file_location(
    "bounded_api", Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
                        "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()


for name, job_id in JOBS.items():
    _, job = api.request(f"actions/jobs/{job_id}", token)
    assert job["run_id"] == RUN and job["head_sha"] == SOURCE
    assert job["status"] == "completed" and job["conclusion"] == "success"
    canonical = ROOT / f"ci-{RUN}-{name}.original.log"
    receipt = ROOT / f"ci-{RUN}-{name}.download.json"
    if canonical.exists():
        assert receipt.exists()
        data = canonical.read_bytes()
        saved = json.loads(receipt.read_text(encoding="utf-8"))
        assert len(data) == saved["bytes"] and hashlib.sha256(data).hexdigest() == saved["sha256"]
        continue
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    attempt = ROOT / f"{name}-transfer-{stamp}.original.log"
    url = api.BASE + f"actions/jobs/{job_id}/logs"
    parsed_hosts = ["api.github.com"]
    config = (
        "url = " + json.dumps(url) + "\n"
        "header = " + json.dumps("Authorization: Bearer " + token) + "\n"
        "header = " + json.dumps("Accept: application/vnd.github+json") + "\n"
    )
    result = subprocess.run([
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "--http1.1", "--tlsv1.2", "--tls-max", "1.2", "--silent", "--show-error",
        "--connect-timeout", "10", "--max-time", "45",
        "--max-filesize", str(16 * 1024 * 1024),
        "--output", str(attempt), "--dump-header", "-", "--write-out", "\nSTATUS:%{http_code}\n",
    ], input=config.encode(), capture_output=True, timeout=50)
    headers = result.stdout.decode("iso-8859-1")
    status_match = re.findall(r"\nSTATUS:(\d{3})", headers)
    status = int(status_match[-1]) if status_match else 0
    if result.returncode:
        (ROOT / f"{name}-log-read-failure-{stamp}.json").write_bytes(
            (json.dumps({
                "jobId": job_id, "returnCode": result.returncode, "httpStatus": status,
                "host": parsed_hosts[-1], "partialFile": attempt.name,
                "credentialsForwardedToRedirect": False,
            }, indent=2) + "\n").encode())
        raise RuntimeError(f"{name} log transfer failed")
    if status in (301, 302, 303, 307, 308):
        locations = re.findall(r"(?im)^location:\s*([^\r\n]+)", headers)
        assert len(locations) == 1
        redirect = locations[0].strip()
        parsed = urlparse(redirect)
        assert parsed.scheme == "https" and not parsed.username and not parsed.password
        assert parsed.hostname.endswith((".blob.core.windows.net", ".githubusercontent.com",
                                         ".actions.githubusercontent.com"))
        parsed_hosts.append(parsed.hostname)
        result = subprocess.run([
            r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
            "--http1.1", "--tlsv1.2", "--tls-max", "1.2", "--silent", "--show-error",
            "--connect-timeout", "10", "--max-time", "45",
            "--max-filesize", str(16 * 1024 * 1024),
            "--output", str(attempt), "--write-out", "%{http_code}",
        ], input=("url = " + json.dumps(redirect) + "\n").encode(),
            capture_output=True, timeout=50)
        assert result.returncode == 0 and result.stdout == b"200"
    else:
        assert status == 200
    data = attempt.read_bytes()
    assert 0 < len(data) <= 16 * 1024 * 1024
    text = data.decode("utf-8-sig")
    checkout_matches = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout_matches and all(value == SOURCE for value in checkout_matches)
    attempt.rename(canonical)
    receipt.write_bytes((json.dumps({
        "runId": RUN, "jobId": job_id, "job": name, "actualCheckout": SOURCE,
        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
        "certificateVerificationEnabled": True, "redirectCredentialsForwarded": False,
        "hosts": parsed_hosts, "workflowReexecuted": False,
    }, indent=2) + "\n").encode())
    print(json.dumps({"job": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}))
