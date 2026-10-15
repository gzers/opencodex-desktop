"""Fetch logs for verified completed runs only; retain exact original ZIP bytes."""
import gzip
import hashlib
import io
import json
import os
import re
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
files = sorted(root.glob("ci-terminal-*.json"))
assert files
prior = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n", text=True, capture_output=True,
        cwd=code, check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
    summaries = []
    for file in files:
        value = json.loads(file.read_text(encoding="utf-8-sig"))
        run = value["run"]
        assert run["head_sha"] == source and run["status"] == "completed"
        run_id = run["id"]
        summary_path = root / f"ci-log-summary-{run_id}.json"
        if summary_path.exists():
            summaries.append(json.loads(summary_path.read_text(encoding="utf-8")))
            continue
        archive_path = root / f"ci-original-logs-{run_id}.zip"
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/{run_id}/logs"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n')
        if not archive_path.exists():
            response = subprocess.run([
                r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--location", "--proto-redir", "=https", "--connect-timeout", "10",
                "--max-time", "45", "--fail", "--silent", "--show-error", "--output", str(archive_path)],
                input=config.encode(), capture_output=True, timeout=50)
            if response.returncode:
                (root / f"ci-log-download-failure-{run_id}.json").write_text(
                    json.dumps({"runId": run_id, "curlExit": response.returncode,
                                "error": response.stderr.decode(errors="replace")}, indent=2) + "\n", encoding="utf-8")
                raise RuntimeError("Log download failed; keep partial bytes, no CI rerun")
        raw = archive_path.read_bytes()
        selected = []
        counts, checkouts, failed_tests = {}, {}, []
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            assert archive.testzip() is None
            for item in archive.infolist():
                path = PurePosixPath(item.filename.replace("\\", "/"))
                assert not path.is_absolute() and ".." not in path.parts
                if item.is_dir():
                    continue
                is_checkout = (item.filename.startswith(("backend/", "backend-windows/"))
                               and re.fullmatch(r"\d+_Run actions_checkout@v5\.txt", path.name) is not None)
                is_test = (item.filename.startswith(("backend/", "backend-windows/"))
                           and ("regression" in item.filename.lower()
                                or "cargo test " in item.filename))
                if not is_checkout and not is_test:
                    continue
                data = archive.read(item)
                label = path.parts[0] + ("-checkout" if is_checkout else "-tests")
                destination = root / f"ci-{run_id}-{label}.log.gz"
                if destination.exists():
                    assert gzip.decompress(destination.read_bytes()) == data
                else:
                    destination.write_bytes(gzip.compress(data, mtime=0))
                text = data.decode("utf-8-sig", errors="strict")
                selected.append({"member": item.filename, "savedAs": destination.name,
                                 "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
                if is_checkout:
                    checkouts[path.parts[0]] = sorted(set(re.findall(r"\b[0-9a-f]{40}\b", text)))
                if is_test:
                    totals = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
                    counts[path.parts[0]] = {
                        "suites": len(totals), "passed": sum(int(row[0]) for row in totals),
                        "failed": sum(int(row[1]) for row in totals),
                        "ignored": sum(int(row[2]) for row in totals)}
                    failed_tests.extend(line for line in text.splitlines() if re.search(r"test .* \.\.\. FAILED", line))
        assert selected
        summary = {
            "atUtc": datetime.now(timezone.utc).isoformat(), "runId": run_id,
            "associatedHead": source, "event": run["event"], "conclusion": run["conclusion"],
            "archiveBytes": len(raw), "archiveSha256": hashlib.sha256(raw).hexdigest(),
            "zipCrcPassed": True, "safeEntries": True, "selectedOriginalLogs": selected,
            "actualCheckoutShas": checkouts, "rustCounts": counts, "failedTestLines": failed_tests,
            "collectorRecordOrScopeTestsExecutedByCi": False,
            "originalBd2FailedRunCleared": False,
            "authorizationHeaderForwardedAcrossHosts": False,
        }
        summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        summaries.append(summary)
    print(json.dumps([{"runId": item["runId"], "conclusion": item["conclusion"],
                       "rustCounts": item["rustCounts"]} for item in summaries]))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
