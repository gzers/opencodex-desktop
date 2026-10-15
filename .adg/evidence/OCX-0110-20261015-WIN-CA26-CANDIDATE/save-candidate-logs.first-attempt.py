"""Retain the terminal job log ZIP and prove the actual candidate checkout."""
import gzip
import hashlib
import io
import json
import os
import re
import subprocess
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
archive_path = root / "candidate-original-job-logs.zip"
assert not archive_path.exists()
terminal = json.loads((root / "candidate-terminal.json").read_text(encoding="utf-8"))
assert terminal["run"]["status"] == "completed"
credential = subprocess.run(
    ["git", "credential", "fill"], text=True, capture_output=True,
    input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    env=dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never"))
token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
config = (
    'url = "https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/38663871103/logs"\n'
    f'header = "Authorization: Bearer {token}"\nheader = "Accept: application/vnd.github+json"\n')
response = subprocess.run(
    [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1", "--location",
     "--proto-redir", "=https", "--connect-timeout", "10", "--max-time", "45",
     "--fail", "--silent", "--show-error", "--max-filesize", "16777216", "--output", str(archive_path)],
    input=config.encode(), capture_output=True, timeout=50)
assert response.returncode == 0, "Preserve partial log archive; do not rerun the candidate."
raw = archive_path.read_bytes()
entries = []
checkout = None
with zipfile.ZipFile(io.BytesIO(raw)) as archive:
    assert archive.testzip() is None
    for entry in archive.infolist():
        path = PurePosixPath(entry.filename)
        assert not path.is_absolute() and ".." not in path.parts and "\\" not in entry.orig_filename
        if entry.is_dir() or not entry.filename.startswith("package/"):
            continue
        data = archive.read(entry)
        text = data.decode("utf-8-sig")
        saved_name = "candidate-job-" + path.name.replace(" ", "_") + ".gz"
        (root / saved_name).write_bytes(gzip.compress(data, mtime=0))
        entries.append({"member": entry.filename, "savedAs": saved_name, "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
        if re.fullmatch(r"\d+_Run actions_checkout@v5\.txt", path.name):
            match = re.search(r"log -1 --format=%H\s*\n[^\n]*?\b([0-9a-f]{40})\b", text)
            assert match
            checkout = match[1]
assert checkout == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
report = {"runId": 38663871103, "actualCheckout": checkout, "originalArchiveBytes": len(raw),
          "originalArchiveSha256": hashlib.sha256(raw).hexdigest(), "zipCrcPassed": True,
          "selectedOriginalJobLogs": entries, "candidateRerun": False}
(root / "candidate-job-log-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"actualCheckout": checkout, "zipCrcPassed": True, "originalJobLogs": len(entries)}))
