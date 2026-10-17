"""Verify a bounded review; do not apply its proposed fixture change."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and digest(data) == row["sha256"]
    analysis = read(ROOT / "offline-comparison.json")
    assert analysis["source"] == "1dccf22ec3b93a0b08115eaf794fb45ec9814d2a"
    assert analysis["successfulCiRun"] == 39011042117
    assert analysis["failedCandidateRun"] == 39012344655
    for row in analysis["sourceLogs"]:
        data = Path(row["path"]).read_bytes()
        assert len(data) == row["bytes"] and digest(data) == row["sha256"]
    assert analysis["successfulCiCases"]["success"]["result"] == "Success"
    assert analysis["successfulCiCases"]["nonzero"]["result"] == "Failure"
    assert all(row["result"] == "Timeout" for row in analysis["failedCandidateCases"].values())
    assert not analysis["provedProductDefectCausingFailure"]
    assert not analysis["provedOsSchedulingOrIoRootCause"]
    assert not analysis["sourcePatchApplied"] and not analysis["newCiOrCandidateDispatched"]
    proposal = read(ROOT / "proposal-binding.json")
    assert proposal["assertionCount"] == 16 and proposal["allOriginalDurationLiteralsRetained"]
    assert proposal["simultaneousColdNodeCountReduced"]
    assert proposal["approvalRequiredBeforeApplying"] and proposal["proposalNotRustCompiled"]
    assert not proposal["appliedToImplementation"] and not proposal["ciDispatched"]
    runtime = Path(read(ROOT / "copies.json")[0]["runtimePath"]).parent
    patch = runtime / "control-fixture.proposed.patch"
    assert digest(patch.read_bytes()) == proposal["patchSha256"]
    mechanism = read(ROOT / "control-mechanism.json")
    assert mechanism["retainedHandle"] and mechanism["blockedUntilStdinClosed"]
    assert mechanism["exitAfterStdinEof"] and mechanism["exitCode"] == 1
    assert not mechanism["forceTerminated"] and not mechanism["foregroundUsed"]
    protection = read(ROOT / "protection.json")
    assert protection["nativeCount"] == 0 and protection["implementationTrackedClean"]
    assert protection["originalPreferencesUnchanged"] and protection["originalPointerUnchanged"]
    assert protection["taskUnchanged"] and not protection["candidateDispatched"]


verify()
if "--verify" in sys.argv:
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and digest(data) == row["sha256"]
else:
    with (ROOT / "manifest.json").open("x", encoding="utf-8") as stream:
        json.dump({"files": [
            {"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size,
             "sha256": digest(path.read_bytes())}
            for path in sorted(ROOT.rglob("*"))
            if path.is_file() and path.name != "manifest.json"
        ]}, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
print(json.dumps({"status": "review-verified-decision-pending",
                  "sourceChanged": False, "queryRootCauseProved": False,
                  "ciOrCandidateDispatched": False}))
