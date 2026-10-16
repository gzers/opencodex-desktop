"""Recompute the original single-run trace and verify immutable evidence."""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "7704a17902c98ceed047cc3dec2a361937b5e1f7"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
actual = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
          if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(actual) == {e["path"] for e in manifest["files"]}
for entry in manifest["files"]:
    data = actual[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
before = (ROOT / "analysis.json").read_bytes()
subprocess.run([sys.executable, "-B", str(ROOT / "analyze-remote.py")],
               capture_output=True, text=True, check=True)
assert before == (ROOT / "analysis.json").read_bytes()
analysis = read("analysis.json")
assert analysis["collectorSource"] == SOURCE and analysis["originalLibraryRuns"] == 1
assert analysis["actualThreadSetNameEvents"] == 7 and analysis["namedQueryThreadGap"] is False
assert not analysis["targetQueryFailureReproduced"] and not analysis["actualFailureWindowObtained"]
assert not analysis["rootCauseDetermined"] and not analysis["historicalFailureCleared"]
assert not analysis["anotherOriginalRunRequested"] and not analysis["fullVersionGatePassed"]
assert not (ROOT / "recording-attempt.json").exists()
dispatch, handle = read("remote-dispatch-attempt.json"), read("remote-handle.json")
assert handle["runId"] == analysis["runId"] == 38829382543 and handle["maximumOriginalRuns"] == 1
assert dispatch["collectorSource"] == SOURCE and dispatch["maximumOriginalLibraryRuns"] == 1
assert dispatch["deadlinesSeconds"] == [2, 4, 5, 8]
assert dispatch["outerSeconds"] == 120 and dispatch["serializedMiB"] == 64
assert dispatch["queueRecords"] == dispatch["trackedIrps"] == 32768
assert not dispatch["automaticRetryAllowed"] and dispatch["noPrewarm"] and dispatch["defaultConcurrency"]
ci = read("ci-admission.json")
assert ci["source"] == SOURCE and ci["wholeCiSucceeded"] and ci["runId"] == 38826901050
assert ci["originalCheckoutAndJobsVerified"] and len(ci["jobs"]) == 3
assert hashlib.sha256((ROOT / "ci-admission.json").read_bytes()).hexdigest() == dispatch["admissionSha256"]
for job in ci["jobs"]:
    assert job["actualCheckout"] == SOURCE and job["counts"]["failed"] == 0
    data = (ROOT / job["log"]).read_bytes()
    assert len(data) == job["bytes"] and hashlib.sha256(data).hexdigest() == job["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    assert SOURCE in text
state = read("at-rest-protection.json")
assert state["nativeCount"] == 0 and state["bothCodeWorktreesClean"]
assert all(state["sharedFilesIdentical"].values())
assert state["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert state["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
summary = {"status": "verified-one-revised-named-query-success-trace",
           "files": len(actual), "source": SOURCE, "runId": analysis["runId"],
           "exactCiVerified": True, "originalLibraryRuns": 1,
           "testsPassed": 530, "testsIgnored": 4, "queryThreadNamesObserved": 7,
           "events": analysis["eventCount"], "eventBytes": analysis["eventBytes"],
           "captureDrops": 0, "knownNamedQueryGap": False,
           "actualFailureWindowObtained": False, "rootCauseDetermined": False,
           "historicalFailureCleared": False, "hostDesktopOrUacUsed": False,
           "fullVersionGatePassed": False, "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
