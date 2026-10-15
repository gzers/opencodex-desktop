"""Retain the package job's primary log without manufacturing a run-wide ZIP."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
terminal = json.loads((root / "candidate-terminal.json").read_text(encoding="utf-8-sig"))
run = terminal["run"]
assert run["id"] == 38737628201 and run["head_sha"] == api.SOURCE
assert api.SOURCE == "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
assert run["status"] == "completed" and run["conclusion"] == "success" and run["run_attempt"] == 1
job, = [item for item in terminal["jobs"]["jobs"] if item["name"] == "package"]
assert job["status"] == "completed" and job["conclusion"] == "success"
target = root / "package-job.original.log"
if not target.exists():
    admit.download(api.BASE + f"actions/jobs/{job['id']}/logs", target, 32 * 1024 * 1024,
                   token=api.credential())
data = target.read_bytes()
text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert actual == api.SOURCE
counts = [tuple(map(int, row)) for row in re.findall(
    r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
totals = {"suites": len(counts), "passed": sum(row[0] for row in counts),
          "failed": sum(row[1] for row in counts), "ignored": sum(row[2] for row in counts)}
assert totals == {"suites": 35, "passed": 648, "failed": 0, "ignored": 7}
report = {
    "source": api.SOURCE, "runId": run["id"], "jobId": job["id"], "actualCheckout": actual,
    "rustResults": totals, "logFormat": "individual-job-original", "file": target.name,
    "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
    "archiveFabricated": False, "candidateRerun": False, "hostInstallerExecuted": False,
    "crossHostCredentialsForwarded": False,
}
destination = root / "candidate-log-verification.json"
if destination.exists():
    assert json.loads(destination.read_text()) == report
else:
    destination.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
