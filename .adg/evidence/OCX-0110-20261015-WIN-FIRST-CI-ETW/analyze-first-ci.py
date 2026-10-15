"""Distinguish actual first-regression failures from historical-artifact identity."""
import collections
import hashlib
import gzip
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
run_id = int(sys.argv[1])
assert run_id in {38705201081, 38705207794}
saved = root / f"run-{run_id}"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
download = read(saved / "download-verification.json")
for row in download["selectedEntries"]:
    stored = (saved / "selected-original" / row["savedAs"]).read_bytes()
    assert len(stored) == row["savedBytes"] and hashlib.sha256(stored).hexdigest() == row["savedSha256"]
    data = gzip.decompress(stored) if row["savedAs"].endswith(".jsonl.gz") else stored
    assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
out = saved / "selected-original/out/windows-query-etw"
capture_dir = saved / "selected-original/runtime/ci-bd2-query-etw/capture"
admission = read(out / "admission.json")
attempt = read(out / "first-regression-attempt.json")
capture = read(capture_dir / "capture-result.json")
receipt = read(saved / "selected-original/out/windows-backend/test-executable-sha256.json")
assert admission["firstOrdinaryCiRegression"] and admission["currentCiIsVerificationNotPreviouslyAdmitted"]
assert admission["collectorCiRun"] is None
assert admission["applicationTree"] == "bf4365586d874d1a9268636e5227efad4621bad1"
assert attempt["source"] == admission["collectorSource"]
assert attempt["sha256"] == receipt["Hash"].lower()
assert attempt["arguments"] == capture["executedLibraryArguments"] == ["--show-output"]
assert attempt["libraryExecutionsAllowed"] == capture["boundedOriginalRunCount"] == 1
assert attempt["originalFailureArtifactSha256"] == admission["testSha256"] == (
    "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3")
assert attempt["executableIsOriginalFailedArtifact"] == (attempt["sha256"] == admission["testSha256"])
assert not attempt["executableIsOriginalFailedArtifact"]
assert capture["firstCargoRegression"] and capture["runnerProfilePreserved"]
assert capture["defaultConcurrencyAndCapturePreserved"] and capture["originalLibraryCompleted"]
assert capture["ownedRootExited"] and capture["stdoutSaved"] and capture["stderrSaved"]
assert capture["rootProcessStartSeen"] and capture["processTraceJoined"] and capture["writerJoined"]
assert capture["ownedSessionStopSucceeded"] and capture["serializedBytesWithinLimit"]
assert not capture["forcedOuterCleanup"] and not capture["nodePrewarmed"]
assert capture["droppedRecords"] == capture["kernelEventsLost"] == capture["decoderCallbackErrors"] == 0
assert not capture["knownImageCoverageGap"] and capture["knownImageChildrenNotAdmitted"] == 0
assert capture["cleanupErrors"] == [] and not any(capture[key] for key in ("captureError", "runError", "writerError"))
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
raw = gzip.decompress((capture_dir / "owned-etw.events.jsonl.gz").read_bytes())
assert len(raw) == capture["actualSerializedBytes"] == capture["queuedSerializedBytes"]
assert b"\r\n" not in raw and raw.endswith(b"\n") and not raw.startswith(b"\xef\xbb\xbf")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"]
binding = next(row for row in events if row["kind"] == "root-handle-binding")
assert binding["pid"] == capture["testPid"] and binding["firstCargoRegression"]
assert binding["executedLibraryArguments"] == ["--show-output"]
assert binding["actualToken"]["elevated"]
starts = [row for row in events if row["kind"] == "process-start"]
stops = [row for row in events if row["kind"] == "process-stop"]
assert collections.Counter(row["pid"] for row in starts) == collections.Counter(row["pid"] for row in stops)
text = (capture_dir / "test-stdout.original.log").read_text(encoding="utf-8-sig")
summary = re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
assert summary
queries = [{"test": name, "result": outcome} for name, outcome in re.findall(r"test (\S+) \.\.\. (ok|FAILED)", text)
           if name.startswith(("infrastructure::official_version_source::windows_tests::",
                               "modules::about::remote::windows_tests::"))]
assert len(queries) == 7
failed = [row for row in queries if row["result"] == "FAILED"]
report = {
    "runId": run_id, "event": download["event"], "actualSource": admission["collectorSource"],
    "firstOrdinaryCargoLibraryExecutionCaptured": True, "executions": 1,
    "actualLibrarySha256": attempt["sha256"], "actualLibraryBytes": attempt["bytes"],
    "sameOriginalBd2ApplicationTree": True, "sameHistoricalFailedBinary": False,
    "libraryExitCode": capture["testExitCode"],
    "libraryResult": {"passed": int(summary[2]), "failed": int(summary[3]), "ignored": int(summary[4])},
    "queryFixtures": queries, "queryFailureObservedInCurrentBuild": bool(failed),
    "currentBuildFailureWindowEtwObtained": bool(failed),
    "historicalFixedArtifactFailureWindowEtwObtained": False,
    "rootPid": capture["testPid"], "rootCreationUtc": capture["rootCreationUtc"],
    "rootToken": binding["actualToken"], "recordedEvents": len(events), "eventBytes": len(raw),
    "eventSha256": hashlib.sha256(raw).hexdigest(), "eventKinds": dict(collections.Counter(row["kind"] for row in events)),
    "processStartsAndStops": [len(starts), len(stops)], "captureLossOrDecoderErrors": 0,
    "originalQueryAssertionsOrDeadlinesChanged": False, "rootCauseDetermined": False,
    "historicalFailuresCleared": False, "fullVersionGatePassed": False,
}
path = saved / "analysis.json"
if path.exists():
    assert read(path) == report
else:
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
