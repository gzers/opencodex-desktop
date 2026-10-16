"""Verify recorder preparation; this cannot certify a captured failure window."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "7704a17902c98ceed047cc3dec2a361937b5e1f7"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(files) == {item["path"] for item in manifest["files"]}
for item in manifest["files"]:
    data = files[item["path"]].read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
prep, provenance = read("preparation.original.json"), read("code-provenance.json")
assert provenance["source"] == SOURCE and provenance["productAppsTreeUnchanged"]
code = Path(prep["codeRoot"])


def git(*args):
    return subprocess.run(["git", "-C", str(code), *args], capture_output=True, check=True).stdout


assert git("rev-parse", SOURCE + ":apps") == git("rev-parse", provenance["baseSource"] + ":apps")
for path, expected in provenance["sourceBlobSha256"].items():
    assert hashlib.sha256(git("show", SOURCE + ":" + path)).hexdigest() == expected
before = git("show", provenance["baseSource"] + ":test/smoke/windows-owned-etw.cs")
after = git("show", SOURCE + ":test/smoke/windows-owned-etw.cs")
start, end = b"public sealed class OwnedEtwLineBudget", b"private sealed class PendingIo"
assert before[before.index(start):before.index(end)] == after[after.index(start):after.index(end)]
for item in provenance["inputs"]:
    data = Path(item["path"]).read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
assert provenance["inputs"][0]["sha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert provenance["inputs"][1]["sha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
checks = read("pure-checks-final.original.log")
assert checks["status"] == "passed" and checks["namedQueryThreadScopeChecks"] == 39
assert checks["pureScopeChecks"] == 38 and checks["descendantHotScopeChecks"] == 10
assert checks["utf8LineBudgetChecks"] == 10
assert not checks["etwSessionStarted"] and not checks["testOrNodeStarted"]
assert not checks["recordingRootCreated"] and not checks["diagnosticRecordExecutionTested"]
assert read("powershell-parsing.json")["status"] == "passed"
tools = (ROOT / "etw-ci-tools.original.log").read_text(encoding="utf-8-sig")
assert "Ran 13 tests" in tools and tools.rstrip().endswith("OK")
runs = read("ci-first-read.original.json")["workflow_runs"]
assert len(runs) == 1 and runs[0]["id"] == provenance["runId"] == 38826901050
assert runs[0]["head_sha"] == SOURCE and runs[0]["run_attempt"] == 1
summary = {
    "status": "verified-query-thread-recorder-preparation",
    "source": SOURCE, "files": len(files), "newScopeChecks": 39,
    "admissionTests": 13, "productAppsTreeUnchanged": True,
    "originalInputsStillPinned": True, "etwSessionStarted": False,
    "testOrNodeStarted": False, "liveThreadNameCoverageVerified": False,
    "actualFailureWindowObtained": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
