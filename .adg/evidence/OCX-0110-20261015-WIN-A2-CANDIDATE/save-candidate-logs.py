"""Save original terminal package logs and independently verify exact checkout."""
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import zipfile

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FIRST-CI-ETW"
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
source = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
terminal = json.loads((root / "candidate-terminal.json").read_text(encoding="utf-8-sig"))
assert terminal["run"]["id"] == 38729632145 and terminal["run"]["head_sha"] == source
assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
archive_path = root / "candidate-job-logs.original.zip"
if not archive_path.exists():
    admit.download(api.BASE + "actions/runs/38729632145/logs", archive_path, 16 * 1024 * 1024,
                   token=api.credential())
checkout = None
entries = []
selected = ("checkout", "Windows_regression", "Windows regression", "Build_NSIS", "Build NSIS",
            "Install,_verify", "Install, verify", "Verify_process", "Verify process", "Read_candidate",
            "Read candidate", "Freeze_and", "Freeze and")
counts = None
with zipfile.ZipFile(archive_path) as archive:
    admit.checked_zip(archive)
    for member in archive.namelist():
        if not member.startswith("package/") or not member.endswith(".txt"):
            continue
        name = member.split("/")[-1]
        if not any(part in name for part in selected):
            continue
        data = archive.read(member)
        text = data.decode("utf-8-sig")
        saved = "candidate-job-" + re.sub(r"[^A-Za-z0-9_.-]", "_", name) + ".gz"
        target = root / saved
        compressed = gzip.compress(data, mtime=0)
        if target.exists():
            assert target.read_bytes() == compressed
        else:
            with target.open("xb") as stream:
                stream.write(compressed)
        entries.append({"member": member, "file": saved, "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
        if "checkout" in name:
            matches = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
            assert len(matches) <= 1
            if matches:
                assert checkout is None
                checkout = matches[0]
        if "Windows regression with the package lock" in name:
            totals = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
            assert len(totals) == 35
            counts = {"suites": len(totals), "passed": sum(int(row[0]) for row in totals),
                      "failed": sum(int(row[1]) for row in totals), "ignored": sum(int(row[2]) for row in totals)}
assert checkout == source
assert counts == {"suites": 35, "passed": 643, "failed": 0, "ignored": 7}
report = {
    "source": source, "runId": 38729632145, "actualCheckout": checkout, "rustResults": counts,
    "archiveBytes": archive_path.stat().st_size, "archiveSha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
    "safePathsAndCrcVerified": True, "selectedLogs": entries, "candidateRerun": False,
    "hostInstallerExecuted": False, "crossHostCredentialsForwarded": False,
}
target = root / "candidate-log-verification.json"
assert not target.exists()
target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": source, "runId": 38729632145, "actualCheckout": checkout,
                  "rustResults": counts, "originalLogsSelected": len(entries)}))
