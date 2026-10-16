"""Verify collector checks and unchanged product identity without a performance claim."""
import hashlib
import json
import re
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")
SOURCE = "f2271f361b9da562b62c71afcdbeb15e61d183e2"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def digest(data):
    return hashlib.sha256(data).hexdigest()


manifest = read("manifest.json")
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(files) == {e["path"] for e in manifest["files"]}
for entry in manifest["files"]:
    data = files[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and digest(data) == entry["sha256"]
provenance = read("source-and-protection.json")
assert provenance["source"] == SOURCE
assert provenance["productAppsTreeUnchanged"] and provenance["etwCollectorFilesUnchanged"]
assert provenance["productSource"] == "d94c215d869c0560717d79dc02881ce8858afe88"
for path, expected in provenance["gitBlobSha256"].items():
    data = subprocess.run(["git", "-C", str(CODE), "show", SOURCE + ":" + path],
                          check=True, capture_output=True).stdout
    assert digest(data) == expected
for name in ("observer-result.json", "ci-result.json"):
    result = read(name)
    assert result["status"] == "passed" and result["checks"] == 55
    assert result["failureRecords"] == 7 and result["controlledPowershellChildrenStarted"] == 1
    assert not result["productStarted"] and not result["etwStarted"] and not result["foregroundActivated"]
    assert not result["longMeasurementRun"] and not result["fixtureForceTerminated"]
    assert result["injectedMissingPidAndGenerationSnapshotsUsed"]
    assert not result["originalPerformanceBudgetChanged"] and not result["fullPerformanceGatePassed"]
expected_stages = {"child-snapshot-generation", "child-process-open", "child-generation-match",
                   "foreground-lost", "window-hidden", "window-minimized", "native-instance-count"}
for name in ("observer-failure-records.original.jsonl", "ci-failure-records.original.jsonl"):
    rows = [json.loads(line) for line in (ROOT / name).read_text(encoding="utf-8-sig").splitlines()]
    assert len(rows) == 7 and {r["stage"] for r in rows} == expected_stages
    assert all(r["observationInvalid"] and not r["measurementRetryRequested"]
               and not r["rawExceptionMessageSaved"] and not r["commandLineSaved"] for r in rows)
    missing, = [r for r in rows if r["stage"] == "child-process-open"]
    assert missing["processId"] == 2147483647 and missing["expectedCreationUtc"]
    assert missing["exceptionType"] and missing["baseExceptionType"] and missing["hresult"] is not None
    mismatch, = [r for r in rows if r["stage"] == "child-generation-match"]
    assert mismatch["actualCreationUtc"] != mismatch["expectedCreationUtc"]
    for row in rows:
        if row["stage"] in {"foreground-lost", "window-hidden", "window-minimized", "native-instance-count"}:
            assert row["lastWindowFrame"] is not None
assert (ROOT / "observer-terminal.json").read_bytes() == (ROOT / "observer-progress.json").read_bytes()
status = read("observer-terminal.json")
assert status["phase"] == "invalid" and status["counterSpanSeconds"] == 0
assert not status["measurementCompleted"] and not status["automaticRetryRequested"]
assert read("parsing-observer.original.json")["status"] == "passed"
smoke = read("smoke-verification.json")
job = read("smoke-job.original.json")
assert smoke["source"] == job["head_sha"] == SOURCE and smoke["runId"] == 38834762492
assert smoke["jobId"] == job["id"] and job["conclusion"] == "success"
step, = [s for s in job["steps"] if s["name"] == "Verify idle counter integrity without a desktop instance"]
assert step["conclusion"] == "success" and smoke["checks"] == 55
log = (ROOT / "smoke-job.original.log").read_bytes()
receipt = read("smoke-job-download.json")
assert len(log) == receipt["bytes"] and digest(log) == receipt["sha256"]
text = re.sub(r"\x1b\[[0-9;]*m", "", log.decode("utf-8-sig"))
assert re.search(r'"checks":\s*55', text)
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == SOURCE
artifact = read("smoke-artifact.original.json")
archive = ROOT / "smoke-artifact.original.zip"
assert artifact["workflow_run"]["id"] == smoke["runId"] and artifact["workflow_run"]["head_sha"] == SOURCE
assert len(archive.read_bytes()) == artifact["size_in_bytes"]
assert digest(archive.read_bytes()) == artifact["digest"][7:] == smoke["artifactSha256"]
with zipfile.ZipFile(archive) as bundle:
    assert bundle.testzip() is None
    for entry in smoke["entries"]:
        data = bundle.read(entry["archivePath"])
        assert data == (ROOT / entry["savedAs"]).read_bytes()
        assert len(data) == entry["bytes"] and digest(data) == entry["sha256"]
assert provenance["nativeCount"] == 0 and provenance["bothCodeWorktreesClean"]
assert all(provenance["sharedFilesIdentical"].values())
assert provenance["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert provenance["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not provenance["nativeLongMeasurementExecuted"] and not provenance["newCandidateBuildRequested"]
summary = {"status": "verified-idle-collector-integrity-fix", "files": len(files),
           "source": SOURCE, "localChecksPassed": 55, "ciChecksPassed": 55,
           "ciRunId": smoke["runId"], "scope": "local regression and completed Windows smoke job",
           "wholeCiStatusAtRead": smoke["wholeCiStatusAtRead"],
           "productAppsUnchanged": True, "productSource": provenance["productSource"],
           "nativeProductStarted": False, "longPerformanceMeasurementExecuted": False,
           "fullPerformanceGatePassed": False, "historicalFailuresCleared": False,
           "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
