"""Same exact-head logs through the verified IPv6 local SOCKS listener."""
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
PROXY = "socks5h://[::1]:3060"
terminal = json.loads((ROOT / "ci-terminal.original.json").read_text(encoding="utf-8"))
assert terminal["run"]["id"] == 38858859494 and terminal["run"]["head_sha"] == SOURCE
assert terminal["run"]["conclusion"] == "success" and terminal["run"]["run_attempt"] == 1
spec = importlib.util.spec_from_file_location("proxy_log_reader",
    ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
curl = r"D:\Software\Develop\Git\mingw64\bin\curl.exe"
for name in ("frontend", "backend", "backend-windows"):
    job, = [j for j in terminal["jobs"]["jobs"] if j["name"] == name]
    canonical = ROOT / f"ci-38858859494-{name}.original.log"
    receipt = canonical.with_suffix(".download.json")
    if canonical.exists():
        assert receipt.exists(), "Never overwrite an incomplete canonical log"
        data = canonical.read_bytes()
        expected = json.loads(receipt.read_text(encoding="utf-8"))
        assert len(data) == expected["bytes"] and hashlib.sha256(data).hexdigest() == expected["sha256"]
        continue
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    attempt = ROOT / f"{name}-proxy-transfer-{stamp}.original.log"
    assert not attempt.exists()
    url = api.BASE + f"actions/jobs/{job['id']}/logs"
    hosts = []
    for hop in range(4):
        parsed = urlparse(url)
        assert parsed.scheme == "https" and not parsed.username and not parsed.password
        assert parsed.hostname == "api.github.com" if hop == 0 else parsed.hostname.endswith(
            (".blob.core.windows.net", ".githubusercontent.com", ".actions.githubusercontent.com"))
        hosts.append(parsed.hostname)
        config = ["url = " + json.dumps(url)]
        if hop == 0:
            config += ["header = " + json.dumps("Authorization: Bearer " + token),
                       'header = "Accept: application/vnd.github+json"']
        result = subprocess.run([
            curl, "-q", "--config", "-", "--proxy", PROXY, "--noproxy", "",
            "--http1.1", "--tlsv1.2", "--silent", "--show-error",
            "--connect-timeout", "10", "--max-time", "45", "--max-filesize", str(16 * 1024 * 1024),
            "--output", str(attempt), "--dump-header", "-", "--write-out", "\nSTATUS:%{http_code}\n",
        ], input=("\n".join(config) + "\n").encode(), capture_output=True, timeout=50)
        headers = result.stdout.decode("iso-8859-1")
        statuses = re.findall(r"\nSTATUS:(\d{3})", headers)
        status = int(statuses[-1]) if statuses else 0
        if result.returncode:
            (ROOT / f"{name}-proxy-failure-{stamp}.json").write_bytes((json.dumps({
                "jobId": job["id"], "source": SOURCE, "curlExitCode": result.returncode,
                "httpStatus": status, "host": parsed.hostname, "proxy": PROXY,
                "partialFile": attempt.name, "credentialsForwardedToRedirect": False,
                "systemSettingsChanged": False,
            }, indent=2) + "\n").encode())
            raise RuntimeError(f"Proxy read failed for {name}; preserve this attempt")
        if status == 200:
            break
        assert status in (301, 302, 303, 307, 308), f"Unexpected HTTP status {status}"
        locations = re.findall(r"(?im)^location:\s*([^\r\n]+)", headers)
        assert len(locations) == 1
        url = locations[0].strip()
    else:
        raise RuntimeError("Too many authenticated-to-anonymous log redirects")
    data = attempt.read_bytes()
    assert 0 < len(data) <= 16 * 1024 * 1024
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE, "Another checkout cannot admit this recording"
    attempt.rename(canonical)
    receipt.write_bytes((json.dumps({
        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
        "certificateVerificationEnabled": True, "redirectAuthorizationForwarded": False,
        "transport": "Git curl command-local SOCKS5 remote DNS", "proxy": PROXY, "hosts": hosts,
        "systemSettingsChanged": False,
    }, indent=2) + "\n").encode())
    print(json.dumps({"job": name, "jobId": job["id"], "bytes": len(data), "actualCheckout": checkout}))
