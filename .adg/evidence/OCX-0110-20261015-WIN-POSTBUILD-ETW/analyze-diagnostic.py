"""Analyze the single changed-condition run without converting success into failure evidence."""
import collections
import hashlib
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
terminal = read(root / "diagnostic-terminal.original.json")
assert terminal["run"]["id"] == 38702455024 and terminal["run"]["run_attempt"] == 1
assert terminal["run"]["head_sha"] == "8e266abac78ff6d1b78cb05e680c7c5dbe1f0e56"
assert terminal["run"]["status"] == "completed"
package = next(job for job in terminal["jobs"]["jobs"] if job["name"] == "package")
assert package["conclusion"] == "skipped"
download = read(root / "diagnostic-download-verification.json")
for row in download["entries"]:
    data = (root / "diagnostic-original" / row["path"]).read_bytes()
    assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
out = root / "diagnostic-original/out/windows-query-etw"
capture_dir = root / "diagnostic-original/runtime/ci-bd2-query-etw/capture"
admission = read(out / "admission.json")
build = read(out / "post-build-context.original.json")
capture = read(capture_dir / "capture-result.json")
assert admission["collectorSource"] == build["source"] == terminal["run"]["head_sha"]
assert admission["applicationTree"] == build["applicationTree"] == "bf4365586d874d1a9268636e5227efad4621bad1"
assert admission["diagnosticPath"] == build["diagnosticPath"] == "post-build-runner-temp"
assert admission["testSha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert admission["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert admission["nodeVersion"] == "24.21.0"
assert admission["originalLibraryRunsAllowed"] == build["originalLibraryRunsAllowed"] == 1
assert admission["originalLibraryCommandArguments"] == [] and admission["originalConcurrencyAndDeadlinesPreserved"]
assert not build["originalArtifactOrNodeExecutedDuringBuild"] and not build["artificialStressOrPrewarm"]
assert not build["originalFixtureDeadlinesChanged"] and not build["exactHistoricalEnvironmentReproduced"]
assert capture["runnerRegressionEnvironment"] and capture["runnerProfilePreserved"]
assert capture["boundedOriginalRunCount"] == 1 and capture["defaultConcurrencyAndCapturePreserved"]
assert not capture["nodePrewarmed"] and not capture["forcedOuterCleanup"]
assert capture["originalLibraryCompleted"] and capture["ownedRootExited"]
assert capture["stdoutSaved"] and capture["stderrSaved"]
assert capture["rootProcessStartSeen"] and capture["processTraceStarted"] and capture["processTraceJoined"]
assert capture["writerJoined"] and capture["ownedSessionStopSucceeded"] and capture["serializedBytesWithinLimit"]
assert capture["maximumSerializedBytes"] == 64 * 1024 * 1024
assert capture["maximumQueueRecords"] == capture["maximumTrackedIrps"] == 32768
assert capture["kernelEventsLost"] == capture["droppedRecords"] == capture["decoderCallbackErrors"] == 0
assert not capture["knownImageCoverageGap"] and capture["knownImageChildrenNotAdmitted"] == 0
assert not capture["captureError"] and not capture["runError"] and not capture["writerError"]
assert capture["cleanupErrors"] == []
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert not capture["systemWideEtlWritten"] and not capture["unrelatedEventPayloadsSerialized"]
assert not capture["versionSpecificWaitSemanticsVerified"]
raw = (capture_dir / "owned-etw.events.jsonl").read_bytes()
assert len(raw) == capture["actualSerializedBytes"] == capture["queuedSerializedBytes"]
assert raw.endswith(b"\n") and b"\r\n" not in raw and not raw.startswith(b"\xef\xbb\xbf")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"]
starts = [row for row in events if row["kind"] == "process-start"]
stops = [row for row in events if row["kind"] == "process-stop"]
assert collections.Counter(row["pid"] for row in starts) == collections.Counter(row["pid"] for row in stops)
binding = next(row for row in events if row["kind"] == "root-handle-binding")
assert binding["pid"] == capture["testPid"] and binding["createdUtc"] == capture["rootCreationUtc"]
assert binding["runnerProfilePreserved"] and binding["runnerRegressionEnvironment"]
assert binding["fixtureTemporaryDirectory"] == capture["fixtureTemporaryDirectory"]
assert binding["fullLibraryArgumentCount"] == 0 and binding["defaultConcurrency"] and binding["noPrewarm"]
assert binding["actualToken"]["elevated"]
for row in events:
    if row["kind"] == "owned-thread-scheduled-out":
        assert isinstance(row["waitModeDecodedRaw"], int) and isinstance(row["waitReasonDecodedRaw"], int)
        assert isinstance(row["stateDecodedRaw"], int) and not row["versionSpecificWaitSemanticsVerified"]
stdout = (capture_dir / "test-stdout.original.log").read_text(encoding="utf-8-sig")
total = re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", stdout)
assert total
queries = [{"name": name, "result": outcome}
           for name, outcome in re.findall(r"test (\S+) \.\.\. (ok|FAILED)", stdout)
           if name.startswith(("infrastructure::official_version_source::windows_tests::",
                               "modules::about::remote::windows_tests::"))]
assert len(queries) == 7
failures = [row for row in queries if row["result"] == "FAILED"]
report = {
    "source": terminal["run"]["head_sha"], "runId": terminal["run"]["id"],
    "diagnosticPath": admission["diagnosticPath"], "originalLibraryRuns": 1,
    "libraryExitCode": capture["testExitCode"],
    "tests": {"passed": int(total[2]), "failed": int(total[3]), "ignored": int(total[4])},
    "queryFixtures": queries, "targetQueryFailureReproduced": bool(failures),
    "actualFailingQueryExecutionWindowEtwObtained": bool(failures),
    "rootCauseDetermined": False, "historicalFailureCleared": False,
    "processStarts": len(starts), "processStops": len(stops),
    "processRoles": dict(collections.Counter(row["role"] for row in starts)),
    "eventKinds": dict(collections.Counter(row["kind"] for row in events)),
    "eventBytes": len(raw), "eventSha256": hashlib.sha256(raw).hexdigest(),
    "rootPid": capture["testPid"], "rootCreationUtc": capture["rootCreationUtc"],
    "rootToken": binding["actualToken"], "logicalProcessors": binding["logicalProcessors"],
    "fixtureTemporaryDirectory": capture["fixtureTemporaryDirectory"],
    "captureAndCleanupVerified": True, "versionSpecificWaitSemanticsVerified": False,
    "rootSchedulingEventsOmitted": capture["rootSchedulingEventsIntentionallyOmitted"],
    "rootFileIoInitiationsOmitted": capture["rootFileIoInitiationsIntentionallyOmitted"],
    "fullRootHotEventCoverage": False, "exactHistoricalEnvironmentReproduced": False,
    "performanceOrNativeAcceptanceClaimed": False, "fullVersionGatePassed": False,
}
report_path = root / "diagnostic-analysis.json"
if report_path.exists():
    assert read(report_path) == report
else:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
