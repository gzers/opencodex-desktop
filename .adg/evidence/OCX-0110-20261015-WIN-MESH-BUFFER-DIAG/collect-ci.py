import hashlib
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "8be87e2d54a895d4fe02e31300100a1ffed90121"
spec = importlib.util.spec_from_file_location("ghread", ROOT / "github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
token = api.credential()
terminal = json.loads((ROOT / "ci-terminal.json").read_text())
assert all(row["conclusion"] == "success" for row in terminal["runs"])
summary = []
for identifier in (38763397150, 38763409850):
    _, run = api.request(f"actions/runs/{identifier}", token)
    _, jobs = api.request(f"actions/runs/{identifier}/jobs?per_page=100", token)
    assert run["head_sha"] == SOURCE and run["status"] == "completed" and run["conclusion"] == "success"
    assert run["run_attempt"] == 1 and len(jobs["jobs"]) == jobs["total_count"]
    (ROOT / f"run-{identifier}-jobs-terminal.original.json").write_text(json.dumps(jobs, indent=2) + "\n")
    report = {"run": identifier, "event": run["event"], "head": SOURCE, "jobs": []}
    for name in ("backend", "backend-windows"):
        job, = [item for item in jobs["jobs"] if item["name"] == name]
        assert job["conclusion"] == "success"
        path = ROOT / f"run-{identifier}-job-{job['id']}.original.log"
        receipt_path = ROOT / f"run-{identifier}-job-{job['id']}-download.json"
        if not path.exists():
            download = admit.download(api.BASE + f"actions/jobs/{job['id']}/logs",
                                      path, 16 * 1024 * 1024, token=token)
            receipt_path.write_text(json.dumps(download, indent=2) + "\n")
        else:
            assert receipt_path.exists(), "Partial original log retained; do not treat it as complete"
        data = path.read_bytes()
        download = json.loads(receipt_path.read_text())
        assert len(data) == download["bytes"] and hashlib.sha256(data).hexdigest() == download["sha256"]
        text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        counts = [tuple(map(int, row)) for row in re.findall(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
        assert len(counts) == 35
        assert sum(row[0] for row in counts) == (923 if name == "backend" else 648)
        assert sum(row[2] for row in counts) == (6 if name == "backend" else 7)
        _, commit = api.request(f"git/commits/{checkout}", token)
        (ROOT / f"checkout-{checkout}.original.json").write_text(json.dumps(commit, indent=2) + "\n")
        report["jobs"].append({
            "job": name, "id": job["id"], "actualCheckout": checkout, "tree": commit["tree"]["sha"],
            "suites": len(counts), "passed": sum(row[0] for row in counts),
            "failed": sum(row[1] for row in counts), "ignored": sum(row[2] for row in counts),
            "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "originalLog": path.name,
        })
    summary.append(report)
_, head = api.request(f"git/commits/{SOURCE}", token)
(ROOT / "head-commit.original.json").write_text(json.dumps(head, indent=2) + "\n")
assert all(job["tree"] == head["tree"]["sha"] and job["failed"] == 0 for report in summary for job in report["jobs"])
(ROOT / "ci-log-verification.json").write_text(json.dumps({
    "head": SOURCE, "headTree": head["tree"]["sha"], "runs": summary,
    "actualCheckoutTreesEqual": True, "historicalQueryFailureCleared": False,
    "runWideZipFabricated": False,
}, indent=2) + "\n")
print(json.dumps(summary))
