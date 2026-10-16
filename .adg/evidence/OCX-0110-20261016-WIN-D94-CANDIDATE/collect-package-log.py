"""Bind the successful package job's actual checkout and original test counts."""
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"
terminal = json.loads((ROOT / "candidate-terminal.json").read_text(encoding="utf-8"))
assert terminal["run"]["head_sha"] == SOURCE and terminal["run"]["conclusion"] == "success"
job, = [j for j in terminal["jobs"]["jobs"] if j["name"] == "package"]
assert job["conclusion"] == "success"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("package_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
admit = load("package_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
path = ROOT / "package-job.original.log"
receipt = ROOT / "package-job-download.json"
if not path.exists():
    result = admit.download(api.BASE + f"actions/jobs/{job['id']}/logs",
                             path, 16 * 1024 * 1024, token=api.credential())
    receipt.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
assert receipt.exists(), "Preserve an incomplete download without accepting it"
data = path.read_bytes()
download = json.loads(receipt.read_text(encoding="utf-8"))
assert len(data) == download["bytes"] and hashlib.sha256(data).hexdigest() == download["sha256"]
text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
counts = [tuple(map(int, values)) for values in re.findall(
    r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
assert len(counts) == 35 and sum(v[0] for v in counts) == 648
assert sum(v[1] for v in counts) == 0 and sum(v[2] for v in counts) == 7
result = {"source": SOURCE, "runId": terminal["run"]["id"], "jobId": job["id"],
          "actualCheckout": checkout, "suites": 35, "passed": 648, "failed": 0,
          "ignored": 7, "log": path.name, **download, "hostInstallerExecuted": False}
(ROOT / "package-log-verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
