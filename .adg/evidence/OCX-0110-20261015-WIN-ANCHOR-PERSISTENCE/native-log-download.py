"""Read-only TLS fallback: never forward GitHub authorization across hosts."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import urllib.parse
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
run_id = int(sys.argv[1])
assert run_id in {38734133095, 38734138996}
assert sys.argv[2:] in ([], ["git-curl"])
curl = (r"D:\Software\Develop\Git\mingw64\bin\curl.exe" if sys.argv[2:] else
        r"C:\Windows\System32\curl.exe")
target = root / f"run-{run_id}-logs.original.zip"
partial = target.with_suffix(".native-partial")
assert not target.exists() and not partial.exists()
token = api.credential()
assert "\r" not in token and "\n" not in token
config = "\n".join([
    "url = " + json.dumps(api.BASE + f"actions/runs/{run_id}/logs"),
    "header = " + json.dumps("Authorization: Bearer " + token),
    'header = "Accept: application/vnd.github+json"',
]) + "\n"
result = subprocess.run(
    [curl, "-q", "--config", "-", "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
     "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "25",
     "--dump-header", "-", "--output", "NUL"],
    input=config.encode(), capture_output=True, timeout=30)
assert result.returncode == 0, f"Read transport failed (curl {result.returncode}); do not expose response URLs"
headers = result.stdout.decode("iso-8859-1")
assert re.findall(r"HTTP/\S+ (\d+)", headers)[-1] == "302"
locations = re.findall(r"^location:\s*(.+)\r?$", headers, re.I | re.M)
assert len(locations) == 1
url = locations[0].strip()
parsed = urllib.parse.urlparse(url)
assert parsed.scheme == "https" and parsed.hostname and not parsed.username and not parsed.password
# No Authorization header is included in this second request.
result = subprocess.run(
    [curl, "-q", "--config", "-", "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
     "--silent", "--show-error", "--proto", "=https", "--connect-timeout", "10",
     "--max-time", "45", "--max-filesize", str(32 * 1024 * 1024),
     "--write-out", "%{http_code}", "--output", str(partial)],
    input=("url = " + json.dumps(url) + "\n").encode(), capture_output=True, timeout=50)
assert result.returncode == 0 and result.stdout == b"200", "Download incomplete; inspect retained partial"
assert partial.stat().st_size <= 32 * 1024 * 1024
with zipfile.ZipFile(partial) as archive:
    assert archive.testzip() is None
partial.rename(target)
report = {"runId": run_id, "bytes": target.stat().st_size,
          "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
          "tlsCertificateChecksEnabled": True, "crossHostAuthorizationForwarded": False,
          "remoteRunRestarted": False, "curlExecutable": curl}
(root / f"run-{run_id}-native-download.json").write_text(
    json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
