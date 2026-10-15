"""Prove actual retention harness checkouts and keep install-step originals."""
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("common", api.CODE / "test/smoke/windows-etw-ci-admit.py")
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
terminal = json.loads((root / "retention-terminal.json").read_text(encoding="utf-8-sig"))
run_id = terminal["run"]["id"]
assert terminal["run"]["head_sha"] == api.SOURCE and terminal["run"]["status"] == "completed"
archive_path = root / "retention-job-logs.original.zip"
if not archive_path.exists():
    common.download(api.BASE + f"actions/runs/{run_id}/logs", archive_path, 16 * 1024 * 1024,
                    token=api.credential())
entries, checkouts = [], {}
with zipfile.ZipFile(archive_path) as archive:
    common.checked_zip(archive)
    for member in archive.namelist():
        job, _, name = member.partition("/")
        if not job.startswith("retention (") or not name.endswith(".txt"):
            continue
        if not any(part in name for part in ("checkout", "Admit exact harness", "Verify installer scope",
                                             "Install 019, upgrade")):
            continue
        data = archive.read(member)
        text = data.decode("utf-8-sig")
        saved = "retention-job-" + re.sub(r"[^A-Za-z0-9_.-]", "_", member) + ".gz"
        compressed = gzip.compress(data, mtime=0)
        if (root / saved).exists():
            assert (root / saved).read_bytes() == compressed
        else:
            with (root / saved).open("xb") as stream:
                stream.write(compressed)
        entries.append({"member": member, "savedAs": saved, "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
        if "checkout" in name:
            matches = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
            assert len(matches) <= 1
            if matches:
                assert job not in checkouts
                checkouts[job] = matches[0]
assert set(checkouts) == {"retention (msi)", "retention (nsis)"}
assert set(checkouts.values()) == {api.SOURCE}
jobs = [job for job in terminal["jobs"]["jobs"] if job["name"].startswith("retention (")]
assert len(jobs) == 2 and len({job["runner_id"] for job in jobs}) == 2
report = {"runId": run_id, "harnessSource": api.SOURCE, "actualCheckouts": checkouts,
          "distinctRunnerIds": [job["runner_id"] for job in jobs],
          "archiveBytes": archive_path.stat().st_size,
          "archiveSha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
          "safePathsAndCrcVerified": True, "entries": entries,
          "runRepeated": False, "crossHostCredentialsForwarded": False}
target = root / "retention-job-log-verification.json"
assert not target.exists()
target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": run_id, "actualCheckouts": checkouts,
                  "distinctRunnersVerified": True, "selectedLogs": len(entries)}))
