"""Read exact GitHub API objects through a command-local verified SOCKS route."""
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("original_ci_api_identity",
    ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
original = importlib.util.module_from_spec(spec)
spec.loader.exec_module(original)
BASE, CODE, credential = original.BASE, original.CODE, original.credential
PROXY = "socks5h://[::1]:3060"
CURL = r"D:\Software\Develop\Git\mingw64\bin\curl.exe"


def request(path, token, payload=None, method=None):
    assert payload is None and method in (None, "GET"), "Read-only API transport"
    assert not path.startswith(("/", "http")) and ".." not in path.split("/")
    assert all(character not in path and character not in token for character in "\r\n")
    config = "\n".join([
        "url = " + json.dumps(BASE + path),
        "header = " + json.dumps("Authorization: Bearer " + token),
        'header = "Accept: application/vnd.github+json"',
    ]) + "\n"
    result = subprocess.run([
        CURL, "-q", "--config", "-", "--proxy", PROXY, "--noproxy", "",
        "--http1.1", "--tlsv1.2", "--tls-max", "1.2", "--silent", "--show-error",
        "--connect-timeout", "10", "--max-time", "25",
        "--max-filesize", str(6 * 1024 * 1024), "--write-out", "\n%{http_code}",
    ], input=config.encode(), capture_output=True, timeout=30)
    body, _, status = result.stdout.rpartition(b"\n")
    if result.returncode or status != b"200":
        raise RuntimeError(f"Read-only proxied API failed (curl {result.returncode}, HTTP {status.decode(errors='replace')}); no mutation requested")
    assert len(body) <= 6 * 1024 * 1024
    return 200, json.loads(body)
