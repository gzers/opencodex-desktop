"""Admit the first successful exact-source runs from their original job logs."""
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("mesh_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
admit = load("ci_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
terminal = json.loads((ROOT / "ci-terminal.json").read_text(encoding="utf-8"))
assert len(terminal["runs"]) == 2
assert all(r["conclusion"] == "success" and r["run_attempt"] == 1 for r in terminal["runs"])
token = api.credential()
_, head = api.request(f"git/commits/{SOURCE}", token)
(ROOT / "head-commit.original.json").write_text(json.dumps(head, indent=2) + "\n", encoding="utf-8")
reports = []
for row in terminal["runs"]:
    identifier = row["id"]
    _, run = api.request(f"actions/runs/{identifier}", token)
    _, jobs = api.request(f"actions/runs/{identifier}/jobs?per_page=100", token)
    assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert len(jobs["jobs"]) == jobs["total_count"]
    assert all(j["conclusion"] == "success" for j in jobs["jobs"])
    (ROOT / f"ci-{identifier}-jobs.original.json").write_text(
        json.dumps(jobs, indent=2) + "\n", encoding="utf-8")
    report = {"id": identifier, "event": run["event"], "jobs": []}
    for name in ("frontend", "backend", "backend-windows"):
        job, = [j for j in jobs["jobs"] if j["name"] == name]
        path = ROOT / f"ci-{identifier}-{name}.original.log"
        receipt = path.with_suffix(".download.json")
        if not path.exists():
            downloaded = admit.download(api.BASE + f"actions/jobs/{job['id']}/logs",
                                         path, 16 * 1024 * 1024, token=token)
            receipt.write_text(json.dumps(downloaded, indent=2) + "\n", encoding="utf-8")
        assert receipt.exists(), "Partial download is not a valid original log"
        downloaded = json.loads(receipt.read_text(encoding="utf-8"))
        data = path.read_bytes()
        assert len(data) == downloaded["bytes"]
        assert hashlib.sha256(data).hexdigest() == downloaded["sha256"]
        text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        _, commit = api.request(f"git/commits/{checkout}", token)
        (ROOT / f"checkout-{checkout}.original.json").write_text(
            json.dumps(commit, indent=2) + "\n", encoding="utf-8")
        assert commit["tree"]["sha"] == head["tree"]["sha"]
        if name == "frontend":
            assert re.search(r"Test Files\s+99 passed", text)
            assert re.search(r"Tests\s+535 passed", text)
            counts = {"files": 99, "passed": 535, "failed": 0}
        else:
            suites = [tuple(map(int, values)) for values in re.findall(
                r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
            assert len(suites) == 35 and sum(x[1] for x in suites) == 0
            assert sum(x[0] for x in suites) == (923 if name == "backend" else 648)
            assert sum(x[2] for x in suites) == (6 if name == "backend" else 7)
            counts = {"suites": 35, "passed": sum(x[0] for x in suites),
                      "failed": 0, "ignored": sum(x[2] for x in suites)}
        report["jobs"].append({"name": name, "id": job["id"], "actualCheckout": checkout,
                               "tree": commit["tree"]["sha"], "counts": counts,
                               "log": path.name, **downloaded})
        (ROOT / "ci-collection-progress.json").write_text(json.dumps({
            "source": SOURCE, "run": identifier, "lastCompletedJob": name,
            "sameRunIdsRetained": True}, indent=2) + "\n", encoding="utf-8")
    reports.append(report)
result = {"source": SOURCE, "tree": head["tree"]["sha"], "runs": reports,
          "actualCheckoutTreesEqual": True, "runAttempt": 1,
          "historicalQueryFailureCleared": False, "memoryGatePassed": False}
(ROOT / "ci-log-verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
