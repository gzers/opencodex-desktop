"""Read one accepted package run; optionally verify/extract inert ZIP payloads."""
import hashlib
import importlib.util
import json
import re
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38893610415
DESTINATION = REPO / f"test/software/candidate-b4d487-{RUN}"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("artifact_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("safe_zip_reader", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
_, run = api.request(f"actions/runs/{RUN}", token)
_, jobs = api.request(f"actions/runs/{RUN}/jobs?per_page=100", token)
_, artifacts = api.request(f"actions/runs/{RUN}/artifacts?per_page=100", token)
assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
assert run["event"] == "workflow_dispatch" and run["path"] == ".github/workflows/windows-candidate.yml"
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
observed = {"run": run, "jobs": jobs, "artifacts": artifacts}
snapshot = ROOT / f"candidate-progress-{stamp}.original.json"
snapshot.write_bytes((json.dumps(observed, ensure_ascii=False, indent=2) + "\n").encode())
summary = {
    "runId": RUN, "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{"id": j["id"], "name": j["name"], "status": j["status"], "conclusion": j["conclusion"],
              "runningSteps": [s["name"] for s in j["steps"] if s["status"] == "in_progress"]}
             for j in jobs["jobs"]],
    "artifacts": [{"id": a["id"], "name": a["name"], "bytes": a["size_in_bytes"],
                   "digest": a["digest"]} for a in artifacts["artifacts"]],
    "newDispatchRequested": False, "hostProductStarted": False,
}
print(json.dumps(summary), flush=True)
if sys.argv[1:] != ["--retrieve"]:
    sys.exit(0)
assert run["status"] == "completed" and run["conclusion"] == "success", "Do not retrieve an incomplete/failed package as admitted"
assert [j["conclusion"] for j in jobs["jobs"] if j["name"] == "package"] == ["success"]
assert all(j["conclusion"] == "skipped" for j in jobs["jobs"] if j["name"] in ("query-etw", "retention"))
terminal = ROOT / "candidate-terminal.original.json"
if terminal.exists():
    previous = json.loads(terminal.read_text(encoding="utf-8"))
    assert previous["run"]["id"] == RUN and previous["run"]["head_sha"] == SOURCE
    assert previous["run"]["conclusion"] == "success"
else:
    terminal.write_bytes(snapshot.read_bytes())
assert not (ROOT / "candidate-download-result.json").exists(), "Inspect completed retrieval instead of repeating it"
assert not DESTINATION.is_symlink() and not DESTINATION.is_junction()
DESTINATION.mkdir(parents=True, exist_ok=True)
assert DESTINATION.resolve().is_relative_to((REPO / "test/software").resolve())


def fetch(url, target, cap, expected=None):
    if target.exists():
        assert expected and hashlib.sha256(target.read_bytes()).hexdigest() == expected
        return target.read_bytes()
    attempt = target.with_name(target.name + ".transfer-" + stamp)
    assert not attempt.exists()
    try:
        safe.download(url, attempt, cap, token=token, expected=expected)
    except Exception as error:
        failure = ROOT / (target.name + ".failure-" + stamp + ".json")
        failure.write_bytes((json.dumps({
            "atUtc": datetime.now(timezone.utc).isoformat(), "runId": RUN,
            "exceptionType": type(error).__name__, "message": str(error),
            "partialPath": str(attempt), "retryIsReadOnly": True,
            "workflowReexecuted": False,
        }, indent=2) + "\n").encode())
        raise
    attempt.rename(target)
    return target.read_bytes()


package_job, = [j for j in jobs["jobs"] if j["name"] == "package"]
log_path = ROOT / "candidate-package.original.log"
if log_path.exists():
    log = log_path.read_bytes()
else:
    log = fetch(api.BASE + f"actions/jobs/{package_job['id']}/logs", log_path, 16 * 1024 * 1024)
text = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", log.decode("utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
counts = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
assert len(counts) == 35
assert tuple(sum(int(row[i]) for row in counts) for i in range(3)) == (650, 0, 7)
(ROOT / "candidate-package-log-verification.json").write_bytes((json.dumps({
    "source": SOURCE, "runId": RUN, "jobId": package_job["id"], "actualCheckout": checkout,
    "bytes": len(log), "sha256": hashlib.sha256(log).hexdigest(), "suites": 35,
    "passed": 650, "failed": 0, "ignored": 7, "hostInstallerExecuted": False,
}, indent=2) + "\n").encode())

downloaded = []
for name, basename in [
    ("opencodex-desktop-0.1.10-windows-x64-candidate", "installers.original.zip"),
    ("windows-candidate-evidence", "runner-evidence.original.zip"),
]:
    item, = [a for a in artifacts["artifacts"] if a["name"] == name]
    assert item["workflow_run"]["id"] == RUN and item["workflow_run"]["head_sha"] == SOURCE
    assert not item["expired"] and re.fullmatch(r"sha256:[0-9a-f]{64}", item["digest"])
    target = DESTINATION / basename
    data = fetch(item["archive_download_url"], target, 32 * 1024 * 1024, item["digest"][7:])
    assert len(data) == item["size_in_bytes"]
    entries = []
    with zipfile.ZipFile(target) as archive:
        safe.checked_zip(archive)
        for member in archive.infolist():
            if member.is_dir():
                continue
            payload = archive.read(member)
            entries.append({"path": member.filename, "bytes": len(payload),
                            "sha256": hashlib.sha256(payload).hexdigest()})
            if basename.startswith("installers"):
                output = DESTINATION / "installers" / member.filename
                output.parent.mkdir(parents=True, exist_ok=True)
                if output.exists():
                    assert output.read_bytes() == payload
                else:
                    output.write_bytes(payload)
        if basename.startswith("runner"):
            for member_name, saved_name in [
                ("installer-verification.json", "candidate-installer-verification.original.json"),
                ("environment.json", "candidate-environment.original.json"),
            ]:
                member, = [m for m in archive.infolist() if Path(m.filename).name == member_name]
                output = ROOT / saved_name
                payload = archive.read(member)
                if output.exists():
                    assert output.read_bytes() == payload
                else:
                    output.write_bytes(payload)
    downloaded.append({
        "artifactId": item["id"], "name": name, "path": str(target),
        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
        "entries": entries, "safePathsAndCrcVerified": True,
    })
receipt = json.loads((ROOT / "candidate-installer-verification.original.json").read_text(encoding="utf-8-sig"))
environment = json.loads((ROOT / "candidate-environment.original.json").read_text(encoding="utf-8-sig"))
assert receipt["sourceCommit"] == environment["source"] == SOURCE
assert receipt["version"] == "0.1.10" and receipt["result"] == "pass"
assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
for row in receipt["installers"]:
    package, = (DESTINATION / "installers").rglob(row["name"])
    payload = package.read_bytes()
    assert len(payload) == row["bytes"] and hashlib.sha256(payload).hexdigest() == row["sha256"]
    assert row["result"] == row["firstScreen"] == "pass" and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": RUN,
    "destination": str(DESTINATION), "artifacts": downloaded, "hostInstallerExecuted": False,
    "hostProductStarted": False, "fullVersionGatePassed": False, "releaseRequested": False,
}
(ROOT / "candidate-download-result.json").write_bytes((json.dumps(result, indent=2) + "\n").encode())
print(json.dumps({"runId": RUN, "artifactsVerified": len(downloaded),
                  "destination": str(DESTINATION), "hostProductStarted": False}))
