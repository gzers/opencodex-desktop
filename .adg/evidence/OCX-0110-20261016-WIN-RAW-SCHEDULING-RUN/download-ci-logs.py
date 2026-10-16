"""Bounded verified-TLS reads; GitHub credentials never follow a redirect."""
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
assert not (ROOT / "manifest.json").exists()
SOURCE = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
terminal = json.loads((ROOT / "ci-terminal.original.json").read_text(encoding="utf-8"))
assert terminal["run"]["head_sha"] == SOURCE
assert terminal["run"]["conclusion"] == "success" and terminal["run"]["run_attempt"] == 1
spec = importlib.util.spec_from_file_location("log_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
curl = r"D:\Software\Develop\Git\mingw64\bin\curl.exe"
for name in ("frontend", "backend", "backend-windows"):
    job, = [j for j in terminal["jobs"]["jobs"] if j["name"] == name]
    canonical = ROOT / f"ci-38858859494-{name}.original.log"
    receipt = canonical.with_suffix(".download.json")
    if canonical.exists():
        assert receipt.exists(), "Existing partial log must not be overwritten"
        data = canonical.read_bytes()
        expected = json.loads(receipt.read_text(encoding="utf-8"))
        assert len(data) == expected["bytes"] and hashlib.sha256(data).hexdigest() == expected["sha256"]
        continue
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    attempt = ROOT / f"{name}-transfer-{stamp}.original.log"
    assert not attempt.exists()
    url = api.BASE + f"actions/jobs/{job['id']}/logs"
    hosts = []
    for hop in range(4):
        parsed = urlparse(url)
        assert parsed.scheme == "https" and not parsed.username and not parsed.password
        if hop == 0:
            assert parsed.hostname == "api.github.com"
        else:
            assert parsed.hostname.endswith((".blob.core.windows.net", ".githubusercontent.com", ".actions.githubusercontent.com"))
        hosts.append(parsed.hostname)
        config = ["url = " + json.dumps(url)]
        if hop == 0:
            config += ["header = " + json.dumps("Authorization: Bearer " + token),
                       'header = "Accept: application/vnd.github+json"']
        result = subprocess.run([
            curl, "-q", "--config", "-", "-4", "--http1.1",
            "--tlsv1.2", "--tls-max", "1.2", "--silent", "--show-error",
            "--connect-timeout", "10", "--max-time", "45", "--max-filesize", str(16 * 1024 * 1024),
            "--output", str(attempt), "--dump-header", "-", "--write-out", "\nSTATUS:%{http_code}\n",
        ], input=("\n".join(config) + "\n").encode(), capture_output=True, timeout=50)
        headers = result.stdout.decode("iso-8859-1")
        status = re.findall(r"\nSTATUS:(\d{3})", headers)
        status = int(status[-1]) if status else 0
        if result.returncode:
            (ROOT / f"{name}-transfer-failure-{stamp}.json").write_bytes((json.dumps({
                "stage": "read-only-ci-job-log", "jobId": job["id"], "returnCode": result.returncode,
                "httpStatus": status, "host": parsed.hostname, "partialFile": attempt.name,
                "credentialsForwardedToRedirect": False, "source": SOURCE,
            }, indent=2) + "\n").encode())
            raise RuntimeError(f"Read-only {name} log transfer failed ({result.returncode}); partial retained")
        if status == 200:
            break
        assert status in (301, 302, 303, 307, 308)
        locations = re.findall(r"(?im)^location:\s*([^\r\n]+)", headers)
        assert len(locations) == 1
        url = locations[0].strip()
    else:
        raise RuntimeError("Too many anonymous log redirects")
    data = attempt.read_bytes()
    assert 0 < len(data) <= 16 * 1024 * 1024
    text = data.decode("utf-8-sig")
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE, "Do not admit another checkout's log"
    attempt.rename(canonical)
    record = {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
              "certificateVerificationEnabled": True, "redirectAuthorizationForwarded": False,
              "transport": "Git bundled curl, anonymous explicit redirect", "hosts": hosts}
    receipt.write_bytes((json.dumps(record, indent=2) + "\n").encode())
    print(json.dumps({"job": name, "bytes": len(data), "actualCheckout": checkout,
                      "redirectCredentialsForwarded": False}))
