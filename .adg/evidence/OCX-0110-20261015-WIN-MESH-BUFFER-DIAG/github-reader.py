"""Read fallback with Git's bundled curl; never weaken TLS or retry mutations."""
import importlib.util
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "original_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
original = importlib.util.module_from_spec(spec)
spec.loader.exec_module(original)
BASE, CODE, credential = original.BASE, original.CODE, original.credential

def request(path, token, payload=None, method=None):
    try:
        return original.request(path, token, payload, method)
    except Exception:
        if payload is not None or method not in (None, "GET"):
            raise
        assert not path.startswith(("/", "http")) and ".." not in path.split("/")
        assert "\n" not in token and "\r" not in token
        config = "\n".join([
            "url = " + json.dumps(BASE + path),
            "header = " + json.dumps("Authorization: Bearer " + token),
            'header = "Accept: application/vnd.github+json"',
        ]) + "\n"
        result = subprocess.run([
            r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
            "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
            "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "25",
            "--max-filesize", str(6 * 1024 * 1024), "--write-out", "\n%{http_code}",
        ], input=config.encode(), capture_output=True, timeout=30)
        if result.returncode:
            raise RuntimeError("Read-only Git curl fallback failed; no remote state changed")
        body, _, status = result.stdout.rpartition(b"\n")
        assert status == b"200" and len(body) <= 6 * 1024 * 1024
        return 200, json.loads(body)
