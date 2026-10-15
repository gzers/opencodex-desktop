"""Save a terminal run's originals and extract exact checkout/test evidence."""
from datetime import datetime, timezone
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("etw_admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
run_id = int(sys.argv[1])
token = api.credential()
_, run = api.request("actions/runs/" + str(run_id), token)
_, jobs = api.request("actions/runs/" + str(run_id) + "/jobs?per_page=100", token)
assert run["head_sha"] == api.SOURCE and run["status"] == "completed" and run["run_attempt"] == 1
prefix = "run-" + str(run_id)
terminal_path = root / (prefix + "-terminal.original.json")
if terminal_path.exists():
    previous = json.loads(terminal_path.read_text())
    assert previous["run"]["id"] == run_id and previous["run"]["head_sha"] == api.SOURCE
    assert previous["run"]["status"] == "completed" and previous["run"]["conclusion"] == run["conclusion"]
else:
    terminal_path.write_text(
        json.dumps({"atUtc": datetime.now(timezone.utc).isoformat(), "run": run, "jobs": jobs}, indent=2) + "\n",
        encoding="utf-8")
archive_path = root / (prefix + "-logs.original.zip")
if archive_path.exists():
    # Reuse only the complete original ZIP after an offline-parser failure.
    with zipfile.ZipFile(archive_path) as archive:
        admit.checked_zip(archive)
    download = {"bytes": archive_path.stat().st_size,
                "sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest()}
else:
    download = admit.download(api.BASE + "actions/runs/" + str(run_id) + "/logs",
                              archive_path, 32 * 1024 * 1024, token=token)
selected, checkout, counts = [], {}, {}
with zipfile.ZipFile(archive_path) as archive:
    admit.checked_zip(archive)
    for member in archive.namelist():
        job, _, name = member.partition("/")
        if job not in {"backend", "backend-windows", "query-etw"} or not name.endswith(".txt"):
            continue
        if "checkout" not in name and not any(part in name for part in (
                "Windows backend regression", "cargo test", "Prepare the original",
                "Record one original", "Verify collector", "Admit exact-head")):
            continue
        data = archive.read(member)
        target = prefix + "-" + job + "-" + re.sub(r"[^A-Za-z0-9_.-]", "_", name) + ".gz"
        destination = root / target
        if destination.exists():
            assert gzip.decompress(destination.read_bytes()) == data
        else:
            with destination.open("xb") as stream:
                stream.write(gzip.compress(data, mtime=0))
        selected.append({"member": member, "file": target, "bytes": len(data),
                         "sha256": hashlib.sha256(data).hexdigest()})
        text = data.decode("utf-8-sig")
        if "checkout" in name:
            matches = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
            assert len(matches) <= 1, member
            if matches:
                assert job not in checkout
                checkout[job] = matches[0]
        matches = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if matches:
            numbers = [tuple(map(int, row)) for row in matches]
            counts[job] = {"suites": len(numbers), "passed": sum(row[0] for row in numbers),
                           "failed": sum(row[1] for row in numbers), "ignored": sum(row[2] for row in numbers)}
summary = {
    "runId": run_id, "source": api.SOURCE, "event": run["event"], "status": run["status"],
    "conclusion": run["conclusion"], "logs": download, "safeEntriesAndCrcVerified": True,
    "actualCheckout": checkout, "rustResults": counts, "selectedLogs": selected,
    "crossHostCredentialsForwarded": False, "historicalFailureCleared": False,
}
(root / (prefix + "-log-verification.json")).write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
