"""Independently check the original run, artifact, test outcomes and retained owned ETW."""
import collections
import hashlib
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
terminal = read("diagnostic-terminal.json")
assert terminal["run"]["id"] == 38662786466 and terminal["run"]["head_sha"] == sha
assert terminal["run"]["status"] == "completed" and terminal["run"]["conclusion"] == "success"
assert terminal["run"]["run_attempt"] == 1
assert next(row for row in terminal["jobs"]["jobs"] if row["name"] == "package")["conclusion"] == "skipped"
download = read("diagnostic-download-verification.json")
assert download["artifactId"] == 11939861138 and download["apiDigestMatches"] and download["zipCrcAndSafePathsPassed"]
for row in download["entries"]:
    path = root / "diagnostic-original" / row["path"]
    assert path.stat().st_size == row["bytes"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
out = "diagnostic-original/out/windows-query-etw/"
capture_dir = root / "diagnostic-original/runtime/ci-bd2-query-etw/capture"
admission = read(out + "admission.json")
assert admission["collectorSource"] == sha and admission["collectorCiRun"] == 38660621496
assert admission["originalTestSource"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert admission["originalFailedRun"] == 38537166560 and admission["originalArtifactId"] == 11881042603
assert admission["testSha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert admission["nodeVersion"] == "24.21.0"
assert admission["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert admission["originalLibraryRunsAllowed"] == 1 and admission["originalLibraryCommandArguments"] == []
assert not admission["prewarm"] and admission["originalConcurrencyAndDeadlinesPreserved"]
assert not admission["desktopStarted"] and not admission["installerExecuted"]
prep = read(out + "pure-guards.original.json")
assert prep["status"] == "passed" and not prep["etwSessionStarted"] and not prep["testOrNodeStarted"]
assert sum(prep[key] for key in (
    "pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks", "controllerProcessImageChecks",
    "descendantHotScopeChecks", "utf8LineBudgetChecks", "executableAdmissionChecks")) == 91
capture = json.loads((capture_dir / "capture-result.json").read_text(encoding="utf-8-sig"))
assert capture["testExitCode"] == capture["testExitCodeAfterCleanup"] == 0
assert capture["testPid"] == 9668 and capture["originalLibraryCompleted"] and capture["ownedRootExited"]
for key in ("rootProcessStartSeen", "processTraceStarted", "processTraceJoined",
            "ownedSessionStopSucceeded", "writerJoined", "stdoutSaved", "stderrSaved", "serializedBytesWithinLimit"):
    assert capture[key]
assert capture["droppedRecords"] == capture["decoderCallbackErrors"] == capture["kernelEventsLost"] == 0
assert capture["knownImageChildrenNotAdmitted"] == 0 and not capture["knownImageCoverageGap"]
assert capture["captureError"] is capture["runError"] is capture["writerError"] is None
assert capture["cleanupErrors"] == []
assert capture["nativeOwnedSessionStop"] == {
    "Status": 0, "StoppedOrAlreadyAbsent": True, "FinalEventsLost": 0, "FinalRealTimeBuffersLost": 0}
assert not capture["forcedOuterCleanup"] and not capture["nodePrewarmed"]
assert capture["boundedOriginalRunCount"] == 1 and capture["defaultConcurrencyAndCapturePreserved"]
assert not capture["systemWideEtlWritten"] and not capture["unrelatedEventPayloadsSerialized"]
raw = (capture_dir / "owned-etw.events.jsonl").read_bytes()
assert len(raw) == capture["actualSerializedBytes"] == capture["queuedSerializedBytes"] == 5811506
assert not raw.startswith(b"\xef\xbb\xbf") and b"\r\n" not in raw and raw.endswith(b"\n")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"] == 17085
starts = [row for row in events if row["kind"] == "process-start"]
stops = [row for row in events if row["kind"] == "process-stop"]
assert len(starts) == len(stops)
assert {(row["pid"], row["original"]["qpc"]) for row in starts}
assert collections.Counter(row["pid"] for row in starts) == collections.Counter(row["pid"] for row in stops)
assert all(row["apiIdentityObservation"]["fullImageMatches"] for row in starts
           if row["apiIdentityObservation"]["retained"])
root_binding = next(row for row in events if row["kind"] == "root-handle-binding")
assert root_binding["pid"] == capture["testPid"] and root_binding["actualToken"]["elevated"]
assert root_binding["actualToken"]["elevationType"] == 1
assert root_binding["fullLibraryArgumentCount"] == 0 and root_binding["defaultConcurrency"]
assert root_binding["originalCapture"] and root_binding["noPrewarm"]
kinds = collections.Counter(row["kind"] for row in events)
assert kinds["owned-thread-ready"] and kinds["owned-thread-scheduled-in"] and kinds["owned-file-create"]
stdout = (capture_dir / "test-stdout.original.log").read_text(encoding="utf-8-sig")
totals = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", stdout)
assert totals and tuple(map(int, totals.groups())) == (530, 0, 4)
queries = [
    {"name": name, "result": outcome}
    for name, outcome in re.findall(r"test (\S+) \.\.\. (ok|FAILED)", stdout)
    if name.startswith(("infrastructure::official_version_source::windows_tests::",
                        "modules::about::remote::windows_tests::"))
]
assert len(queries) == 7 and all(row["result"] == "ok" for row in queries)
assert (capture_dir / "test-stderr.original.log").stat().st_size == 0
report = {
    "collectorSource": sha, "originalBd2FailedRun": 38537166560, "diagnosticRun": 38662786466,
    "artifactId": 11939861138, "originalUnfilteredLibraryRuns": 1,
    "originalTestExitCode": 0, "tests": {"passed": 530, "failed": 0, "ignored": 4},
    "queryFixtures": queries, "targetQueryFailureReproduced": False,
    "recordedEvents": len(events), "eventBytes": len(raw), "eventSha256": hashlib.sha256(raw).hexdigest(),
    "eventKinds": dict(kinds), "processStarts": len(starts), "processStops": len(stops),
    "roleCounts": dict(collections.Counter(row["role"] for row in starts)),
    "allRetainedProcessStartsHaveStops": True, "knownImageCoverageGap": False,
    "rootToken": root_binding["actualToken"], "rootCreationUtc": capture["rootCreationUtc"],
    "serializationAndKernelDrops": 0, "rootHotEventsOmitted": {
        "scheduling": capture["rootSchedulingEventsIntentionallyOmitted"],
        "fileIoInitiations": capture["rootFileIoInitiationsIntentionallyOmitted"]},
    "fullRootHotEventCoverage": False, "actualFailureTimeWindowEvidenceObtained": False,
    "historicalBd2AndE6FailuresCleared": False, "versionOrPerformanceAcceptanceClaimed": False,
    "localDesktopStarted": False,
}
path = root / "diagnostic-analysis.json"
if path.exists():
    assert read(path.name) == report
else:
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
