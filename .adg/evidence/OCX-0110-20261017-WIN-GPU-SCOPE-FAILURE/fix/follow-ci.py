"""Follow only the current first-push handle; never dispatch, replay or launch."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def save(name, value):
    with (ROOT / name).open("x", encoding="utf-8") as out:
        json.dump(value, out, ensure_ascii=False, indent=2)
        out.write("\n")


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


api = load("scope_reader", DOCS / ".adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py")
safe = load("scope_download", REPO / "test/smoke/windows-etw-ci-admit.py")
handle = read(ROOT / "ci-handle.json")
source, run_id = handle["source"], handle["runId"]
assert source == "fde1dbb5b1731c056bbfa72a420bf930133671ea" and run_id == 38946198178
token = api.credential()
_, run = api.request(f"actions/runs/{run_id}", token)
_, jobs = api.request(f"actions/runs/{run_id}/jobs?per_page=100", token)
assert run["head_sha"] == source and run["run_attempt"] == 1
assert run["event"] == "push" and run["path"] == ".github/workflows/ci.yml"
assert jobs["total_count"] == len(jobs["jobs"])
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
snapshot = f"ci-{stamp}.original.json"
save(snapshot, {"atUtc": stamp, "run": run, "jobs": jobs})
method, = [j for j in jobs["jobs"] if j["name"] == "windows-smoke-evidence"]
if method["status"] == "completed" and method["conclusion"] == "success" and not (ROOT / "method-admission.json").exists():
    _, artifacts = api.request(f"actions/runs/{run_id}/artifacts?per_page=100", token)
    assert artifacts["total_count"] == len(artifacts["artifacts"])
    artifact, = [a for a in artifacts["artifacts"] if a["name"] == "windows-process-token-evidence"]
    assert not artifact["expired"] and artifact["workflow_run"]["head_sha"] == source
    assert artifact["workflow_run"]["id"] == run_id
    save("method-job-and-artifacts.original.json", {"job": method, "artifacts": artifacts})
    archive_path = ROOT / "method.original.zip"
    if not archive_path.exists():
        safe.download(artifact["archive_download_url"], archive_path, 16 * 1024 * 1024,
                      token=token, expected=artifact["digest"][7:])
    assert archive_path.stat().st_size == artifact["size_in_bytes"]
    assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == artifact["digest"][7:]
    rows = []
    with zipfile.ZipFile(archive_path) as archive:
        safe.checked_zip(archive)
        for name in archive.namelist():
            if not name.endswith((".json", ".jsonl", ".log")) or not name.startswith(("memory-target/", "controlled-memory/")):
                continue
            data = archive.read(name)
            target = ROOT / "ci-method" / name
            target.parent.mkdir(parents=True, exist_ok=True)
            if target.exists():
                assert target.read_bytes() == data
            else:
                target.write_bytes(data)
            rows.append({"path": target.relative_to(ROOT).as_posix(), "member": name, "bytes": len(data),
                         "sha256": hashlib.sha256(data).hexdigest()})
    result = read(ROOT / "ci-method/memory-target/result.json")
    assert result["passed"] and result["checks"] == 22
    assert result["helpersAvailableAfterOpenReturns"] and result["actualHarnessTokenRereadAfterOpen"]
    assert result["roleParserCheckedAfterOpen"] and not result["productGpuBindingExecuted"]
    blob = subprocess.check_output(["git", "show", source + ":test/smoke/windows-memory-target.ps1"], cwd=REPO)
    variants = [hashlib.sha256(blob).hexdigest(), hashlib.sha256(blob.replace(b"\n", b"\r\n")).hexdigest()]
    assert b"\r\n" not in blob and result["sourceMethodSha256"] in variants
    controlled = read(ROOT / "ci-method/controlled-memory/result.json")
    capture = read(ROOT / "ci-method/controlled-memory/capture/result.json")
    assert controlled["passed"] and controlled["checks"] == 15 and controlled["compiledGuardChecks"] == 20
    assert controlled["administratorToken"] and capture["passed"] and capture["controlledSelf"]
    assert capture["matchingProbeAllocationEvents"] == capture["matchingAllocationStacks"] == 1
    assert capture["probeFreed"] and capture["ownedSessionAbsent"] and capture["readerJoined"]
    assert capture["omittedRecords"] == capture["callbackErrors"] == 0
    assert capture["nativeStop"]["FinalEventsLost"] == capture["nativeStop"]["FinalRealTimeBuffersLost"] == 0
    log = ROOT / "ci-method-job.original.log"
    if not log.exists():
        safe.download(api.BASE + f"actions/jobs/{method['id']}/logs", log, 20 * 1024 * 1024, token=token)
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.read_text(encoding="utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == source
    save("method-admission.json", {
        "source": source, "runId": run_id, "jobId": method["id"], "actualCheckout": checkout,
        "artifactId": artifact["id"], "artifactBytes": artifact["size_in_bytes"],
        "artifactSha256": artifact["digest"][7:], "crcVerified": True, "extracted": rows,
        "jobLogSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
        "bindingChecks": 22, "controlledSelfStackPairs": 1,
        "wholeCiAdmitted": False, "actualProductGpuRecordingExecuted": False,
        "historicalFailureRewritten": False, "hostProductStarted": False,
    })
if run["status"] == "completed" and run["conclusion"] == "success" and not (ROOT / "ci-complete-admission.json").exists():
    safe.verify_ci(run, jobs, source)
    logs = []
    for job in jobs["jobs"]:
        if job["name"] not in safe.REQUIRED_JOBS:
            assert job["name"] == "build" and job["conclusion"] == "skipped"
            continue
        assert job["head_sha"] == source and job["run_attempt"] == 1
        log = ROOT / ("ci-complete-" + job["name"] + ".original.log")
        if job["name"] == "windows-smoke-evidence":
            log = ROOT / "ci-method-job.original.log"
        if not log.exists():
            safe.download(api.BASE + f"actions/jobs/{job['id']}/logs", log, 20 * 1024 * 1024, token=token)
        data = log.read_bytes()
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == source
        results = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        assert all(int(r[1]) == 0 for r in results)
        logs.append({"job": job["name"], "jobId": job["id"], "path": log.name, "actualCheckout": checkout,
                     "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "rustResults": results})
    save("ci-complete-admission.json", {
        "source": source, "runId": run_id, "attempt": 1, "event": run["event"],
        "status": run["status"], "conclusion": run["conclusion"], "snapshot": snapshot,
        "requiredJobLogs": logs, "wholeToolCiAdmitted": True,
        "actualProductGpuRecordingExecuted": False, "hostProductStarted": False,
        "originalBd2FailureCleared": False, "rssFailureCleared": False, "fullProductGateAccepted": False,
    })
print(json.dumps({"source": source, "runId": run_id, "status": run["status"], "conclusion": run["conclusion"],
                  "methodAdmitted": (ROOT / "method-admission.json").exists(),
                  "wholeCiAdmitted": (ROOT / "ci-complete-admission.json").exists(),
                  "jobs": [{"id": j["id"], "name": j["name"], "status": j["status"],
                            "conclusion": j["conclusion"],
                            "active": [s["name"] for s in j["steps"] if s["status"] == "in_progress"]}
                           for j in jobs["jobs"]]}))
